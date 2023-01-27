extern crate pest;
#[macro_use]
extern crate pest_derive;

mod environment;
mod functions;
mod globals;
mod io;
mod parser;

use std::process;

use environment::Env;
use parser::{ImpcoreParser, Rule};
use pest::{iterators::Pairs, Parser};

fn eval(_pairs: Pairs<Rule>, _env: &mut Env) {}

pub fn eval_def(pairs: Pairs<Rule>, env: &mut Env) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::define => functions::eval_define(pair.into_inner(), env),
            Rule::val => globals::eval_val(pair.into_inner(), env),
            Rule::check_assert => eval(pair.into_inner(), env),
            Rule::check_error => eval(pair.into_inner(), env),
            Rule::check_expect => eval(pair.into_inner(), env),
            Rule::file_use => io::eval_file_use(pair.into_inner(), env),
            _ => unreachable!(),
        }
    }
}

pub fn eval_exp(pairs: Pairs<Rule>, env: &mut Env) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::error => eval(pair.into_inner(), env),
            Rule::set => globals::eval_set(pair.into_inner(), env),
            Rule::ifx => eval(pair.into_inner(), env),
            Rule::whilex => eval(pair.into_inner(), env),
            Rule::begin => eval(pair.into_inner(), env),
            Rule::function => functions::eval_function(pair.into_inner(), env),
            _ => unreachable!(),
        }
    }
}

const MB: usize = 1 << 20;

fn main() {
    let pairs = ImpcoreParser::parse(Rule::impcore, include_str!("../hw1.imp"))
        .map_err(|e| {
            eprintln!("Parsing Error: {}", e);
            process::exit(1);
        })
        .unwrap()
        .next()
        .unwrap();

    let mut env = Env::new(64 * MB);

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::def => eval_def(pair.into_inner(), &mut env),
            Rule::exp => eval_exp(pair.into_inner(), &mut env),
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }
}
