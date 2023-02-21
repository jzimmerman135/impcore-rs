use super::*;
use crate::ast::AstType;

pub fn build_implib_printers(compiler: &mut Compiler) {
    let context = &compiler.context;
    let module = &compiler.module;
    let builder = &compiler.builder;

    let int_type = context.i32_type();
    let ptr_type = int_type.ptr_type(AddressSpace::default());
    let printf_fn = *compiler
        .lib
        .get("printf")
        .expect("Trying to build implib printers without printf");

    let impcore_printers = [
        ("println", "%i\n", "fmt_ln", AstType::Integer),
        ("print", "%i", "fmt_i", AstType::Integer),
        ("printu", "%u", "fmt_u", AstType::Integer),
        ("printstr", "%s", "fmt_str", AstType::Pointer),
    ];

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
        compiler.lib.insert(printer_name, print_fn);
    }
}

pub fn add_printres<'ctx>(
    fn_value: FunctionValue<'ctx>,
    compiler: &Compiler<'ctx>,
) -> FunctionValue<'ctx> {
    let block = fn_value.get_last_basic_block().unwrap();
    compiler.builder.position_at_end(block);
    let fifty = compiler.context.i32_type().const_int(50, false);
    let println = *compiler.lib.get("println").unwrap();
    compiler
        .builder
        .build_call(println, &[fifty.into()], "printres");
    fn_value
}
