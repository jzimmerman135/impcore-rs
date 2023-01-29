use crate::{flow, functions, globals};
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct ImpcoreParser;

#[allow(unused)]
pub enum Expr {
    Definition(String, Vec<Expr>, Box<Expr>),
    NewVar(String, Box<Expr>),

    Literal(String),
    Identifier(String),
    GlobalDataAddr(String),
    Call(String, Box<Expr>),
    Assign(String, Box<Expr>),

    Inc(Box<Expr>),
    Dec(Box<Expr>),
    Not(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),

    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),

    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    ShiftLeft(Box<Expr>, Box<Expr>),
    ShiftRight(Box<Expr>, Box<Expr>),

    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    Begin(Vec<Expr>),
    Print(Box<Expr>),
    Printu(Box<Expr>),
    Error,
}

pub fn parse_top_level(top_level_expression: Pair<Rule>) -> Option<Expr> {
    match top_level_expression.as_rule() {
        Rule::def => Some(parse_def(top_level_expression)),
        Rule::exp => Some(parse_exp(top_level_expression)),
        Rule::EOI => None,
        _ => unreachable!(),
    }
}

fn parse_def(def: Pair<Rule>) -> Expr {
    match def.as_rule() {
        Rule::define => functions::parse_define(def),
        Rule::val => globals::parse_val(def),
        // Rule::alloc => globals::eval_alloc(pair.into_inner(), env),
        // Rule::check_assert => tests::eval_assert(pair.into_inner(), env),
        // Rule::check_expect => tests::eval_assert(pair.into_inner(), env),
        // Rule::check_error => tests::eval_assert(pair.into_inner(), env),
        _ => unreachable!(),
    }
}

pub fn parse_exp(pair: Pair<Rule>) -> Expr {
    let expr = pair.into_inner().next().unwrap();
    match expr.as_rule() {
        Rule::integer_literal => globals::parse_literal(expr),
        Rule::variable => globals::parse_variable(expr),
        Rule::binary => functions::parse_binary(expr),
        Rule::unary => functions::parse_unary(expr),
        Rule::ifx => flow::parse_ifx(expr),
        Rule::set => globals::parse_set(expr),
        Rule::whilex => flow::parse_whilex(expr),
        Rule::begin => flow::parse_begin(expr),
        Rule::error => Expr::Error,
        _ => unreachable!(),
    }
}
