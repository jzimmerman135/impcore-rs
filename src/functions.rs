use crate::ast::{self, AstNode, Rule};
use pest::iterators::Pair;

pub fn parse_define(def: Pair<Rule>) -> AstNode {
    let mut func_def = def.into_inner();
    let function_name = func_def.next().unwrap().as_str();
    let mut params = vec![];
    while let Some(def) = func_def.next() {
        if def.as_rule() == Rule::parameter {
            params.push(AstNode::Identifier(def.as_str()))
        } else {
            let body = ast::parse_exp(def);
            return AstNode::Definition(function_name, params, Box::new(body));
        }
    }
    unreachable!()
}

pub fn parse_binary(expr: Pair<Rule>) -> AstNode {
    let mut binary_func = expr.into_inner();
    let binary_operator = binary_func.next().unwrap();
    let expr1 = binary_func.next().unwrap();
    let expr2 = binary_func.next().unwrap();
    let u = Box::new(ast::parse_exp(expr1));
    let v = Box::new(ast::parse_exp(expr2));
    match binary_operator.as_str() {
        "*" => AstNode::Mul(u, v),
        "/" => AstNode::Div(u, v),
        "+" => AstNode::Add(u, v),
        "-" => AstNode::Sub(u, v),
        "%" => AstNode::Mod(u, v),
        "=" => AstNode::Eq(u, v),
        "&&" => AstNode::And(u, v),
        "||" => AstNode::Or(u, v),
        ">" => AstNode::Gt(u, v),
        "<" => AstNode::Lt(u, v),
        ">=" => AstNode::Ge(u, v),
        "<=" => AstNode::Le(u, v),
        ">>" => AstNode::ShiftRight(u, v),
        "<<" => AstNode::ShiftLeft(u, v),
        "^" => AstNode::Xor(u, v),
        "&" => AstNode::BitAnd(u, v),
        "|" => AstNode::BitOr(u, v),
        _ => unreachable!(),
    }
}

pub fn parse_unary(expr: Pair<Rule>) -> AstNode {
    let mut unary_func = expr.into_inner();
    let unary_operator = unary_func.next().unwrap();
    let expr = unary_func.next().unwrap();
    let v = Box::new(ast::parse_exp(expr));
    match unary_operator.as_str() {
        "++" => AstNode::Incr(v),
        "--" => AstNode::Decr(v),
        "!" | "not" => AstNode::Not(v),
        _ => unreachable!(),
    }
}

pub fn parse_user(expr: Pair<Rule>) -> AstNode {
    let mut func_call = expr.into_inner();
    let func_name = func_call.next().unwrap().as_str();
    let args = func_call.map(ast::parse_exp).collect();
    AstNode::Call(func_name, args)
}
