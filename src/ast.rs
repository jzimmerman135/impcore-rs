use crate::{environment::Env, flow, functions, globals, io, tests};
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct ImpcoreParser;

pub fn eval_top_level(pairs: Pair<Rule>, env: &mut Env) {
    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::def => eval_def(pair.into_inner(), env),
            Rule::exp => eval_exp(pair.into_inner(), env),
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }
}

fn eval_def(pair: Pairs<Rule>, env: &mut Env) {
    match pair.as_rule() {
        Rule::define => functions::eval_define(pair.into_inner(), env),
        Rule::check_assert => tests::eval_assert(pair.into_inner(), env),
        Rule::check_error => tests::eval_error(pair.into_inner(), env),
        Rule::check_expect => tests::eval_expect(pair.into_inner(), env),
        Rule::file_use => io::eval_file_use(pair.into_inner(), env),
        _ => unreachable!(),
    }
}

fn eval_exp(pairs: Pairs<Rule>, env: &mut Env) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::error => io::eval_error(pair.into_inner(), env),
            Rule::val => globals::eval_val(pair.into_inner(), env),
            Rule::set => globals::eval_set(pair.into_inner(), env),
            Rule::ifx => flow::eval_ifx(pair.into_inner(), env),
            Rule::whilex => flow::eval_whilex(pair.into_inner(), env),
            Rule::begin => flow::eval_begin(pair.into_inner(), env),
            Rule::function => functions::eval_function(pair.into_inner(), env),
            _ => unreachable!(),
        }
    }
}
