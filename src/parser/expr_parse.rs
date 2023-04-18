use super::*;
use std::i32;

impl<'a> AstExpr<'a> {
    pub fn parse(expr: Pair<Rule>) -> AstExpr {
        match expr.as_rule() {
            Rule::literal => expr_parse::parse_literal(expr),
            Rule::variable => expr_parse::parse_variable(expr),

            Rule::binary => expr_parse::parse_binary(expr),
            Rule::unary => expr_parse::parse_unary(expr),
            Rule::print => expr_parse::parse_unary(expr),
            Rule::user => expr_parse::parse_call(expr),

            Rule::ifx => expr_parse::parse_if(expr),
            Rule::whilex => expr_parse::parse_while(expr),
            Rule::begin => expr_parse::parse_begin(expr),
            Rule::matchx => expr_parse::parse_match(expr),
            Rule::set => expr_parse::parse_set(expr),

            Rule::array_value => expr_parse::parse_indexer(expr),
            Rule::pointer => expr_parse::parse_pointer(expr),

            Rule::fgetc => AstExpr::Call(expr.as_str(), vec![]),

            Rule::macroval => macro_parse::parse_macroval(expr),
            Rule::parameter => macro_parse::parse_inlinerparam(expr),
            Rule::inline => expr_parse::parse_call(expr),

            Rule::error => AstExpr::Error,
            _ => unreachable!("got unreachable expr rule {:?}", expr.as_rule()),
        }
    }
}

pub fn parse_literal(expr: Pair<Rule>) -> AstExpr {
    let numstr = expr.as_str();
    let number = if let Some(hexadecimal_str) = numstr.strip_prefix("0x") {
        u32::from_str_radix(hexadecimal_str, 16).unwrap() as i32
    } else {
        numstr.parse().unwrap()
    };
    AstExpr::Literal(number)
}

pub fn parse_variable(expr: Pair<Rule>) -> AstExpr {
    let name = expr.as_str();
    if name == "_" {
        return AstExpr::Literal(0);
    }
    AstExpr::Variable(name, None)
}

pub fn parse_pointer(expr: Pair<Rule>) -> AstExpr {
    let name = expr.as_str();
    AstExpr::Pointer(name)
}

pub fn parse_indexer(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let name = inner_expr.next().unwrap().as_str();
    let index = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::Variable(name, Some(Box::new(index)))
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
    let name = inner_expr.next().unwrap();
    if let Rule::array_value = name.as_rule() {
        let mut array = name.into_inner();
        let name = array.next().unwrap().as_str();
        let index = AstExpr::parse(array.next().unwrap());
        let value = AstExpr::parse(inner_expr.next().unwrap());
        return AstExpr::Assign(name, Box::new(value), Some(Box::new(index)));
    }

    let name = name.as_str();
    let newval = AstExpr::parse(inner_expr.next().unwrap());
    AstExpr::Assign(name, Box::new(newval), None)
}

enum MatchArm<'a> {
    Case((AstExpr<'a>, AstExpr<'a>)),
    Default(AstExpr<'a>),
}

pub fn parse_match(expr: Pair<Rule>) -> AstExpr {
    let mut inner_expr = expr.into_inner();
    let scrutinee = AstExpr::parse(inner_expr.next().unwrap());
    let mut default = AstExpr::default();
    let arms = inner_expr
        .filter_map(|expr| match parse_match_arm(expr) {
            MatchArm::Case(arm) => Some(arm),
            MatchArm::Default(then) => {
                default = then;
                None
            }
        })
        .collect::<Vec<_>>();
    AstExpr::Match(Box::new(scrutinee), arms, Box::new(default))
}

fn parse_match_arm(expr: Pair<Rule>) -> MatchArm {
    let mut inner_expr = expr.into_inner();
    let case = AstExpr::parse(inner_expr.next().unwrap());
    if let Some(expr) = inner_expr.next() {
        let then = AstExpr::parse(expr);
        MatchArm::Case((case, then))
    } else {
        MatchArm::Default(case)
    }
}
