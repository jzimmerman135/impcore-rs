use inkwell::AddressSpace;

use crate::ast::AstExpr;

use super::*;

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
    compiler.curr_function = None;

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
    params: &[&'a str],
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    let fn_value = defgen_prototype(name, params, compiler);
    let itype = compiler.context.i32_type();

    let entry = compiler.context.append_basic_block(fn_value, name);
    compiler.builder.position_at_end(entry);
    compiler.curr_function = Some(fn_value);

    compiler.param_table.clear();
    for (&param, param_value) in params.iter().zip(fn_value.get_param_iter()) {
        let alloca = compiler.builder.build_alloca(itype, "alloca");
        compiler
            .builder
            .build_store(alloca, param_value.into_int_value());
        compiler.param_table.insert(param, alloca);
    }

    let body = body.codegen(compiler)?;
    compiler.curr_function = None;
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
    value: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    // setup block
    let itype = compiler.context.i32_type();
    let fn_type = itype.fn_type(&[], false);
    let fn_value = compiler.module.add_function("val", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    compiler.curr_function = Some(fn_value);

    // calculate value
    let n_elem = itype.const_int(1, false);
    let _len = compiler
        .builder
        .build_int_mul(itype.size_of(), n_elem, "n_bytes");

    let v = value.codegen(compiler)?;

    let addr = compiler.builder.build_malloc(v.get_type(), "malloc")?;
    let array = compiler
        .builder
        .build_load(addr, "array")
        .into_pointer_value();

    let global_ptr =
        compiler
            .module
            .add_global(array.get_type(), Some(AddressSpace::default()), name);

    global_ptr.set_initializer(&addr);

    let _store_instr = compiler.builder.build_store(array, v);
    compiler.builder.insert_instruction(&_store_instr, None);

    compiler.builder.build_return(Some(&v));
    compiler.curr_function = None;

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        return Err(format!(
            "Could not declare global pointer variable {} ",
            name
        ));
    }

    Ok(fn_value)
}

pub fn defgen_free_globals<'a>(compiler: &mut Compiler<'a>) -> Result<FunctionValue<'a>, String> {
    let itype = compiler.context.i32_type();
    let fn_type = itype.fn_type(&[], false);
    let fn_value = compiler.module.add_function("cleanup", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);

    if let Some(first_global) = compiler.module.get_first_global() {
        compiler.builder.build_free(first_global.as_pointer_value());
        while let Some(next_global) = first_global.get_next_global() {
            compiler.builder.build_free(next_global.as_pointer_value());
        }
    }

    let v = itype.const_zero();
    compiler.builder.build_return(Some(&v));
    compiler.curr_function = None;

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        return Err("broken free".into());
    }

    Ok(fn_value)
}
