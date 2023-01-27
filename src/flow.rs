use crate::ast::Rule;
use crate::environment::Env;
use pest::iterators::Pairs;

pub fn eval_ifx(_pairs: Pairs<Rule>, _env: &mut Env) {}
pub fn eval_begin(_pairs: Pairs<Rule>, _env: &mut Env) {}
pub fn eval_whilex(_pairs: Pairs<Rule>, _env: &mut Env) {}
