use std::{collections::HashMap, process};
mod codegen;
mod defgen;
pub(crate) mod implib;
use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{Linkage, Module},
    passes::PassManager,
    support::LLVMString,
    targets::{InitializationConfig, Target},
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue},
    AddressSpace, OptimizationLevel,
};

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub fpm: PassManager<FunctionValue<'ctx>>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub quiet_mode: bool,
    param_table: HashMap<&'ctx str, PointerValue<'ctx>>,
    pub global_table: HashMap<&'ctx str, GlobalValue<'ctx>>,
    pub exec_mode: ExecutionMode,
    curr_function: Option<FunctionValue<'ctx>>,
    pub lib: HashMap<&'ctx str, FunctionValue<'ctx>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Interpreter,
    Jit,
    Dead,
}

#[derive(Debug)]
pub enum NativeTopLevel<'ctx> {
    CheckAssert(FunctionValue<'ctx>, &'ctx str),
    CheckExpect(FunctionValue<'ctx>, FunctionValue<'ctx>, &'ctx str),
    TopLevelExpr(FunctionValue<'ctx>),
    PrintFunctionName(&'ctx str),
    FreeAll(FunctionValue<'ctx>),
    Noop,
}

impl<'a> NativeTopLevel<'a> {
    fn is_test(&self) -> bool {
        matches!(self, Self::CheckAssert(..) | Self::CheckExpect(..))
    }
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, exec_mode: ExecutionMode) -> Result<Self, LLVMString> {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native execution target");

        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let execution_engine = match exec_mode {
            ExecutionMode::Jit => {
                module.create_jit_execution_engine(OptimizationLevel::Aggressive)?
            }
            ExecutionMode::Interpreter => module.create_interpreter_execution_engine()?,
            ExecutionMode::Dead => panic!("Cannot create a compiler with dead execution engine"),
        };

        let fpm = Self::get_optimization_pass_manager(&module);

        let mut compiler = Self {
            context,
            module,
            fpm,
            builder,
            execution_engine,
            quiet_mode: false,
            exec_mode,
            param_table: HashMap::new(),
            global_table: HashMap::new(),
            curr_function: None,
            lib: HashMap::new(),
        };

        compiler.build_lib();
        Ok(compiler)
    }

    pub fn native_interpret_one(&mut self, native: &NativeTopLevel<'ctx>) -> Result<(), String> {
        self.verify_engine();
        unsafe { self.run_native_unverified(native) }
    }

    pub fn native_run_all(&mut self, native_top_level_exprs: &[NativeTopLevel<'ctx>]) {
        self.verify_engine();
        let mut successful = 0;
        let mut fail_messages = vec![];
        for native in native_top_level_exprs {
            let success = unsafe { self.run_native_unverified(native) };
            if native.is_test() {
                match success {
                    Ok(_) => successful += 1,
                    Err(reason) => fail_messages.push(reason),
                }
            }
        }
        summarize_tests(successful, &fail_messages, self.quiet_mode);
    }

    fn build_lib(&mut self) {
        let addr_space = AddressSpace::default();
        let int_type = self.context.i32_type();
        let str_type = self.context.i8_type().ptr_type(addr_space);

        let printf_type = int_type.fn_type(&[str_type.into()], true);
        let printf_fn = self
            .module
            .add_function("printf", printf_type, Some(Linkage::External));
        self.lib.insert("__printf", printf_fn);

        self.add_print_functions();
    }

    /// Does not check if name is actually bound
    /// This should be a band aid for a more sophisticated type system
    fn is_pointer(&self, name: &str) -> bool {
        name.ends_with('[')
    }

    fn get_optimization_pass_manager(module: &Module<'ctx>) -> PassManager<FunctionValue<'ctx>> {
        let fpm = PassManager::create(module);
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_tail_call_elimination_pass();
        fpm.initialize();
        fpm
    }

    fn clear_curr_function(&mut self) {
        self.param_table.clear();
        self.curr_function = None;
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

    /// unsafe because will segfault if in Jit exec mode and the module has been modified since running.
    unsafe fn run_native_unverified(
        &mut self,
        top_level_def: &NativeTopLevel<'ctx>,
    ) -> Result<(), String> {
        match *top_level_def {
            NativeTopLevel::PrintFunctionName(name) if !self.quiet_mode => println!("{}", name),
            NativeTopLevel::TopLevelExpr(fn_value) => unsafe {
                self.execution_engine.run_function(fn_value, &[]);
            },
            NativeTopLevel::CheckAssert(assert_fn, contents) => {
                let res =
                    unsafe { self.execution_engine.run_function(assert_fn, &[]) }.as_int(true);
                if res == 0 {
                    return Err(format!("Failed test ({}) -> assertion false", contents));
                }
            }
            NativeTopLevel::CheckExpect(lhs, rhs, contents) => {
                let lhs = unsafe { self.execution_engine.run_function(lhs, &[]) }.as_int(true);
                let rhs = unsafe { self.execution_engine.run_function(rhs, &[]) }.as_int(true);
                if lhs != rhs {
                    return Err(format!(
                        "Failed test ({}) -> got \'{}\' and expected \'{}\'",
                        contents, lhs, rhs
                    ));
                }
            }
            NativeTopLevel::FreeAll(fn_value) => {
                let cleanup_code = unsafe { self.execution_engine.run_function(fn_value, &[]) };
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

fn summarize_tests(successful: usize, fail_messages: &[String], quiet_mode: bool) {
    let n_tests = successful + fail_messages.len();
    if n_tests != 0 {
        for reason in fail_messages {
            eprintln!("{}", reason);
        }
        match fail_messages.len() == 0 {
            true => {
                if !quiet_mode {
                    eprintln!("All {} tests successful", successful)
                }
            }
            false => eprintln!("Passed {} of {} tests", successful, n_tests),
        }
    }
}
