use crate::ast::{self, Expr, Rule};
use pest::iterators::Pair;

pub fn parse_literal(expr: Pair<Rule>) -> Expr {
    let integer_literal = expr.into_inner().next().unwrap();
    Expr::Literal(integer_literal.as_str().to_string())
}

pub fn parse_variable(expr: Pair<Rule>) -> Expr {
    let variable = expr.into_inner().next().unwrap();
    Expr::Identifier(variable.as_str().to_string())
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
