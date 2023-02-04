extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod jit;
mod parser;

use ast::AstNode;
use parser::ImpcoreParser;
use std::{error, fs};

fn main() -> Result<(), Box<dyn error::Error>> {
    let filename = "./imp/ez.imp";
    let contents =
        fs::read_to_string(filename).map_err(|_| format!("Failed to open file {}", filename))?;

    let top_level_expressions: Vec<AstNode> = ImpcoreParser::generate_ast(&contents)?;

    println!("PRINTING AST\n------------");
    for tle in top_level_expressions.iter() {
        println!("{:?}", tle);
    }
    println!("--------------------------------------------------\n\n");

    let context = inkwell::context::Context::create();
    let mut compiler = jit::Compiler::new(&context).expect("Failed to build compiler");

    for tle in top_level_expressions.iter() {
        compiler.top_level_run(&tle)?;
    }

    Ok(())
}
