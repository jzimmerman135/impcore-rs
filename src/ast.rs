use crate::{flow, functions, globals, io, tests};
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct ImpcoreParser;

#[derive(Debug)]
pub enum AstNode<'a> {
    Definition(&'a str, Vec<AstNode<'a>>, Box<AstNode<'a>>),
    NewVar(&'a str, Box<AstNode<'a>>),

    Literal(&'a str),
    Identifier(&'a str),
    Indexer(&'a str, Box<AstNode<'a>>),
    Alloc(&'a str, Box<AstNode<'a>>),
    Call(&'a str, Vec<AstNode<'a>>),
    Assign(&'a str, Box<AstNode<'a>>),

    Incr(Box<AstNode<'a>>),
    Decr(Box<AstNode<'a>>),
    Not(Box<AstNode<'a>>),

    Eq(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Lt(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Le(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Gt(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Ge(Box<AstNode<'a>>, Box<AstNode<'a>>),

    And(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Or(Box<AstNode<'a>>, Box<AstNode<'a>>),

    Add(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Sub(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Mul(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Div(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Mod(Box<AstNode<'a>>, Box<AstNode<'a>>),

    BitAnd(Box<AstNode<'a>>, Box<AstNode<'a>>),
    BitOr(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Xor(Box<AstNode<'a>>, Box<AstNode<'a>>),
    ShiftLeft(Box<AstNode<'a>>, Box<AstNode<'a>>),
    ShiftRight(Box<AstNode<'a>>, Box<AstNode<'a>>),

    If(Box<AstNode<'a>>, Box<AstNode<'a>>, Box<AstNode<'a>>),
    While(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Begin(Vec<AstNode<'a>>),
    Print(&'a str, Box<AstNode<'a>>),

    Test(Box<AstNode<'a>>),
    Error,
}

pub fn parse_top_level(top_level_expression: Pair<Rule>) -> Option<AstNode> {
    match top_level_expression.as_rule() {
        Rule::def => Some(parse_def(top_level_expression)),
        Rule::exp => Some(parse_exp(top_level_expression)),
        Rule::EOI => None,
        _ => unreachable!(),
    }
}

fn parse_def(pair: Pair<Rule>) -> AstNode {
    let def = pair.into_inner().next().unwrap();
    match def.as_rule() {
        Rule::define => functions::parse_define(def),
        Rule::val => globals::parse_val(def),
        Rule::alloc => globals::parse_alloc(def),
        Rule::check_assert => tests::parse_assert(def),
        Rule::check_expect => tests::parse_expect(def),
        Rule::check_error => tests::parse_is_error(def),
        _ => unreachable!(
            "found def rule: {:?} with body {:?}",
            def.as_rule(),
            def.as_str()
        ),
    }
}

pub fn parse_exp(pair: Pair<Rule>) -> AstNode {
    let expr = pair.into_inner().next().unwrap();
    match expr.as_rule() {
        Rule::literal => globals::parse_literal(expr),
        Rule::variable => globals::parse_variable(expr),
        Rule::array_value => globals::parse_array(expr),
        Rule::binary => functions::parse_binary(expr),
        Rule::unary => functions::parse_unary(expr),
        Rule::user => functions::parse_user(expr),
        Rule::ifx => flow::parse_ifx(expr),
        Rule::set => globals::parse_set(expr),
        Rule::whilex => flow::parse_whilex(expr),
        Rule::begin => flow::parse_begin(expr),
        Rule::print => io::parse_print(expr),
        Rule::error => AstNode::Error,
        _ => unreachable!(
            "found exp rule: {:?} with body {:?}",
            expr.as_rule(),
            expr.as_str()
        ),
    }
}
