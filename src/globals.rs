use crate::ast::{Expr, Rule};
use pest::iterators::Pair;

pub fn parse_literal(integer_literal: Pair<Rule>) -> Expr {
    Expr::Literal(integer_literal.as_str().to_string())
}
