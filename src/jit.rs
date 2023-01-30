use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{self, ExecutionEngine, JitFunction},
    module::Module,
    values::IntValue,
};

use crate::ast::AstNode;

#[derive(Debug)]
#[allow(unused)]
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("top level");
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .expect("Failed to build LLVM JIT Engine");
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            execution_engine,
        }
    }

    #[allow(unused)]
    pub fn codegen(&mut self, expr: AstNode) -> IntValue<'ctx> {
        match expr {
            AstNode::Add(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder.build_int_add(lhs, rhs, "add")
            }
            AstNode::Sub(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder.build_int_sub(lhs, rhs, "sub")
            }
            AstNode::Mul(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder.build_int_mul(lhs, rhs, "mul")
            }
            AstNode::Div(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder.build_int_mul(lhs, rhs, "div")
            }
            AstNode::Mod(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder.build_int_signed_rem(lhs, rhs, "mod")
            }
            AstNode::Eq(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "eq")
            }
            AstNode::Le(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "eq")
            }
            AstNode::Lt(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, "eq")
            }
            AstNode::Ge(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "eq")
            }
            AstNode::Gt(lhs, rhs) => {
                let (lhs, rhs) = (self.codegen(*lhs), self.codegen(*rhs));
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "eq")
            }
            AstNode::Not(expr) => {
                let expr = self.codegen(*expr);
                self.builder.build_not(expr, "not")
            }
            _ => unreachable!("reached unreachable {:?}", expr),
        }
    }
}
