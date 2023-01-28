use crate::ast::{ImpcoreValue, Rule};
use crate::environment::Env;
use pest::iterators::{Pair, Pairs};

pub struct Globals {}
#[allow(dead_code)]
impl Globals {
    pub fn new() -> Self {
        Self {}
    }
    fn lookup_variable(&self, _name: &str) -> ImpcoreValue {
        todo!()
    }
    fn lookup_array(&self, _name: &str, _index: ImpcoreValue) -> ImpcoreValue {
        todo!()
    }
    fn set_variable(&mut self, _name: &str, _val: ImpcoreValue) {
        todo!()
    }
    fn set_array(&mut self, _name: &str, _index: ImpcoreValue, _val: ImpcoreValue) {
        todo!()
    }
}

#[allow(dead_code)]
pub fn eval_set(_pairs: Pairs<Rule>, _env: &mut Env) -> ImpcoreValue {
    todo!()
}

#[allow(dead_code)]
pub fn eval_accessor(mut _pairs: Pairs<Rule>, _env: &mut Env) -> ImpcoreValue {
    todo!()
}

#[allow(dead_code)]
pub fn eval_val(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

#[allow(dead_code)]
pub fn eval_alloc(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

pub fn eval_literal(integer_literal: Pair<Rule>) -> ImpcoreValue {
    integer_literal.as_str().parse().unwrap()
}
