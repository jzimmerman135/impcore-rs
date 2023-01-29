use crate::ast::{self, AstNode, Rule};
use pest::iterators::{Pair, Pairs};

#[allow(dead_code)]
pub fn parse_file_use(_pairs: Pairs<Rule>) -> AstNode {
    todo!()
}

pub fn parse_print(expr: Pair<Rule>) -> AstNode {
    let mut print_expr = expr.into_inner();
    let operator = print_expr.next().unwrap().as_str();
    let arg = ast::parse_exp(print_expr.next().unwrap());
    AstNode::Print(operator.into(), arg.into())
}
