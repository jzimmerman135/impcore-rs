use crate::{environment::Env, flow, functions, globals, io, tests};
use pest::iterators::Pair;

pub type ImpcoreValue = i32;

#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct ImpcoreParser;

pub fn eval_top_level(pairs: Pair<Rule>, env: &mut Env) {
    for top_level_expression in pairs.into_inner() {
        match top_level_expression.as_rule() {
            Rule::def => {
                println!("{}", eval_def(top_level_expression, env));
            }
            Rule::exp => {
                println!("{}", eval_exp(top_level_expression, env));
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }
}

fn eval_def(pair: Pair<Rule>, _env: &mut Env) -> String {
    match pair.as_rule() {
        // Rule::define => functions::eval_define(pair.into_inner(), env),
        // Rule::val => globals::eval_val(pair.into_inner(), env),
        // Rule::alloc => globals::eval_alloc(pair.into_inner(), env),
        // Rule::file_use => io::eval_file_use(pair.into_inner(), env),
        // Rule::check_assert => tests::eval_assert(pair.into_inner(), env),
        // Rule::check_expect => tests::eval_assert(pair.into_inner(), env),
        // Rule::check_error => tests::eval_assert(pair.into_inner(), env),
        _ => unreachable!(),
    }
}

pub fn eval_exp(pair: Pair<Rule>, env: &mut Env) -> ImpcoreValue {
    let expr = pair.into_inner().next().unwrap();
    match expr.as_rule() {
        Rule::function_call => functions::eval_function(expr, env),
        Rule::control_flow => flow::eval_control_flow(expr, env),
        Rule::integer_literal => globals::eval_literal(expr),
        // Rule::accessor => globals::eval_accessor(pair.into_inner(), env),
        _ => unreachable!(),
    }
}
