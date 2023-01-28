extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod flow;
mod functions;
mod globals;
mod io;
mod jit;
mod tests;

use ast::{ImpcoreParser, Rule};
use pest::Parser;
use std::{fs, process};

fn main() {
    let filename = "./imp/ez.imp";
    let contents = fs::read_to_string(filename)
        .map_err(|_| {
            eprintln!("Failed to open file {}", filename);
            process::exit(1);
        })
        .unwrap();

    let top_level_expressions = ImpcoreParser::parse(Rule::impcore, &contents)
        .map_err(|e| {
            eprintln!("Parsing Error: {}", e);
            process::exit(1);
        })
        .unwrap()
        .next()
        .unwrap()
        .into_inner()
        .filter_map(ast::parse_top_level)
        .collect();

    let mut compiler = jit::JIT::default();

    let _code = compiler
        .compile(top_level_expressions)
        .map_err(|e| {
            eprintln!("Parsing Error: {}", e);
            process::exit(1);
        })
        .unwrap();
}
