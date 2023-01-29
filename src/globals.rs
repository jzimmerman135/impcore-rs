use crate::ast::{self, AstNode, Rule};
use pest::iterators::Pair;

pub fn parse_literal(expr: Pair<Rule>) -> AstNode {
    AstNode::Literal(expr.as_str())
}

pub fn parse_variable(expr: Pair<Rule>) -> AstNode {
    AstNode::Identifier(expr.as_str())
}

pub fn parse_array(expr: Pair<Rule>) -> AstNode {
    let mut array = expr.into_inner();
    let name = array.next().unwrap().as_str();
    let index = ast::parse_exp(array.next().unwrap());
    AstNode::Indexer(name, index.into())
}

pub fn parse_alloc(def: Pair<Rule>) -> AstNode {
    let mut alloc_def = def.into_inner();
    let name = alloc_def.next().unwrap().as_str();
    let expr = ast::parse_exp(alloc_def.next().unwrap());
    AstNode::Alloc(name.into(), expr.into())
}

pub fn parse_val(def: Pair<Rule>) -> AstNode {
    let mut val_def = def.into_inner();
    let name = val_def.next().unwrap().as_str();
    let expr = ast::parse_exp(val_def.next().unwrap());
    AstNode::NewVar(name, Box::new(expr))
}

pub fn parse_set(expr: Pair<Rule>) -> AstNode {
    let mut set_expr = expr.into_inner();
    let name = set_expr.next().unwrap().as_str();
    let expr = ast::parse_exp(set_expr.next().unwrap());
    AstNode::Assign(name, Box::new(expr))
}
