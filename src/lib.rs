use ast::Ast;
use std::process;

extern crate pest;

#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod compile;
pub mod jit;
pub mod lazygraph;
pub mod parser;
pub mod preprocessor;
pub mod static_analysis;

pub const MAX_MACRO_DEPTH: u32 = 15;

pub mod errors {
    pub static UNBOUND_FUNCTION: &str = "#$@UBF";
    pub static MACRO_LOOP: &str = "#$@RCM";
}

#[allow(unused)]
pub fn print_ast(ast: &Ast) {
    eprintln!("\nPRINTING AST\n------------");
    for node in ast.iter() {
        eprintln!("{:?}", node);
    }
}

#[allow(unused)]
pub fn print_ir(compiler: &jit::Compiler) {
    eprintln!("\nLLVM IR\n--------------------------------------------------");
    compiler.module.print_to_stderr();
}

pub fn rip(e: String) -> ! {
    eprintln!("error: {}", e);
    process::exit(1)
}
