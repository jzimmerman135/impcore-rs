use crate::ast::Rule;
use crate::environment::Env;
use pest::iterators::Pairs;

pub struct TestExpr {}

pub fn eval_assert(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
pub fn eval_error(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
pub fn eval_expect(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
