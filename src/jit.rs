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

use crate::ast::{self, AstNode, Function};

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

pub enum ExecutionMode {
    Interpreter,
    JIT,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, mode: ExecutionMode) -> Result<Self, LLVMString> {
        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = match mode {
            ExecutionMode::JIT => module.create_jit_execution_engine(OptimizationLevel::None)?,
            ExecutionMode::Interpreter => module.create_interpreter_execution_engine()?,
        };
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
    pub fn top_level_compile(
        &mut self,
        node: &'ctx AstNode,
    ) -> Result<Option<FunctionValue<'ctx>>, String> {
        if let AstNode::Function(inner) = &node {
            let function = self.defgen_function(inner)?;
            self.functions.push(function);
            println!("{}", inner.0);
            return Ok(None);
        }

        let anon = self.defgen_anonymous(node)?;
        Ok(Some(anon))
    }

    // Use if you expect to add functions to the module later
    pub fn top_level_run_one(&mut self, function: &FunctionValue<'ctx>) {
        let res = unsafe { self.execution_engine.run_function(*function, &[]) };
        println!("{}", res.as_int(true));
    }

    // Use only if the module is finalized
    pub fn top_level_run_all(&mut self, top_level_functions: &[FunctionValue<'ctx>]) {
        for &tl in top_level_functions {
            let res = unsafe { self.execution_engine.run_function(tl, &[]) };
            println!("{}", res.as_int(true));
        }
    }
}
