use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    passes::PassManager,
    support::LLVMString,
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue},
    OptimizationLevel,
};

const ANON: &str = "#anon";

use crate::ast::{self, AstNode};

pub trait CodeGen {
    fn codegen<'ctx>(&self, compiler: &'ctx mut Compiler) -> Result<IntValue<'ctx>, String>;
}

#[derive(Debug)]
#[allow(unused)]
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub global_table: HashMap<&'ctx str, IntValue<'ctx>>,
    pub formal_table: HashMap<&'ctx str, IntValue<'ctx>>,
    pub fpm: PassManager<FunctionValue<'ctx>>,
    pub execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Result<Self, LLVMString> {
        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
        let fpm = PassManager::create(&module);
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.initialize();

        Ok(Self {
            context,
            module,
            fpm,
            builder,
            execution_engine,
            global_table: HashMap::new(),
            formal_table: HashMap::new(),
        })
    }
}

impl<'ctx> Compiler<'ctx> {
    pub fn codegen_anonymous(
        &mut self,
        node: &'ctx AstNode,
    ) -> Result<FunctionValue<'ctx>, String> {
        let fn_type = self.context.i32_type().fn_type(&[], false);
        let function = self.module.add_function("", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "top_level_entry");
        self.builder.position_at_end(basic_block);
        let v = self.codegen(node)?;
        self.builder.build_return(Some(&v));

        if !function.verify(false) {
            return Err(format!("Could not verify top level function"));
        }

        Ok(function)
    }

    #[allow(unused)]
    pub fn top_level_run(&mut self, node: &'ctx AstNode) -> Result<(), String> {
        if let AstNode::Function(inner) = &node {
            let function = self.codegen_function(inner);
            todo!();
        }

        let anon = self.codegen_anonymous(node)?;
        let res = unsafe {
            {
                // self.module.print_to_stderr();
                // self.module.get_functions().for_each(|e| println!("{e:?}"));
            }
            self.execution_engine.run_function(anon, &[]).as_int(true)
        };

        println!("{}", res as i32);

        Ok(())
    }

    fn codegen(&mut self, node: &'ctx AstNode) -> Result<IntValue<'ctx>, String> {
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

    fn make_function_value<'a>(&self, name: &'a str, params: &[&&'a str]) -> FunctionValue<'ctx> {
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
        let function_value = self.make_function_value(function_name, &params);

        self.formal_table.clear();
        for (param, param_value) in params.into_iter().zip(function_value.get_param_iter()) {
            self.formal_table
                .insert(*param, param_value.into_int_value());
        }

        let entry = self
            .context
            .append_basic_block(function_value, function_name);

        self.builder.position_at_end(entry);
        let body = self.codegen(&function.2)?;
        self.builder.build_return(Some(&body));

        if !function_value.verify(false) {
            unsafe {
                function_value.delete();
            }
            return Err(format!("Could not verify function {}", function_name));
        }

        self.fpm.run_on(&function_value);

        Ok(function_value)
    }

    fn codegen_val(&mut self, val: &'ctx ast::NewGlobal) -> Result<IntValue, String> {
        let name = val.0;
        let value = self.codegen(&val.1)?;
        self.global_table.insert(name, value);
        Ok(value)
    }
}
