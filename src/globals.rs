use crate::ast::{self, Expr, Rule};
use pest::iterators::Pair;

pub fn parse_literal(expr: Pair<Rule>) -> Expr {
    Expr::Literal(expr.as_str().to_string())
}

pub fn parse_variable(expr: Pair<Rule>) -> Expr {
    Expr::Identifier(expr.as_str().to_string())
}

pub fn parse_array(expr: Pair<Rule>) -> Expr {
    let mut array = expr.into_inner();
    let name = array.next().unwrap().as_str().to_string();
    let index = ast::parse_exp(array.next().unwrap());
    Expr::Indexer(name, index.into())
}

pub fn parse_alloc(def: Pair<Rule>) -> Expr {
    let mut alloc_def = def.into_inner();
    let name = alloc_def.next().unwrap().as_str();
    let expr = ast::parse_exp(alloc_def.next().unwrap());
    Expr::Alloc(name.into(), expr.into())
}

pub fn parse_val(def: Pair<Rule>) -> Expr {
    let mut val_def = def.into_inner();
    let name = val_def.next().unwrap().as_str().to_string();
    let expr = ast::parse_exp(val_def.next().unwrap());
    Expr::NewVar(name, Box::new(expr))
}

pub fn parse_set(expr: Pair<Rule>) -> Expr {
    let mut set_expr = expr.into_inner();
    let name = set_expr.next().unwrap().as_str().to_string();
    let expr = ast::parse_exp(set_expr.next().unwrap());
    Expr::Assign(name, Box::new(expr))
}
