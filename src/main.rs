extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod jit;
mod translation;

use ast::{AstNode, ImpcoreParser, Rule};
use pest::Parser;
use std::{fs, process};

fn main() {
    let filename = "./imp/hard.imp";
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

    let mut tests = vec![];

    for tle in top_level_expressions {
        println!("{:?}", tle);
        match tle {
            AstNode::Test(test_expression) => tests.push(test_expression),
            AstNode::Literal(value) => println!("{}", value.parse::<u32>().unwrap()),
            AstNode::Definition(name, ..) => {
                println!("{}", name);
            }
            expr @ _ => println!("{:?}", expr),
        }
    }

    for test in tests {
        println!("TEST 0 != {:?}", test);
    }
}
