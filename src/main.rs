extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod environment;
mod flow;
mod functions;
mod globals;
mod io;
mod tests;

use ast::{ImpcoreParser, Rule};
use environment::Env;
use pest::Parser;
use std::{fs, process};

const MB: usize = 1 << 20;

fn main() {
    let filename = "../imp/hw1.imp";
    let contents = fs::read_to_string(filename)
        .map_err(|_| {
            eprintln!("Failed to open file {}", filename);
            process::exit(1);
        })
        .unwrap();

    let top_level_pair = ImpcoreParser::parse(Rule::impcore, &contents)
        .map_err(|e| {
            eprintln!("Parsing Error: {}", e);
            process::exit(1);
        })
        .unwrap()
        .next()
        .unwrap();

    let mut environment = Env::new(64 * MB);
    ast::eval_top_level(top_level_pair, &mut environment);
}
