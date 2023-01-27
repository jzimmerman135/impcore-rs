use crate::ast::Rule;
use crate::environment::Env;
use pest::iterators::Pairs;

pub struct TestExpr {}

pub fn eval_assert(_pairs: Pairs<Rule>, _env: &mut Env) {}
pub fn eval_error(_pairs: Pairs<Rule>, _env: &mut Env) {}
pub fn eval_expect(_pairs: Pairs<Rule>, _env: &mut Env) {}
