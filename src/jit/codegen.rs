use crate::ast;

use super::*;

impl<'ctx> Compiler<'ctx> {
    pub fn codegen(&mut self, node: &'ctx AstNode) -> Result<IntValue<'ctx>, String> {
        match node {
            AstNode::Literal(inner) => self.codegen_literal(inner),
            AstNode::Variable(inner) => self.codegen_variable(inner),
            AstNode::Binary(inner) => self.codegen_binary(inner),
            AstNode::Unary(inner) => self.codegen_unary(inner),
            AstNode::Call(inner) => self.codegen_call(inner),
            _ => todo!(),
            // AstNode::If(inner) => inner.codegen(compiler),
            // AstNode::While(inner) => inner.codegen(compiler),
            // AstNode::Begin(inner) => inner.codegen(compiler),
            // AstNode::Assign(inner) => inner.codegen(compiler),
            // AstNode::NewGlobal(inner) => inner.codegen(compiler),
            // AstNode::Error(inner) => inner.codegen(compiler),
        }
    }

    fn codegen_literal(&mut self, literal: &'ctx ast::Literal) -> Result<IntValue<'ctx>, String> {
        Ok(self.context.i32_type().const_int(literal.0 as u64, false))
    }

    fn codegen_variable(
        &mut self,
        variable: &'ctx ast::Variable,
    ) -> Result<IntValue<'ctx>, String> {
        match self.formal_table.get(variable.0) {
            Some(&val) => Ok(val),
            None => match self.global_table.get(variable.0) {
                Some(&val) => Ok(val),
                None => Err(format!("variable {} not found", variable.0)),
            },
        }
    }
    fn codegen_binary(&mut self, binary: &'ctx ast::Binary) -> Result<IntValue<'ctx>, String> {
        let operator = binary.0;
        let lhs = self.codegen(&binary.1)?;
        let rhs = self.codegen(&binary.2)?;
        Ok(match operator {
            "*" => self.builder.build_int_mul(lhs, rhs, "mul"),
            "/" => self.builder.build_int_signed_div(lhs, rhs, "div"),
            "+" => self.builder.build_int_add(lhs, rhs, "mul"),
            "-" => self.builder.build_int_sub(lhs, rhs, "sub"),
            _ => todo!(),
        })
    }

    fn codegen_unary(&mut self, unary: &'ctx ast::Unary) -> Result<IntValue<'ctx>, String> {
        let operator = unary.0;
        let arg = self.codegen(&unary.1)?;
        let one = self.context.i32_type().const_int(1, false);
        Ok(match operator {
            "++" => self.builder.build_int_add(arg, one, "incr"),
            "--" => self.builder.build_int_sub(arg, one, "decr"),
            _ => todo!(),
        })
    }

    fn codegen_call(&mut self, call: &'ctx ast::Call) -> Result<IntValue<'ctx>, String> {
        let function_name = call.0;
        let arg_nodes = call.1.iter();
        let function = match self.module.get_function(function_name) {
            Some(f) => f,
            None => return Err(format!("Could not find function {}", function_name)),
        };
        let args = arg_nodes
            .map(|e| match self.codegen(e) {
                Ok(val) => Ok(BasicMetadataValueEnum::IntValue(val)),
                Err(str) => Err(str),
            })
            .collect::<Result<Vec<_>, String>>()?;
        match self
            .builder
            .build_call(function, &args, "user_func_call")
            .try_as_basic_value()
            .left()
        {
            Some(BasicValueEnum::IntValue(inner)) => Ok(inner),
            _ => unreachable!(),
        }
    }
}
