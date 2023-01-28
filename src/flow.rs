use crate::ast::{Expr, Rule};
use pest::iterators::Pair;

pub fn parse_ifx(expr: Pair<Rule>) -> Expr {
    let _ifx = expr.into_inner();
    todo!()
}
