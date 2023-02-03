use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    support::LLVMString,
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue},
    OptimizationLevel,
};

use crate::ast::{self, AstNode};

pub trait CodeGen {
    fn codegen<'a, 'c>(&self, compiler: &'c mut Compiler) -> Result<IntValue<'c>, String>;
}

#[derive(Debug)]
#[allow(unused)]
pub struct Compiler<'a> {
    pub context: &'a Context,
    pub module: Module<'a>,
    pub builder: Builder<'a>,
    pub global_table: HashMap<&'a str, IntValue<'a>>,
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
            global_table: HashMap::new(),
            formal_table: HashMap::new(),
        })
    }
}

impl<'ctx> Compiler<'ctx> {
    pub fn defgen(&mut self, node: &'ctx AstNode) -> Result<FunctionValue<'ctx>, String> {
        match node {
            AstNode::Function(inner) => self.codegen_function(inner),
            _ => todo!(),
        }
    }

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
        let lhs = self.codegen(&*binary.1)?;
        let rhs = self.codegen(&*binary.2)?;
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
        let arg = self.codegen(&*unary.1)?;
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

    fn protogen<'a>(&self, name: &'a str, params: &[&&'a str]) -> FunctionValue<'ctx> {
        let ret_type = self.context.i32_type();
        let args_types = std::iter::repeat(ret_type)
            .take(params.len())
            .map(|f| f.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let fn_type = self.context.i32_type().fn_type(&args_types, false);
        let fn_val = self.module.add_function(name, fn_type, None);
        // set arguments names
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_float_value().set_name(params[i]);
        }
        fn_val
    }

    fn codegen_function(
        &mut self,
        function: &'ctx ast::Function<'ctx>,
    ) -> Result<FunctionValue<'ctx>, String> {
        let function_name = function.0;
        let params = function.1.iter().collect::<Vec<_>>();
        let function_value = self.protogen(function_name, &params);

        self.formal_table.clear();
        for (param, param_value) in params.into_iter().zip(function_value.get_param_iter()) {
            self.formal_table
                .insert(*param, param_value.into_int_value());
        }

        let entry = self
            .context
            .append_basic_block(function_value, function_name);

        self.builder.position_at_end(entry);

        let body = self.codegen(&*function.2)?;
        self.builder.build_return(Some(&body));

        if !function_value.verify(true) {
            unsafe {
                function_value.delete();
            }
            return Err(format!("Could not verify function {}", function_name));
        }

        // TODO: add function pass manager
        // self.fpm.run_on(&function_value);
        Ok(function_value)
    }
}
