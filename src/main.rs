extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod jit;
mod parser;

use ast::AstNode;
use parser::ImpcoreParser;
use std::{error, fs, process};

fn main() -> Result<(), Box<dyn error::Error>> {
    let filename = "./imp/ez.imp";
    let contents =
        fs::read_to_string(filename).map_err(|_| format!("Failed to open file {}", filename))?;

    let top_level_expressions: Vec<AstNode> = ImpcoreParser::generate_top_level_exprs(&contents)?;

    println!("\nPRINTING AST\n------------");
    for tle in top_level_expressions.iter() {
        println!("{:?}", tle);
    }

    let context = inkwell::context::Context::create();
    let mut compiler =
        jit::Compiler::new(&context, jit::ExecutionMode::Jit).expect("Failed to build compiler");

    println!("\nLLVM IR\n--------------------------------------------------");
    let tlfs = top_level_expressions
        .iter()
        .map(|e| compiler.top_level_compile(e))
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            process::exit(1)
        });
    compiler.module.print_to_stderr();

    println!("\nEXECUTION OUTPUT\n--------------------------------------------------");

    compiler.top_level_run_all(&tlfs);

    Ok(())
}
