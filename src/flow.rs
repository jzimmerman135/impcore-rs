use crate::ast::{ImpcoreValue, Rule};
use crate::environment::Env;
use pest::iterators::{Pair, Pairs};

pub fn eval_control_flow(pair: Pair<Rule>, env: &mut Env) -> ImpcoreValue {
    match pair.as_rule() {
        Rule::ifx => eval_ifx(pair.into_inner(), env),
        Rule::whilex => eval_whilex(pair.into_inner(), env),
        Rule::begin => eval_begin(pair.into_inner(), env),
        Rule::error => eval_error(env),
        _ => unreachable!(),
    }
}

fn eval_ifx(_pairs: Pairs<Rule>, _env: &mut Env) -> ImpcoreValue {
    todo!()
}

fn eval_whilex(_pairs: Pairs<Rule>, _env: &mut Env) -> ImpcoreValue {
    todo!()
}

fn eval_begin(_pairs: Pairs<Rule>, _env: &mut Env) -> ImpcoreValue {
    todo!()
}

fn eval_error(_env: &mut Env) -> ImpcoreValue {
    todo!()
}
