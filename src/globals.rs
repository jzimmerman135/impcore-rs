use crate::ast::{Expr, ImpcoreValue, Rule};
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

pub fn parse_literal(integer_literal: Pair<Rule>) -> Expr {
    Expr::Literal(integer_literal.as_str().to_string())
}
