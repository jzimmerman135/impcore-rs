use std::collections::HashMap;
mod codegen;
mod defgen;

pub use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    passes::PassManager,
    support::LLVMString,
    types::BasicMetadataTypeEnum,
    values::{AsValueRef, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue},
    OptimizationLevel,
};

use crate::ast::{self, AstNode};

pub trait CodeGen {
    fn codegen<'ctx>(&'ctx self, compiler: &'ctx mut Compiler) -> Result<IntValue<'ctx>, String>;
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
    functions: Vec<FunctionValue<'ctx>>,
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
            functions: vec![],
        })
    }
}

impl<'ctx> Compiler<'ctx> {
    #[allow(unused)]
    pub fn top_level_run(&mut self, node: &'ctx AstNode) -> Result<(), String> {
        if let AstNode::Function(inner) = &node {
            let function = self.defgen_function(inner)?;
            self.functions.push(function);
            return Ok(println!("{}", inner.0));
        }

        self.module = self.context.create_module("tmp");

        let anon = self.defgen_anonymous(node)?;

        self.execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|_| "Failed to make execution engine")?;

        let res = unsafe { self.execution_engine.run_function(anon, &[]) };

        Ok(println!("{:?}", res))
    }
}
