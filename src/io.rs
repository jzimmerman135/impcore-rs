use crate::ast::{Expr, Rule};
use pest::iterators::{Pair, Pairs};

#[allow(dead_code)]
pub fn parse_file_use(_pairs: Pairs<Rule>) -> Expr {
    todo!()
}

#[allow(dead_code)]
pub fn parse_print(_func_call: Pair<Rule>) -> Expr {
    todo!();
}
