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
mod translation;

use ast::{AstNode, ImpcoreParser, Rule};
use pest::Parser;
use std::{fs, process};

fn main() {
    let filename = "./imp/hw1.imp";
    let contents = fs::read_to_string(filename)
        .map_err(|_| {
            eprintln!("Failed to open file {}", filename);
            process::exit(1);
        })
        .unwrap();

    let top_level_expressions: Vec<AstNode> = ImpcoreParser::parse(Rule::impcore, &contents)
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

    for tle in top_level_expressions {
        println!("{:?}", tle);
    }
}
