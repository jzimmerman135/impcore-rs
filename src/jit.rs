use std::collections::HashMap;
mod codegen;
mod defgen;

use inkwell::targets::{InitializationConfig, Target};
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
    curr_function: Option<FunctionValue<'ctx>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(unused)]
pub enum ExecutionMode {
    Interpreter,
    Jit,
    Dead,
}

#[derive(Copy, Clone, Debug)]
pub enum TopLevelExpr<'ctx> {
    ExprDef(FunctionValue<'ctx>),
    FuncDef(&'ctx str),
    TestAssertDef(FunctionValue<'ctx>),
    TestExpectDef(FunctionValue<'ctx>, FunctionValue<'ctx>),
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, exec_mode: ExecutionMode) -> Result<Self, LLVMString> {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native execution target");

        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = match exec_mode {
            ExecutionMode::Jit => module.create_jit_execution_engine(OptimizationLevel::None)?,
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
            curr_function: None,
        })
    }
}

impl<'ctx> Compiler<'ctx> {
    #[allow(unused)]
    pub fn top_level_compile(&mut self, node: &'ctx AstNode) -> Result<TopLevelExpr<'ctx>, String> {
        Ok(match node {
            AstNode::Function(inner) => {
                let function = self.defgen_function(inner)?;
                TopLevelExpr::FuncDef(inner.0)
            }
            AstNode::CheckAssert(inner) => {
                TopLevelExpr::TestAssertDef(self.defgen_check_assert(inner)?)
            }
            AstNode::CheckExpect(inner) => {
                let (lhs, rhs) = self.defgen_check_expect(inner)?;
                TopLevelExpr::TestExpectDef(lhs, rhs)
            }
            _ => TopLevelExpr::ExprDef(self.defgen_anonymous(node)?),
        })
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

    fn run_tle_unverified(&mut self, tle: &TopLevelExpr<'ctx>) {
        match tle {
            TopLevelExpr::FuncDef(name) => println!("{}", name),
            TopLevelExpr::ExprDef(f) => {
                let res = unsafe { self.execution_engine.run_function(*f, &[]) };
                println!("{}", res.as_int(true))
            }
            _ => unreachable!("not a top level expression {:?}", tle),
        }
    }

    #[allow(unused)]
    pub fn top_level_run_one(&mut self, tle: &TopLevelExpr<'ctx>) {
        self.verify_engine();
        self.run_tle_unverified(tle)
    }

    pub fn top_level_run_all(&mut self, top_level_exprs: &[TopLevelExpr<'ctx>]) {
        self.verify_engine();

        for &tle in top_level_exprs {
            self.run_tle_unverified(&tle)
        }
    }

    fn run_test_unverified(&mut self, test: &TopLevelExpr<'ctx>) -> bool {
        match test {
            TopLevelExpr::TestAssertDef(assert_fn) => {
                let res =
                    unsafe { self.execution_engine.run_function(*assert_fn, &[]) }.as_int(true);
                if res == 0 {
                    eprintln!("check-assert failed got: {}", res);
                    return false;
                }
                true
            }
            TopLevelExpr::TestExpectDef(lhs, rhs) => {
                let lhs = unsafe { self.execution_engine.run_function(*lhs, &[]) }.as_int(true);
                let rhs = unsafe { self.execution_engine.run_function(*rhs, &[]) }.as_int(true);
                if lhs != rhs {
                    eprint!("check-expect failed got {} and {}", lhs, rhs);
                    return false;
                }
                true
            }
            _ => unreachable!("not a test expression {:?}", test),
        }
    }

    #[allow(unused)]
    pub fn test_run_one(&mut self, test: &TopLevelExpr<'ctx>) {
        self.verify_engine();
        self.run_test_unverified(test);
    }

    #[allow(unused)]
    pub fn test_run_all(&mut self, top_level_tests: &[TopLevelExpr<'ctx>]) {
        self.verify_engine();
        let mut successful = 0;
        for test in top_level_tests {
            if self.run_test_unverified(test) {
                successful += 1
            }
        }
        if successful == top_level_tests.len() {
            eprint!("All {} tests successful", successful)
        } else {
            eprintln!("Passed {} of {} tests", successful, top_level_tests.len())
        }
    }
}
