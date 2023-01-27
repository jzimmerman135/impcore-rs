use crate::environment::Env;
use crate::parser::Rule;
use pest::iterators::Pairs;
pub struct Function {}

pub fn eval_define(_pairs: Pairs<Rule>, _env: &mut Env) {}

pub fn eval_function(pairs: Pairs<Rule>, env: &mut Env) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::binary => eval_binary(pair.into_inner(), env),
            Rule::unary => eval_unary(pair.into_inner(), env),
            Rule::user => eval_user(pair.into_inner(), env),
            _ => unreachable!(),
        }
    }
}

fn eval_binary(_pairs: Pairs<Rule>, _env: &mut Env) {}
fn eval_unary(_pairs: Pairs<Rule>, _env: &mut Env) {}
fn eval_user(_pairs: Pairs<Rule>, _env: &mut Env) {}
