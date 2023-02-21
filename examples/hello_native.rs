use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::AddressSpace;

fn main() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    // Create LLVM code generation objects
    let ctx = Context::create();
    let module = ctx.create_module("program");
    let builder = ctx.create_builder();
    let ee = module.create_execution_engine().unwrap();

    // Create LLVM types
    let str_type = ctx.i8_type().ptr_type(AddressSpace::default());
    let i32_type = ctx.i32_type();

    // Create printf function
    let printf_type = i32_type.fn_type(&[str_type.into()], true);
    let printf = module.add_function("printf", printf_type, Some(Linkage::External));

    // Create main function
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let block = ctx.append_basic_block(main_fn, "entry");
    builder.position_at_end(block);

    // make a string
    let strarr = ctx.const_string(b"Hello World%c\n", true);
    // make room for the string on the stack
    let alloca = builder.build_alloca(strarr.get_type(), "alloca");
    // put the string on the stack
    builder.build_store(alloca, strarr);
    // get a pointer to the string
    let strptr = builder.build_bitcast(alloca, str_type, "cast");

    // CHANGEUP! switching functions
    let get_exclamation_fn = module.add_function("get-exclamation", main_fn_type, None);
    let another_block = ctx.append_basic_block(get_exclamation_fn, "entry");
    builder.position_at_end(another_block);
    builder.build_return(Some(&i32_type.const_int(0x21, false)));

    // Back to main!
    builder.position_at_end(block);
    let exclamation = builder
        .build_call(get_exclamation_fn, &[], "call")
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    // call printf with the string
    builder.build_call(printf, &[strptr.into(), exclamation.into()], "printf");

    // return result of main
    builder.build_return(Some(&i32_type.const_int(0, false)));

    // check everything is all good
    main_fn.verify(true);

    module.print_to_stderr();

    unsafe { ee.run_function_as_main(main_fn, &[]) };
}
