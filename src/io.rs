use crate::ast::Rule;
use crate::environment::Env;
use pest::iterators::Pairs;

pub fn eval_file_use(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}

#[allow(dead_code)]
pub fn eval_print(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}

pub fn eval_error(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
