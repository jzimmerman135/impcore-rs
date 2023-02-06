extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod jit;
mod parser;

use ast::AstNode;
use parser::ImpcoreParser;
use std::{fs, process};

fn print_ast(ast: &[AstNode]) {
    println!("\nPRINTING AST\n------------");
    for node in ast.iter() {
        println!("{:?}", node);
    }
}

fn print_ir(compiler: &jit::Compiler) {
    println!("\nLLVM IR\n--------------------------------------------------");
    compiler.module.print_to_stderr();
}

fn die(e: String) -> ! {
    eprintln!("error: {}", e);
    process::exit(1)
}

fn main() {
    let filename = "./imp/hw1.imp";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| die(format!("dailed to open file {}", filename)));

    let top_level_nodes: Vec<AstNode> =
        ImpcoreParser::generate_top_level_exprs(&contents).unwrap_or_else(|s| die(s));

    print_ast(&top_level_nodes);

    let context = inkwell::context::Context::create();
    let mut compiler =
        jit::Compiler::new(&context, jit::ExecutionMode::Jit).expect("Failed to build compiler");

    let tles = top_level_nodes
        .iter()
        .map(|e| compiler.top_level_compile(e))
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            process::exit(1)
        });

    print_ir(&compiler);

    println!("\nEXECUTION OUTPUT\n--------------------------------------------------");
    compiler.top_level_run_all(&tles);
}
