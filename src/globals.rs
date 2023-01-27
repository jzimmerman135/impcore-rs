use crate::ast::Rule;
use crate::environment::Env;
use pest::iterators::Pairs;

pub struct Global {}

pub fn eval_set(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
pub fn eval_val(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
