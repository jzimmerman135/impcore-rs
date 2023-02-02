use std::collections::HashMap;

use inkwell::{
    builder::Builder, context::Context, execution_engine::ExecutionEngine, module::Module,
    support::LLVMString, values::IntValue, OptimizationLevel,
};

use crate::ast::{Binary, Literal, Unary, Variable};

pub trait CodeGen {
    fn codegen<'a, 'c>(&'a self, compiler: &'c Compiler) -> Result<IntValue<'c>, String>;
}

#[derive(Debug)]
#[allow(unused)]
pub struct Compiler<'a> {
    pub context: &'a Context,
    pub module: Module<'a>,
    pub builder: Builder<'a>,
    pub formal_table: HashMap<&'a str, IntValue<'a>>,
    pub execution_engine: ExecutionEngine<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(context: &'a Context) -> Result<Self, LLVMString> {
        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
        Ok(Self {
            context,
            module,
            builder,
            execution_engine,
            formal_table: HashMap::new(),
        })
    }
}

impl<'a> CodeGen for Literal {
    fn codegen<'c>(&self, compiler: &'c Compiler) -> Result<IntValue<'c>, String> {
        Ok(compiler.context.i32_type().const_int(self.0 as u64, false))
    }
}

impl<'a> CodeGen for Variable<'a> {
    fn codegen<'c>(&self, compiler: &'c Compiler) -> Result<IntValue<'c>, String> {
        match compiler.formal_table.get(self.0) {
            Some(&val) => Ok(val),
            None => Err(format!("variable {} not found", self.0)),
        }
    }
}

impl<'a> CodeGen for Binary<'a> {
    fn codegen<'c>(&self, compiler: &'c Compiler) -> Result<IntValue<'c>, String> {
        let operator = self.0;
        let lhs = self.1.codegen(compiler)?;
        let rhs = self.2.codegen(compiler)?;
        Ok(match operator {
            "*" => compiler.builder.build_int_mul(lhs, rhs, "mul"),
            "/" => compiler.builder.build_int_signed_div(lhs, rhs, "div"),
            "+" => compiler.builder.build_int_add(lhs, rhs, "mul"),
            "-" => compiler.builder.build_int_sub(lhs, rhs, "sub"),
            _ => todo!(),
        })
    }
}

impl<'a> CodeGen for Unary<'a> {
    fn codegen<'c>(&self, compiler: &'c Compiler) -> Result<IntValue<'c>, String> {
        let operator = self.0;
        let arg = self.1.codegen(compiler)?;
        Ok(match operator {
            "++" => compiler.builder.build_int_add(
                arg,
                compiler.context.i32_type().const_int(1, false),
                "incr",
            ),
            "--" => compiler.builder.build_int_sub(
                arg,
                compiler.context.i32_type().const_int(1, false),
                "incr",
            ),
            _ => todo!(),
        })
    }
}
