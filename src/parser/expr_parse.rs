use super::*;
use crate::ast::AstScope;

pub fn parse_literal(expr: Pair<Rule>) -> AstExpr {
    let num = expr.as_str().parse().unwrap();
    AstExpr::Literal(num)
}

pub fn parse_variable(expr: Pair<Rule>) -> AstExpr {
    let name = expr.as_str();
    AstExpr::Variable(name, AstScope::Unknown)
}

pub fn parse_binary(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let operator = inner_expr.next().unwrap().as_str();
    let lhs = AstExpr::parse(inner_expr.next().unwrap());
    let rhs = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::Binary(operator, Box::new(lhs), Box::new(rhs))
}

pub fn parse_unary(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let operator = inner_expr.next().unwrap().as_str();
    let arg = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::Unary(operator, Box::new(arg))
}

pub fn parse_call(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let name = inner_expr.next().unwrap().as_str();
    let args = inner_expr.map(AstExpr::parse).collect();
    AstExpr::Call(name, args)
}

pub fn parse_if(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let condition = AstExpr::parse(inner_expr.next().unwrap());
    let true_case = AstExpr::parse(inner_expr.next().unwrap());
    let false_case = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::If(
        Box::new(condition),
        Box::new(true_case),
        Box::new(false_case),
    )
}

pub fn parse_while(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let condition = AstExpr::parse(inner_expr.next().unwrap());
    let body = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::While(Box::new(condition), Box::new(body))
}

pub fn parse_begin(expr: Pair<Rule>) -> AstExpr {
    let inner_expr = expr.into_inner();
    AstExpr::Begin(inner_expr.map(AstExpr::parse).collect())
}

pub fn parse_set(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let name = inner_expr.next().unwrap().as_str();
    let newval = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::Assign(name, Box::new(newval), AstScope::Unknown)
}
