use inkwell::{
    self,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, IntValue},
    OptimizationLevel,
};

fn add_function<'ctx, F>(
    codegen: F,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    fn_name: &'static str,
) -> FunctionValue<'ctx>
where
    F: FnOnce(&'ctx Context, &Module<'ctx>, &Builder<'ctx>) -> IntValue<'ctx>,
{
    let fn_type = context.i32_type().fn_type(&[], false);
    let function = module.add_function(fn_name, fn_type, None);
    let basic_block = context.append_basic_block(function, "top_level_entry");
    builder.position_at_end(basic_block);
    let return_val = codegen(context, module, builder);

    builder.build_return(Some(&return_val));

    assert!(function.verify(false));
    function
}

fn main() {
    let context = Context::create();
    let module = context.create_module("bug-example");
    let builder = context.create_builder();

    let f1 = add_function(
        |context, _, _| context.i32_type().const_int(20, false),
        &context,
        &module,
        &builder,
        "",
    );

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let res1 = unsafe { execution_engine.run_function(f1, &[]) };

    let f2 = add_function(
        |context, _, _| {
            let lhs = context.i32_type().const_int(50, false);
            let rhs = context.i32_type().const_int(13, false);
            builder.build_int_add(lhs, rhs, "add")
        },
        &context,
        &module,
        &builder,
        "",
    );

    // let execution_engine = module
    //     .create_jit_execution_engine(OptimizationLevel::None)
    //     .unwrap();

    let res2 = unsafe { execution_engine.run_function(f2, &[]) };
    println!("f1 res: {}, from {:?}", res1.as_int(true), f1);
    println!("f2 res: {}, from {:?}", res2.as_int(true), f2);
}
