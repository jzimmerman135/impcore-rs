use inkwell::AddressSpace;

use super::*;
use crate::ast::AstExpr;

pub fn declare_global<'a>(name: &'a str, compiler: &mut Compiler<'a>) {
    let addr_space = AddressSpace::default();
    let ptr_type = compiler.context.i32_type().ptr_type(addr_space);
    let global_ptr = compiler.module.add_global(ptr_type, Some(addr_space), name);
    global_ptr.set_initializer(&ptr_type.const_null());
    compiler.global_table.insert(name, global_ptr);
}

pub fn defgen_anonymous<'a>(
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    let fn_type = compiler.context.i32_type().fn_type(&[], false);
    let fn_value = compiler.module.add_function("#anon", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    compiler.curr_function = Some(fn_value);
    let v = body.codegen(compiler)?;
    compiler.builder.build_return(Some(&v));

    compiler.module.print_to_file("error.ll").unwrap();
    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        return Err(format!(
            "Could not verify anonymous expression \n{:?}\n{:?}",
            body, fn_value
        ));
    }

    Ok(fn_value)
}

pub fn defgen_function<'a>(
    name: &str,
    variable_params: &[&'a str],
    pointer_params: &[&'a str],
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    let fn_value = defgen_prototype(name, variable_params, compiler);
    let int_type = compiler.context.i32_type();
    let ptr_type = int_type.ptr_type(AddressSpace::default());

    let entry = compiler.context.append_basic_block(fn_value, name);
    compiler.builder.position_at_end(entry);
    compiler.curr_function = Some(fn_value);

    for (&param, param_value) in variable_params.iter().zip(fn_value.get_param_iter()) {
        let alloca = compiler.builder.build_alloca(int_type, "alloca");
        compiler
            .builder
            .build_store(alloca, param_value.into_int_value());
        compiler.param_table.insert(param, alloca);
    }

    let body = body.codegen(compiler)?;
    compiler.builder.build_return(Some(&body));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe {
            fn_value.delete();
        }
        return Err(format!("Could not verify function {}", name));
    }

    compiler.fpm.run_on(&fn_value);

    Ok(fn_value)
}

fn defgen_prototype<'a>(name: &str, params: &[&str], compiler: &Compiler<'a>) -> FunctionValue<'a> {
    let ret_type = compiler.context.i32_type();
    let args_types = std::iter::repeat(ret_type)
        .take(params.len())
        .map(|f| f.into())
        .collect::<Vec<BasicMetadataTypeEnum>>();
    let fn_type = compiler.context.i32_type().fn_type(&args_types, false);
    let fn_val = compiler.module.add_function(name, fn_type, None);
    for (i, arg) in fn_val.get_param_iter().enumerate() {
        arg.into_int_value().set_name(params[i]);
    }
    fn_val
}

pub fn defgen_global<'a>(
    name: &'a str,
    maybe_size: Option<&AstExpr<'a>>,
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    // setup block
    let int_type = compiler.context.i32_type();
    let fn_type = int_type.fn_type(&[], false);
    let fn_value = compiler.module.add_function("val", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    compiler.curr_function = Some(fn_value);
    let int_value = body.codegen(compiler)?;

    let global_ptr = match compiler.global_table.get(name) {
        Some(&global_ptr) => global_ptr,
        None => compiler
            .module
            .get_global(name)
            .map(|g| {
                compiler.global_table.insert(name, g);
                g
            })
            .ok_or(format!("Unbound global variable {}", name))?,
    };

    let array = compiler
        .builder
        .build_load(global_ptr.as_pointer_value(), "load");
    compiler.builder.build_free(array.into_pointer_value());

    let (array, retval) = if let Some(size_expr) = maybe_size {
        let size = size_expr.codegen(compiler)?;
        let array = compiler
            .builder
            .build_array_malloc(int_type, size, "array")?;
        compiler
            .builder
            .build_memset(array, 4, compiler.context.i8_type().const_zero(), size)?;
        (array, size)
    } else {
        let array = compiler.builder.build_malloc(int_type, "single")?;
        compiler.builder.build_store(array, int_value);
        (array, int_value)
    };

    compiler
        .builder
        .build_store(global_ptr.as_pointer_value(), array);
    compiler.builder.build_return(Some(&retval));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        return Err(format!(
            "Could not declare global pointer variable {} ",
            name
        ));
    }

    Ok(fn_value)
}

pub fn defgen_cleanup<'a>(compiler: &mut Compiler<'a>) -> Result<FunctionValue<'a>, String> {
    let itype = compiler.context.i32_type();
    let fn_type = itype.fn_type(&[], false);
    let fn_value = compiler.module.add_function("cleanup", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    compiler.curr_function = Some(fn_value);

    for (_, global_ptr) in compiler.global_table.iter() {
        let array = compiler
            .builder
            .build_load(global_ptr.as_pointer_value(), "load");
        compiler.builder.build_free(array.into_pointer_value());
    }

    let v = itype.const_zero();
    compiler.builder.build_return(Some(&v));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        return Err("broken free".into());
    }

    Ok(fn_value)
}
