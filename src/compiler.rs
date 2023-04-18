use std::process;

use colored::Colorize;
use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    passes::PassManager,
    targets::{InitializationConfig, Target},
    types::IntType,
    values::FunctionValue,
    AddressSpace, OptimizationLevel,
};

use crate::{
    ast::Def,
    env::{Name, Tokens},
};

pub enum ExecutionMode {
    Jit,
    Interpreter,
    Dead,
}

#[derive(Clone)]
pub enum NativeFunction<'ctx> {
    DeclareFunction(Name),
    TopLevelExp(FunctionValue<'ctx>),
    CheckAssert(FunctionValue<'ctx>, Def),
    CheckExpect(FunctionValue<'ctx>, FunctionValue<'ctx>, Def),
    FreeAll(FunctionValue<'ctx>),
    NoOp,
}

#[allow(unused)]
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub optimizer: PassManager<FunctionValue<'ctx>>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub execution_mode: ExecutionMode,
    pub quiet_mode: bool,
    pub itype: IntType<'ctx>,
    pub addr_spc: AddressSpace,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, execution_mode: ExecutionMode) -> Result<Self, String> {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native execution target");

        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = match execution_mode {
            ExecutionMode::Jit => module.create_jit_execution_engine(OptimizationLevel::Aggressive),
            ExecutionMode::Interpreter => module.create_interpreter_execution_engine(),
            ExecutionMode::Dead => {
                panic!("Cannot create a compiler with dead execution engine")
            }
        }
        .expect("Failed to generate execution engine");

        let optimizer = Self::create_optimization_pass_manager(&module);

        Ok(Self {
            context,
            module,
            optimizer,
            builder,
            execution_engine,
            execution_mode,
            quiet_mode: false,
            itype: context.i32_type(),
            addr_spc: AddressSpace::default(),
        })
    }

    pub fn native_interpret_one(
        &mut self,
        native: &NativeFunction<'ctx>,
        tokens: &Tokens,
    ) -> Result<(), String> {
        self.verify_engine();
        unsafe { self.run_native_unverified(native, tokens) }
    }

    pub fn native_run_all(
        &mut self,
        native_top_level_exprs: &[NativeFunction<'ctx>],
        tokens: &Tokens,
    ) {
        self.verify_engine();
        let mut successful = 0;
        let mut fail_messages = vec![];
        for native in native_top_level_exprs {
            let success = unsafe { self.run_native_unverified(native, tokens) };
            if native.is_test() {
                match success {
                    Ok(_) => successful += 1,
                    Err(reason) => fail_messages.push(reason),
                }
            }
        }
        summarize_tests(successful, &fail_messages, self.quiet_mode);
    }

    fn create_optimization_pass_manager(module: &Module<'ctx>) -> PassManager<FunctionValue<'ctx>> {
        let fpm = PassManager::create(module);
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_tail_call_elimination_pass();
        fpm.add_jump_threading_pass();
        fpm.initialize();
        fpm
    }

    /// Panics if engine is invalid
    fn verify_engine(&mut self) {
        match self.execution_mode {
            ExecutionMode::Interpreter => (),
            ExecutionMode::Dead => panic!(
                "Execution engine is has already been used. \
                JIT engine can only be used once"
            ),
            _ => self.execution_mode = ExecutionMode::Dead,
        };
    }

    /// unsafe because will segfault if in Jit exec mode and the module has been modified since running.
    unsafe fn run_native_unverified(
        &mut self,
        top_level_def: &NativeFunction<'ctx>,
        tokens: &Tokens,
    ) -> Result<(), String> {
        match top_level_def {
            NativeFunction::DeclareFunction(name) if !self.quiet_mode => {
                println!("{}", tokens.translate(name))
            }
            NativeFunction::TopLevelExp(fn_value) => unsafe {
                self.execution_engine.run_function(*fn_value, &[]);
            },
            NativeFunction::CheckAssert(assert_fn, contents) => {
                let res =
                    unsafe { self.execution_engine.run_function(*assert_fn, &[]) }.as_int(true);
                if res == 0 {
                    return Err(format!(
                        "Failed test {} -> assertion false",
                        contents.to_string(tokens)
                    ));
                }
            }
            NativeFunction::CheckExpect(lhs, rhs, contents) => {
                let lhs = unsafe { self.execution_engine.run_function(*lhs, &[]) }.as_int(true);
                let rhs = unsafe { self.execution_engine.run_function(*rhs, &[]) }.as_int(true);
                if lhs != rhs {
                    return Err(format!(
                        "{} {}, got \'{}\' and expected \'{}\'",
                        "Failed test".red().bold(),
                        contents.to_string(tokens),
                        lhs,
                        rhs
                    ));
                }
            }
            NativeFunction::FreeAll(fn_value) => {
                let cleanup_code = unsafe { self.execution_engine.run_function(*fn_value, &[]) };
                if cleanup_code.as_int(true) == 1 {
                    eprintln!("FATAL ERROR: failed to free memory. exiting with code 1");
                    process::exit(1);
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl<'ctx> NativeFunction<'ctx> {
    pub fn is_test(&self) -> bool {
        matches!(
            self,
            NativeFunction::CheckAssert(..) | NativeFunction::CheckExpect(..)
        )
    }
}

fn summarize_tests(successful: usize, fail_messages: &[String], quiet_mode: bool) {
    let n_tests = successful + fail_messages.len();
    if n_tests != 0 {
        for reason in fail_messages {
            eprintln!("{}", reason);
        }
        match fail_messages.is_empty() {
            true => {
                if !quiet_mode {
                    eprintln!("All {} tests passed", successful)
                }
            }
            false => eprintln!("Passed {} of {} tests", successful, n_tests),
        }
    }
}
