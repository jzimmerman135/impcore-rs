use std::collections::HashMap;
mod codegen;
mod defgen;
mod implib;
use crate::{
    ast::{Ast, AstDef},
    lazygraph::LazyGraph,
};
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
    exec_mode: ExecutionMode,
    curr_function: Option<FunctionValue<'ctx>>,
    pub lib: HashMap<&'ctx str, FunctionValue<'ctx>>,
}

#[derive(Debug)]
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

    /// performs lazy compilation of ast into native functions
    pub fn compile(&mut self, ast: &'ctx Ast) -> Result<Vec<NativeTopLevel<'ctx>>, String> {
        let mut native_functions = Vec::with_capacity(ast.defs.len());
        let mut lazy_table = LazyGraph::new();
        for def in ast.defs.iter() {
            if let AstDef::Function(name, ..) = &def {
                native_functions.push(NativeTopLevel::PrintFunctionName(name));
            }
            let ready_defs = lazy_table.eval(def, self);
            for def in ready_defs {
                let native_top_level = def.defgen(self).map_err(|s| lazy_table.why_cant(s))?;
                native_functions.push(native_top_level);
            }
        }
        let garbage_collector = NativeTopLevel::FreeAll(implib::build_cleanup(self)?);
        native_functions.push(garbage_collector);
        Ok(native_functions)
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
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
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
    unsafe fn run_native_unverified(&mut self, top_level_def: &NativeTopLevel<'ctx>) {
        match *top_level_def {
            NativeTopLevel::PrintFunctionName(name) if !self.quiet_mode => println!("{}", name),
            NativeTopLevel::TopLevelExpr(fn_value) => unsafe {
                self.execution_engine.run_function(fn_value, &[]);
            },
            NativeTopLevel::FreeAll(fn_value) => {
                let cleanup_code = unsafe { self.execution_engine.run_function(fn_value, &[]) };
                if cleanup_code.as_int(true) == 1 {
                    eprintln!("ERROR: failed to free memory exiting with code 1",)
                }
            }
            _ => {}
        }
    }

    /// unsafe because will segfault if in Jit exec mode and the module has been modified since running.
    unsafe fn run_test_unverified(&mut self, test: &NativeTopLevel<'ctx>) -> bool {
        match *test {
            NativeTopLevel::CheckAssert(assert_fn, contents) => {
                let res =
                    unsafe { self.execution_engine.run_function(assert_fn, &[]) }.as_int(true);
                if res == 0 {
                    eprintln!("Failed test ({}) -> assertion false", contents);
                    return false;
                }
                true
            }
            NativeTopLevel::CheckExpect(lhs, rhs, contents) => {
                let lhs = unsafe { self.execution_engine.run_function(lhs, &[]) }.as_int(true);
                let rhs = unsafe { self.execution_engine.run_function(rhs, &[]) }.as_int(true);
                if lhs != rhs {
                    eprintln!(
                        "Failed test ({}) -> got \'{}\' and expected \'{}\'",
                        contents, lhs, rhs
                    );
                    return false;
                }
                true
            }
            _ => unreachable!("not a test expression {:?}", test),
        }
    }

    pub fn native_run_one(&mut self, def: &NativeTopLevel<'ctx>) {
        self.verify_engine();
        unsafe { self.run_native_unverified(def) };
    }

    pub fn native_run_all(&mut self, native_top_level_exprs: &[NativeTopLevel<'ctx>]) {
        self.verify_engine();
        let mut defs = vec![];
        let mut tests = vec![];

        for native_top_level in native_top_level_exprs {
            if native_top_level.is_test() {
                tests.push(native_top_level);
            } else {
                defs.push(native_top_level);
            }
        }

        for def in defs {
            unsafe { self.run_native_unverified(def) };
        }

        self.run_tests(&tests);
    }

    fn run_tests(&mut self, tests: &[&NativeTopLevel<'ctx>]) {
        if tests.is_empty() {
            return;
        }

        let mut successful = 0;
        for test in tests {
            if unsafe { self.run_test_unverified(test) } {
                successful += 1
            }
        }

        if successful == tests.len() {
            eprintln!("All {} tests successful", successful)
        } else {
            eprintln!("Passed {} of {} tests", successful, tests.len())
        }
    }
}
