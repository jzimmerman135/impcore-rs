use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
};

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

pub type IfFunc = unsafe extern "C" fn(u32, u32, u32) -> u32;
pub type BinaryFunc = unsafe extern "C" fn(u32, u32) -> u32;
pub type UnaryFunc = unsafe extern "C" fn(u32, u32) -> u32;

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_binary(&self) -> Option<JitFunction<BinaryFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("add", fn_type, None);

        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();

        let res = self.builder.build_int_add(x, y, "add");

        self.builder.build_return(Some(&res));

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}
