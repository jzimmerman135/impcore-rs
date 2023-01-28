use crate::ast::{self, Expr, Rule};
use crate::environment::Env;
use pest::iterators::{Pair, Pairs};

#[allow(dead_code)]
pub fn eval_define(_pairs: Pairs<Rule>) {
    todo!()
}

pub fn parse_binary(func_call: Pair<Rule>) -> Expr {
    let mut binary_func = func_call.into_inner();
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
        ">" => Expr::Gt(u, v),
        "<" => Expr::Lt(u, v),
        ">=" => Expr::Ge(u, v),
        "<=" => Expr::Le(u, v),
        ">>" => unimplemented!(),
        "<<" => unimplemented!(),
        "&" => unimplemented!(),
        "|" => unimplemented!(),
        "^" => unimplemented!(),
        "&&" => unimplemented!(),
        "||" => unimplemented!(),
        _ => unreachable!(),
    }
}

pub fn parse_unary(func_call: Pair<Rule>) -> Expr {
    let mut unary_func = func_call.into_inner();
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
