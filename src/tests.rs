use crate::ast::Rule;
use crate::environment::Env;
use pest::iterators::Pairs;

pub struct TestExpr {}

#[allow(dead_code)]
pub fn eval_assert(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

#[allow(dead_code)]
pub fn eval_error(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

#[allow(dead_code)]
pub fn eval_expect(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}
