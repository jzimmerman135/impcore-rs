use crate::ast::{self, parse_exp, Expr, Rule};
use pest::iterators::Pair;

pub fn parse_define(def: Pair<Rule>) -> Expr {
    let mut func_def = def.into_inner();
    let function_name = func_def.next().unwrap().as_str().to_string();
    let mut params = vec![];
    let mut body = Expr::Error;
    while let Some(expr) = func_def.next() {
        match expr.as_rule() {
            Rule::parameter => params.push(Expr::Identifier(expr.as_str().to_string())),
            Rule::exp => body = parse_exp(expr),
            _ => unreachable!(),
        }
    }
    Expr::Definition(function_name, params, Box::new(body))
}

pub fn parse_binary(expr: Pair<Rule>) -> Expr {
    let mut binary_func = expr.into_inner();
    let binary_operator = binary_func.next().unwrap();
    let expr1 = binary_func.next().unwrap();
    let expr2 = binary_func.next().unwrap();
    let u = Box::new(ast::parse_exp(expr1));
    let v = Box::new(ast::parse_exp(expr2));
    match binary_operator.as_str() {
        "*" => Expr::Mul(u, v),
        "/" => Expr::Div(u, v),
        "+" => Expr::Add(u, v),
        "-" => Expr::Sub(u, v),
        "%" => Expr::Mod(u, v),
        "=" => Expr::Eq(u, v),
        "&&" => Expr::And(u, v),
        "||" => Expr::Or(u, v),
        ">" => Expr::Gt(u, v),
        "<" => Expr::Lt(u, v),
        ">=" => Expr::Ge(u, v),
        "<=" => Expr::Le(u, v),
        ">>" => Expr::ShiftRight(u, v),
        "<<" => Expr::ShiftLeft(u, v),
        "^" => Expr::Xor(u, v),
        "&" => Expr::BitAnd(u, v),
        "|" => Expr::BitOr(u, v),
        _ => unreachable!(),
    }
}

pub fn parse_unary(expr: Pair<Rule>) -> Expr {
    let mut unary_func = expr.into_inner();
    let unary_operator = unary_func.next().unwrap();
    let expr = unary_func.next().unwrap();
    let v = Box::new(ast::parse_exp(expr));
    match unary_operator.as_str() {
        "++" => Expr::Inc(v),
        "--" => Expr::Dec(v),
        "!" | "not" => Expr::Not(v),
        _ => unreachable!(),
    }
}
