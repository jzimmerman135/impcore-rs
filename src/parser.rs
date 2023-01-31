use crate::ast::AstNode;
use pest::iterators::Pair;
#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct ImpcoreParser;

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
        Rule::define => parse_define(def),
        Rule::val => parse_val(def),
        Rule::alloc => parse_alloc(def),
        Rule::check_assert => parse_assert(def),
        Rule::check_expect => parse_expect(def),
        Rule::check_error => parse_is_error(def),
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
        Rule::literal => parse_literal(expr),
        Rule::variable => parse_variable(expr),
        Rule::array_value => parse_array(expr),
        Rule::binary => parse_binary(expr),
        Rule::unary => parse_unary(expr),
        Rule::user => parse_user(expr),
        Rule::ifx => parse_ifx(expr),
        Rule::set => parse_set(expr),
        Rule::whilex => parse_whilex(expr),
        Rule::begin => parse_begin(expr),
        Rule::print => parse_print(expr),
        Rule::error => AstNode::Error,
        _ => unreachable!(
            "found exp rule: {:?} with body {:?}",
            expr.as_rule(),
            expr.as_str()
        ),
    }
}

fn parse_ifx(expr: Pair<Rule>) -> AstNode {
    let mut ifx = expr.into_inner();
    let condition = Box::new(parse_exp(ifx.next().unwrap()));
    let true_case = Box::new(parse_exp(ifx.next().unwrap()));
    let false_case = Box::new(parse_exp(ifx.next().unwrap()));
    return AstNode::If(condition, true_case, false_case);
}

fn parse_whilex(expr: Pair<Rule>) -> AstNode {
    let mut whilex = expr.into_inner();
    let condition = Box::new(parse_exp(whilex.next().unwrap()));
    let body = Box::new(parse_exp(whilex.next().unwrap()));
    return AstNode::While(condition, body);
}

fn parse_begin(expr: Pair<Rule>) -> AstNode {
    let begin = expr.into_inner();
    let expressions = begin.map(|exp| parse_exp(exp)).collect();
    return AstNode::Begin(expressions);
}

fn parse_define(def: Pair<Rule>) -> AstNode {
    let mut func_def = def.into_inner();
    let function_name = func_def.next().unwrap().as_str();
    let mut params = vec![];
    while let Some(def) = func_def.next() {
        if def.as_rule() == Rule::parameter {
            params.push(def.as_str())
        } else {
            let body = parse_exp(def);
            return AstNode::Prototype(function_name, params, Box::new(body));
        }
    }
    unreachable!()
}

fn parse_binary(expr: Pair<Rule>) -> AstNode {
    let mut binary_func = expr.into_inner();
    let binary_operator = binary_func.next().unwrap();
    let expr1 = binary_func.next().unwrap();
    let expr2 = binary_func.next().unwrap();
    let u = Box::new(parse_exp(expr1));
    let v = Box::new(parse_exp(expr2));
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

fn parse_unary(expr: Pair<Rule>) -> AstNode {
    let mut unary_func = expr.into_inner();
    let unary_operator = unary_func.next().unwrap();
    let expr = unary_func.next().unwrap();
    let v = Box::new(parse_exp(expr));
    match unary_operator.as_str() {
        "++" => AstNode::Incr(v),
        "--" => AstNode::Decr(v),
        "!" | "not" => AstNode::Not(v),
        _ => unreachable!(),
    }
}

fn parse_user(expr: Pair<Rule>) -> AstNode {
    let mut func_call = expr.into_inner();
    let func_name = func_call.next().unwrap().as_str();
    let args = func_call.map(parse_exp).collect();
    AstNode::Call(func_name, args)
}

fn parse_literal(expr: Pair<Rule>) -> AstNode {
    AstNode::Literal(expr.as_str())
}

fn parse_variable(expr: Pair<Rule>) -> AstNode {
    AstNode::GlobalVar(expr.as_str())
}

fn parse_array(expr: Pair<Rule>) -> AstNode {
    let mut array = expr.into_inner();
    let name = array.next().unwrap().as_str();
    let index = parse_exp(array.next().unwrap());
    AstNode::GlobalArray(name, index.into())
}

fn parse_alloc(def: Pair<Rule>) -> AstNode {
    let mut alloc_def = def.into_inner();
    let name = alloc_def.next().unwrap().as_str();
    let expr = parse_exp(alloc_def.next().unwrap());
    AstNode::NewArray(name.into(), expr.into())
}

fn parse_val(def: Pair<Rule>) -> AstNode {
    let mut val_def = def.into_inner();
    let name = val_def.next().unwrap().as_str();
    let expr = parse_exp(val_def.next().unwrap());
    AstNode::NewVar(name, Box::new(expr))
}

fn parse_set(expr: Pair<Rule>) -> AstNode {
    let mut set_expr = expr.into_inner();
    let name = set_expr.next().unwrap().as_str();
    let expr = parse_exp(set_expr.next().unwrap());
    AstNode::Assign(name, Box::new(expr))
}

fn parse_print(expr: Pair<Rule>) -> AstNode {
    let mut print_expr = expr.into_inner();
    let operator = print_expr.next().unwrap().as_str();
    let arg = parse_exp(print_expr.next().unwrap());
    AstNode::Print(operator.into(), arg.into())
}

fn parse_assert(expr: Pair<Rule>) -> AstNode {
    let assert_expr = parse_exp(expr.into_inner().next().unwrap());
    AstNode::Test(Box::new(assert_expr))
}

fn parse_is_error(expr: Pair<Rule>) -> AstNode {
    let error_expr = parse_exp(expr.into_inner().next().unwrap());
    AstNode::Test(Box::new(error_expr))
}

fn parse_expect(expr: Pair<Rule>) -> AstNode {
    let mut expect_test = expr.into_inner();
    let lhs = parse_exp(expect_test.next().unwrap());
    let rhs = parse_exp(expect_test.next().unwrap());
    let equal = AstNode::Eq(Box::new(lhs), Box::new(rhs));
    AstNode::Test(Box::new(equal))
}
