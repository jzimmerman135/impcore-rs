use crate::ast::{self, Expr, Rule};
use pest::iterators::{Pair, Pairs};

#[allow(dead_code)]
pub fn parse_file_use(_pairs: Pairs<Rule>) -> Expr {
    todo!()
}

pub fn parse_print(expr: Pair<Rule>) -> Expr {
    let arg = ast::parse_exp(expr.into_inner().next().unwrap());
    Expr::Print(Box::new(arg))
}
