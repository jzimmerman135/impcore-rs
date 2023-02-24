use inkwell::AddressSpace;

use super::*;
use crate::ast::{AstDef, AstExpr, AstType};

impl<'a> AstDef<'a> {
    pub fn defgen(&self, compiler: &mut Compiler<'a>) -> Result<NativeTopLevel<'a>, String> {
        compiler.clear_curr_function();
        let native = match self {
            Self::Function(name, params, body) => NativeTopLevel::FunctionDef(
                defgen::defgen_function(name, params, body, compiler)?,
                name,
            ),
            Self::TopLevelExpr(body) => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_anonymous(body, compiler)?)
            }
            Self::CheckAssert(body, contents) => {
                NativeTopLevel::CheckAssert(defgen::defgen_anonymous(body, compiler)?, contents)
            }
            Self::CheckExpect(lhs, rhs, contents) => NativeTopLevel::CheckExpect(
                defgen::defgen_anonymous(lhs, compiler)?,
                defgen::defgen_anonymous(rhs, compiler)?,
                contents,
            ),
            Self::CheckError(..) => todo!(),
            Self::Global(name, value, var_type) => NativeTopLevel::TopLevelExpr(
                defgen::defgen_global(name, value, *var_type, compiler)?,
            ),
            Self::DeclareGlobal(name) => {
                defgen::declare_global(name, compiler);
                NativeTopLevel::Noop
            }
            Self::FreeAll => NativeTopLevel::FreeAll(defgen::defgen_cleanup(compiler)?),
            Self::ImportLib("stdin") => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_stdin(compiler)?)
            }
            Self::ImportLib(name) => {
                return Err(format!("Unbound library {}, got {:?}", name, self))
            }
            _ => unreachable!("unreacheable defgen {:?}", self),
        };
        Ok(native)
    }
}

pub fn declare_global<'a>(name: &'a str, compiler: &mut Compiler<'a>) {
    let addr_space = AddressSpace::default();
    let ptr_type = compiler.context.i32_type().ptr_type(addr_space);
    let global_ptr = compiler.module.add_global(ptr_type, Some(addr_space), name);
    global_ptr.set_initializer(&ptr_type.const_null());
    compiler.global_table.insert(name, global_ptr);
}

pub fn declare_fileptr<'a>(name: &'a str, compiler: &mut Compiler<'a>) {
    let addr_space = AddressSpace::default();
    let fileptr_type = compiler.context.i8_type().ptr_type(addr_space);
    let global_ptr = compiler
        .module
        .add_global(fileptr_type, Some(addr_space), name);
    global_ptr.set_initializer(&fileptr_type.const_null());
    compiler.global_table.insert(name, global_ptr);
}

fn printres<'a>(value: &IntValue<'a>, compiler: &Compiler<'a>) {
    if compiler.quiet_mode {
        return;
    }
    let v = *value;
    let println = *compiler.lib.get("println").unwrap();
    compiler
        .builder
        .build_call(println, &[v.into()], "printres");
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

    printres(&v, compiler);
    compiler.builder.build_return(Some(&v));

    compiler.module.print_to_file("error.ll").unwrap();
    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe { fn_value.delete() };
        return Err(format!(
            "Could not verify anonymous expression \n{:?}\n{:?}",
            body, fn_value
        ));
    }

    Ok(fn_value)
}

pub fn defgen_function<'a>(
    name: &str,
    params: &[(&'a str, AstType)],
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    let fn_value = defgen_prototype(name, params, compiler);
    let int_type = compiler.context.i32_type();
    let ptr_type = int_type.ptr_type(AddressSpace::default());

    let entry = compiler.context.append_basic_block(fn_value, name);
    compiler.builder.position_at_end(entry);
    compiler.curr_function = Some(fn_value);

    for (&param, param_value) in params.iter().zip(fn_value.get_param_iter()) {
        let alloca = match param.1 {
            AstType::Integer => {
                let param_int = param_value.into_int_value();
                let alloca = compiler.builder.build_alloca(int_type, "alloca");
                compiler.builder.build_store(alloca, param_int);
                alloca
            }
            AstType::Pointer => {
                let param_ptr = param_value.into_pointer_value();
                let alloca = compiler.builder.build_alloca(ptr_type, "alloca");
                compiler.builder.build_store(alloca, param_ptr);
                alloca
            }
        };
        compiler.param_table.insert(param.0, alloca);
    }

    let body = body.codegen(compiler)?;
    compiler.builder.build_return(Some(&body));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe { fn_value.delete() };
        return Err(format!("Could not verify function {}", name));
    }

    compiler.fpm.run_on(&fn_value);
    Ok(fn_value)
}

fn defgen_prototype<'a>(
    name: &str,
    params: &[(&str, AstType)],
    compiler: &Compiler<'a>,
) -> FunctionValue<'a> {
    let int_type = compiler.context.i32_type();
    let ptr_type = int_type.ptr_type(AddressSpace::default());

    let mut args_types: Vec<BasicMetadataTypeEnum> = vec![];
    for param in params {
        match param.1 {
            AstType::Integer => args_types.push(int_type.into()),
            AstType::Pointer => args_types.push(ptr_type.into()),
        }
    }

    let fn_type = compiler.context.i32_type().fn_type(&args_types, false);
    let fn_val = compiler.module.add_function(name, fn_type, None);
    for (i, arg) in fn_val.get_param_iter().enumerate() {
        match params[i].1 {
            AstType::Integer => arg.into_int_value().set_name(params[i].0),
            AstType::Pointer => arg.into_pointer_value().set_name(params[i].0),
        }
    }
    fn_val
}

pub fn defgen_global<'a>(
    name: &'a str,
    body: &AstExpr<'a>,
    var_type: AstType,
    compiler: &mut Compiler<'a>,
) -> Result<FunctionValue<'a>, String> {
    // setup block
    let int_type = compiler.context.i32_type();
    let fn_type = int_type.fn_type(&[], false);
    let fn_value = compiler.module.add_function("val", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    compiler.curr_function = Some(fn_value);
    let body_value = body.codegen(compiler)?;

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

    let (array, retval) = if let AstType::Pointer = var_type {
        let size = body_value;
        let sizeof_int =
            compiler
                .builder
                .build_int_cast(size.get_type().size_of(), int_type, "cast");
        let n_bytes = compiler.builder.build_int_mul(size, sizeof_int, "bytes");
        let array = compiler
            .builder
            .build_array_malloc(int_type, size, "array")?;
        compiler.builder.build_memset(
            array,
            4,
            compiler.context.i8_type().const_zero(),
            n_bytes,
        )?;
        (array, size)
    } else {
        let array = compiler.builder.build_malloc(int_type, "single")?;
        compiler.builder.build_store(array, body_value);
        (array, body_value)
    };

    compiler
        .builder
        .build_store(global_ptr.as_pointer_value(), array);
    printres(&retval, compiler);
    compiler.builder.build_return(Some(&retval));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe { fn_value.delete() };
        return Err(format!(
            "Could not mutate global pointer variable {} ",
            name
        ));
    }

    Ok(fn_value)
}

pub fn defgen_stdin<'a>(compiler: &mut Compiler<'a>) -> Result<FunctionValue<'a>, String> {
    compiler.add_stdin();
    compiler
        .lib
        .get("init_stdin")
        .ok_or_else(|| "Couldn't find it".to_string())
        .copied()
}

pub fn defgen_cleanup<'a>(compiler: &mut Compiler<'a>) -> Result<FunctionValue<'a>, String> {
    let itype = compiler.context.i32_type();
    let fn_type = itype.fn_type(&[], false);
    let fn_value = compiler.module.add_function("cleanup", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    compiler.curr_function = Some(fn_value);

    for (name, global_ptr) in compiler.global_table.iter() {
        if name.starts_with('#') {
            continue;
        }
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
