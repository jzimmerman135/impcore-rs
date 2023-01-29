use crate::ast::{parse_exp, AstNode, Rule};
use pest::iterators::Pair;

pub fn parse_ifx(expr: Pair<Rule>) -> AstNode {
    let mut ifx = expr.into_inner();
    let condition = Box::new(parse_exp(ifx.next().unwrap()));
    let true_case = Box::new(parse_exp(ifx.next().unwrap()));
    let false_case = Box::new(parse_exp(ifx.next().unwrap()));
    return AstNode::If(condition, true_case, false_case);
}

pub fn parse_whilex(expr: Pair<Rule>) -> AstNode {
    let mut whilex = expr.into_inner();
    let condition = Box::new(parse_exp(whilex.next().unwrap()));
    let body = Box::new(parse_exp(whilex.next().unwrap()));
    return AstNode::While(condition, body);
}

pub fn parse_begin(expr: Pair<Rule>) -> AstNode {
    let begin = expr.into_inner();
    let expressions = begin.map(|exp| parse_exp(exp)).collect();
    return AstNode::Begin(expressions);
}
