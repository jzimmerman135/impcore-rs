#[allow(unused_imports)]
use crate::{
    ast::{Ast, AstDef},
    jit::{self, Compiler, ExecutionMode, NativeTopLevel},
    lazygraph::LazyGraph,
    parser::ImpcoreParser,
};
#[allow(unused_imports)]
use inkwell::context::{self, Context};

impl<'a> Compiler<'a> {
    /// performs lazy compilation of ast into native functions
    pub fn compile(&mut self, ast: &'a Ast) -> Result<Vec<NativeTopLevel<'a>>, String> {
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
        let garbage_collector = NativeTopLevel::FreeAll(jit::implib::build_cleanup(self)?);
        native_functions.push(garbage_collector);
        Ok(native_functions)
    }

    pub fn interpret(&mut self, ast: &'a Ast) -> Result<(), String> {
        assert!(self.exec_mode == ExecutionMode::Interpreter);
        let mut lazy_table = LazyGraph::new();
        for def in ast.defs.iter() {
            let mut native_defs = vec![];
            if let AstDef::Function(name, ..) = &def {
                native_defs.push(NativeTopLevel::PrintFunctionName(name));
            }
            let ready_defs = lazy_table.eval(def, self);
            native_defs.append(
                &mut ready_defs
                    .into_iter()
                    .map(|ready| ready.defgen(self).map_err(|s| lazy_table.why_cant(s)))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            for native in native_defs {
                self.native_interpret_one(&native)?;
            }
        }
        Ok(())
    }
}

// pub fn read_eval_print_loop(contents: &str, context: Context) -> Result<(), String> {
//     let mut asts = vec![];
//     let mut compiler = Compiler::new(&context, ExecutionMode::Interpreter).unwrap();
//     loop {
//         let ast = ImpcoreParser::interpret_ast(contents)?;
//         asts.push(ast);
//         {
//             let ast = asts.last().unwrap();
//             compiler.interpret(ast)?;
//         }
//     }
// }
