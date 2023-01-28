use crate::ast::{self, ImpcoreValue, Rule};
use crate::environment::Env;
use pest::iterators::{Pair, Pairs};

#[allow(dead_code)]
pub fn eval_file_use(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

#[allow(dead_code)]
pub fn eval_print(func_call: Pair<Rule>, env: &mut Env) {}
