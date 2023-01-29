use crate::ast::{self, Expr, Rule};
use pest::iterators::Pair;

pub fn parse_define(def: Pair<Rule>) -> Expr {
    let mut func_def = def.into_inner();
    let function_name = func_def.next().unwrap().as_str().to_string();
    let mut params = vec![];
    while let Some(def) = func_def.next() {
        if def.as_rule() == Rule::parameter {
            params.push(Expr::Identifier(def.as_str().to_string()))
        } else {
            let body = ast::parse_exp(def);
            return Expr::Definition(function_name, params, Box::new(body));
        }
    }
    unreachable!()
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
        "++" => Expr::Incr(v),
        "--" => Expr::Decr(v),
        "!" | "not" => Expr::Not(v),
        _ => unreachable!(),
    }
}

pub fn parse_user(expr: Pair<Rule>) -> Expr {
    let mut func_call = expr.into_inner();
    let func_name = func_call.next().unwrap().as_str().to_string();
    let args = func_call.map(ast::parse_exp).collect();
    Expr::Call(func_name, args)
}
