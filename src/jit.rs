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
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub global_table: HashMap<&'ctx str, IntValue<'ctx>>,
    pub formal_table: HashMap<&'ctx str, IntValue<'ctx>>,
    pub fpm: PassManager<FunctionValue<'ctx>>,
    pub execution_engine: ExecutionEngine<'ctx>,
    exec_mode: ExecutionMode,
    functions: Vec<FunctionValue<'ctx>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(unused)]
pub enum ExecutionMode {
    Interpreter,
    JIT,
    Dead,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, exec_mode: ExecutionMode) -> Result<Self, LLVMString> {
        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = match exec_mode {
            ExecutionMode::JIT => module.create_jit_execution_engine(OptimizationLevel::None)?,
            ExecutionMode::Interpreter => module.create_interpreter_execution_engine()?,
            _ => panic!("Cannot create a compiler with dead execution engine"),
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
            exec_mode,
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

    /// Panics if engine is invalid
    fn verify_engine(&mut self) {
        match self.exec_mode {
            ExecutionMode::Interpreter => (),
            ExecutionMode::Dead => panic!(
                "Execution engine is has already been used. \
                JIT engine can only be used once"
            ),
            _ => self.exec_mode = ExecutionMode::Dead,
        };
    }

    #[allow(unused)]
    pub fn top_level_run_one(&mut self, function: &FunctionValue<'ctx>) {
        self.verify_engine();

        let res = unsafe { self.execution_engine.run_function(*function, &[]) };
        println!("{}", res.as_int(true));
    }

    pub fn top_level_run_all(&mut self, top_level_functions: &[FunctionValue<'ctx>]) {
        self.verify_engine();

        for &tl in top_level_functions {
            let res = unsafe { self.execution_engine.run_function(tl, &[]) };
            println!("{}", res.as_int(true));
        }
    }
}
