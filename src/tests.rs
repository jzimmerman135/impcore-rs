use crate::ast::{self, AstNode, Rule};
use pest::iterators::Pair;

pub fn parse_assert(expr: Pair<Rule>) -> AstNode {
    let assert_expr = ast::parse_exp(expr.into_inner().next().unwrap());
    AstNode::Test(Box::new(assert_expr))
}

pub fn parse_is_error(expr: Pair<Rule>) -> AstNode {
    let error_expr = ast::parse_exp(expr.into_inner().next().unwrap());
    AstNode::Test(Box::new(error_expr))
}

pub fn parse_expect(expr: Pair<Rule>) -> AstNode {
    let mut expect_test = expr.into_inner();
    let lhs = ast::parse_exp(expect_test.next().unwrap());
    let rhs = ast::parse_exp(expect_test.next().unwrap());
    let equal = AstNode::Eq(Box::new(lhs), Box::new(rhs));
    AstNode::Test(Box::new(equal))
}
