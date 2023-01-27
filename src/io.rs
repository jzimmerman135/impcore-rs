use crate::environment::Env;
use crate::parser::Rule;
use pest::iterators::Pairs;

pub fn eval_file_use(_pairs: Pairs<Rule>, _env: &mut Env) {}
pub fn eval_print(_pairs: Pairs<Rule>, _env: &mut Env) {}
