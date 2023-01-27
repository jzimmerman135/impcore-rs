use crate::parser::Rule;
use pest::iterators::Pairs;

pub fn eval_define(_pairs: Pairs<Rule>) {}
pub fn eval_binary(_pairs: Pairs<Rule>) {}
pub fn eval_unary(_pairs: Pairs<Rule>) {}
pub fn eval_user(_pairs: Pairs<Rule>) {}
