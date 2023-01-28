use crate::ast::{Expr, Rule};
use pest::iterators::Pairs;

#[allow(dead_code)]
pub fn parse_assert(_pairs: Pairs<Rule>) -> Expr {
    todo!()
}

#[allow(dead_code)]
pub fn parse_error(_pairs: Pairs<Rule>) -> Expr {
    todo!()
}

#[allow(dead_code)]
pub fn parse_expect(_pairs: Pairs<Rule>) -> Expr {
    todo!()
}
