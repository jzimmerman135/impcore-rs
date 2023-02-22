use super::*;
use crate::ast::AstType;
pub mod output {
    use super::*;

    pub fn add_print_functions(compiler: &mut Compiler) {
        let printers = build_implib_printers(compiler);
        for (name, function) in printers {
            compiler.lib.insert(name, function);
        }
    }

    fn build_implib_printers<'a>(compiler: &Compiler<'a>) -> Vec<(&'a str, FunctionValue<'a>)> {
        let context = &compiler.context;
        let module = &compiler.module;
        let builder = &compiler.builder;

        let int_type = context.i32_type();
        let ptr_type = int_type.ptr_type(AddressSpace::default());
        let printf_fn = *compiler
            .lib
            .get("__printf")
            .expect("Trying to build implib printers without printf");

        let impcore_printers = [
            ("println", "%i\n", "fmt_ln", AstType::Integer),
            ("print", "%i", "fmt_i", AstType::Integer),
            ("printu", "%u", "fmt_u", AstType::Integer),
            ("printstr", "%s", "fmt_str", AstType::Pointer),
        ];

        let mut print_functions = vec![];
        for (printer_name, fmt_str, fmt, arg_type) in impcore_printers {
            let unary_type = int_type.fn_type(
                &[match arg_type {
                    AstType::Pointer => ptr_type.into(),
                    AstType::Integer => int_type.into(),
                }],
                false,
            );
            let print_fn = module.add_function(printer_name, unary_type, None);
            let block = context.append_basic_block(print_fn, "entry");
            builder.position_at_end(block);
            let fmt_ptr = builder
                .build_global_string_ptr(fmt_str, fmt)
                .as_pointer_value();
            let param = print_fn.get_first_param().unwrap();
            builder.build_call(printf_fn, &[fmt_ptr.into(), param.into()], "printfcall");
            let retval = match arg_type {
                AstType::Integer => param.into_int_value(),
                AstType::Pointer => int_type.const_zero(),
            };
            builder.build_return(Some(&retval));
            compiler.fpm.run_on(&print_fn);
            print_functions.push((printer_name, print_fn));
        }

        print_functions
    }
}

pub mod input {
    use super::*;

    pub fn add_stdin(compiler: &mut Compiler) {
        let addr_space = AddressSpace::default();
        let int_type = compiler.context.i32_type();
        let str_type = compiler.context.i8_type().ptr_type(addr_space);
        let fileptr_type = compiler
            .context
            .opaque_struct_type("FILE")
            .ptr_type(addr_space);

        declare_global_stdin(compiler);
        let fdopen_type = fileptr_type.fn_type(&[int_type.into(), str_type.into()], false);
        let fdopen_fn =
            compiler
                .module
                .add_function("fdopen", fdopen_type, Some(Linkage::External));

        let fgetc_type = int_type.fn_type(&[fileptr_type.into()], false);
        let fgetc_fn = compiler
            .module
            .add_function("fgetc", fgetc_type, Some(Linkage::External));

        compiler.lib.insert("__fdopen", fdopen_fn);
        compiler.lib.insert("__fgetc", fgetc_fn);
        compiler
            .lib
            .insert("init_stdin", build_init_stdin(compiler));
        compiler.lib.insert("getc", build_fgetc(compiler));
    }

    fn declare_global_stdin(compiler: &mut Compiler) {
        let fileptr_type = compiler.context.i8_type().ptr_type(AddressSpace::default());
        let stdin_global =
            compiler
                .module
                .add_global(fileptr_type, Some(AddressSpace::default()), "__stdin");
        stdin_global.set_initializer(&fileptr_type.const_null());
        compiler.global_table.insert("#__stdin", stdin_global);
    }
    fn build_init_stdin<'ctx>(compiler: &Compiler<'ctx>) -> FunctionValue<'ctx> {
        let int_type = compiler.context.i32_type();
        let void_type = compiler.context.void_type();
        let str_type = compiler.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = void_type.fn_type(&[], false);
        let fn_value = compiler.module.add_function("__init_stdin", fn_type, None);
        let block = compiler.context.append_basic_block(fn_value, "entry");
        compiler.builder.position_at_end(block);

        let alloca = compiler
            .builder
            .build_alloca(str_type.ptr_type(AddressSpace::default()), "fp");

        let global_stdin = compiler
            .global_table
            .get("#__stdin")
            .unwrap()
            .as_pointer_value();

        compiler.builder.build_store(alloca, global_stdin);

        let readstr = compiler
            .builder
            .build_global_string_ptr("r", "__fdopen_arg_read")
            .as_pointer_value();

        let fdopen = *compiler.lib.get("__fdopen").expect("Fdopen not in implib");
        let stdin_fp = compiler
            .builder
            .build_call(
                fdopen,
                &[int_type.const_zero().into(), readstr.into()],
                "fdopen",
            )
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let stdin_void_ptr = compiler
            .builder
            .build_pointer_cast(stdin_fp, str_type, "voidcast");

        let ptr_to_global = compiler
            .builder
            .build_load(alloca, "load")
            .into_pointer_value();

        compiler.builder.build_store(ptr_to_global, stdin_void_ptr);
        compiler.builder.build_return(None);

        if !fn_value.verify(true) {
            compiler.module.print_to_stderr();
            panic!("failed to verify the init stdin function");
        }

        fn_value
    }

    fn build_fgetc<'ctx>(compiler: &Compiler<'ctx>) -> FunctionValue<'ctx> {
        let int_type = compiler.context.i32_type();
        let fn_type = int_type.fn_type(&[], false);
        let fileptr_type = compiler
            .context
            .get_struct_type("FILE")
            .unwrap()
            .ptr_type(AddressSpace::default());
        let fgetc_fn = *compiler.lib.get("__fgetc").unwrap();

        let fn_value = compiler.module.add_function("getc", fn_type, None);
        let block = compiler.context.append_basic_block(fn_value, "entry");
        compiler.builder.position_at_end(block);

        let global_stdin = compiler.global_table.get("#__stdin").unwrap();
        let stdin_ptr = compiler
            .builder
            .build_load(global_stdin.as_pointer_value(), "stdin")
            .into_pointer_value();
        let fileptr = compiler
            .builder
            .build_bitcast(stdin_ptr, fileptr_type, "fp");

        let v = compiler
            .builder
            .build_call(fgetc_fn, &[fileptr.into()], "call")
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        compiler.builder.build_return(Some(&v));

        if !fn_value.verify(true) {
            compiler.module.print_to_stderr();
            panic!("couldn't verify fgetc");
        }
        fn_value
    }
}
