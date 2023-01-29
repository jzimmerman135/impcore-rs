use crate::ast::{self, Expr, Rule};
use pest::iterators::Pair;

pub fn parse_assert(expr: Pair<Rule>) -> Expr {
    let assert_expr = ast::parse_exp(expr.into_inner().next().unwrap());
    Expr::Test(Box::new(assert_expr))
}

pub fn parse_is_error(expr: Pair<Rule>) -> Expr {
    let error_expr = ast::parse_exp(expr.into_inner().next().unwrap());
    Expr::Test(Box::new(error_expr))
}

pub fn parse_expect(expr: Pair<Rule>) -> Expr {
    let mut expect_test = expr.into_inner();
    let lhs = ast::parse_exp(expect_test.next().unwrap());
    let rhs = ast::parse_exp(expect_test.next().unwrap());
    let equal = Expr::Eq(Box::new(lhs), Box::new(rhs));
    Expr::Test(Box::new(equal))
}
