use super::*;

pub fn build_impcore_printers<'ctx>(compiler: &mut Compiler<'ctx>) {
    let context = &compiler.context;
    let module = &compiler.module;
    let builder = &compiler.builder;

    let int_type = context.i32_type();
    let str_type = context.i8_type().ptr_type(AddressSpace::default());
    let printf_fn = *compiler.lib.get("printf").unwrap();
    let impcore_printers = [("println", "%i\n"), ("print", "%i"), ("printu", "%u")];

    for (printer_name, fmt_str) in impcore_printers {
        let unary_type = int_type.fn_type(&[int_type.into()], false);
        let fmt_arr = context.const_string(fmt_str.as_bytes(), true);
        let print_fn = module.add_function(printer_name, unary_type, None);
        let block = context.append_basic_block(print_fn, "entry");
        builder.position_at_end(block);
        let int_arg = print_fn.get_first_param().unwrap().into_int_value();
        let alloca = builder.build_alloca(fmt_arr.get_type(), "alloca");
        builder.build_store(alloca, fmt_arr);
        let fmt_ptr = builder.build_bitcast(alloca, str_type, "cast");
        builder.build_call(printf_fn, &[fmt_ptr.into(), int_arg.into()], "printfcall");
        builder.build_return(Some(&int_arg));
        compiler.fpm.run_on(&print_fn);
        compiler.lib.insert(printer_name, print_fn);
    }

    build_printstr(compiler);
}

pub fn build_printstr<'ctx>(compiler: &mut Compiler<'ctx>) {
    let context = &compiler.context;
    let module = &compiler.module;
    let builder = &compiler.builder;

    let int_type = context.i32_type();
    let ptr_type = int_type.ptr_type(AddressSpace::default());
    let str_type = context.i8_type().ptr_type(AddressSpace::default());
    let printf_fn = *compiler.lib.get("printf").unwrap();

    let unary_type = int_type.fn_type(&[ptr_type.into()], false);
    let print_fn = module.add_function("printstr", unary_type, None);
    let block = context.append_basic_block(print_fn, "entry");
    builder.position_at_end(block);

    let iptr_arg = print_fn.get_first_param().unwrap().into_pointer_value();
    let fmt_arr = context.const_string(b"%s", true);
    let fmt_str_alloca = builder.build_alloca(fmt_arr.get_type(), "alloca");
    builder.build_store(fmt_str_alloca, fmt_arr);

    let fmt_ptr = builder.build_bitcast(fmt_str_alloca, str_type, "cast");
    let arg_ptr = builder.build_bitcast(iptr_arg, str_type, "cast");
    builder.build_call(printf_fn, &[fmt_ptr.into(), arg_ptr.into()], "printfcall");
    builder.build_return(Some(&int_type.const_zero()));
    compiler.lib.insert("printstr", print_fn);
}
