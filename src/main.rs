extern crate pest;
#[macro_use]
extern crate pest_derive;

mod environment;
mod functions;
mod parser;

use std::process;

use environment::Env;
use parser::{ImpcoreParser, Rule};
use pest::{iterators::Pairs, Parser};

fn eval_file_use(_pairs: Pairs<Rule>) {
    eprintln!("FILE USED");
}

fn eval_val(_pairs: Pairs<Rule>) {}

fn eval_function(pairs: Pairs<Rule>) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::binary => functions::eval_binary(pair.into_inner()),
            Rule::unary => functions::eval_unary(pair.into_inner()),
            Rule::user => functions::eval_user(pair.into_inner()),
            _ => unreachable!(),
        }
    }
}

fn eval_def(pairs: Pairs<Rule>, _env: &mut Env) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::define => functions::eval_define(pair.into_inner()),
            Rule::val => eval_val(pair.into_inner()),
            Rule::check_assert => eval_val(pair.into_inner()),
            Rule::check_error => eval_val(pair.into_inner()),
            Rule::check_expect => eval_val(pair.into_inner()),
            Rule::usex => eval_file_use(pair.into_inner()),
            _ => unreachable!(),
        }
    }
}

fn eval_exp(pairs: Pairs<Rule>, _env: &mut Env) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::error => eval_function(pair.into_inner()),
            Rule::set => eval_function(pair.into_inner()),
            Rule::ifx => eval_function(pair.into_inner()),
            Rule::whilex => eval_function(pair.into_inner()),
            Rule::begin => eval_function(pair.into_inner()),
            Rule::function => eval_function(pair.into_inner()),
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
