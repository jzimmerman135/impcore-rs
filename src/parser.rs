use pest::iterators::Pair;

use crate::ast::{
    Assign, AstNode, Begin, Binary, Call, Function, If, Literal, NewGlobal, RuntimeError, Unary,
    Variable, While,
};

// the all powerful build step parser
#[derive(Parser)]
#[grammar = "grammar/llvm-style.pest"]
pub struct ImpcoreParser;

pub trait InnerParse {
    fn parse(expr: Pair<Rule>) -> AstNode;
}

impl<'a> InnerParse for AstNode<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        match expr.as_rule() {
            Rule::literal => Literal::parse(expr),
            Rule::variable => Variable::parse(expr),
            Rule::binary => Binary::parse(expr),
            Rule::unary => Unary::parse(expr),
            Rule::user => Call::parse(expr),
            Rule::define => Function::parse(expr),
            Rule::ifx => If::parse(expr),
            Rule::whilex => While::parse(expr),
            Rule::begin => Begin::parse(expr),
            Rule::set => Assign::parse(expr),
            Rule::val => NewGlobal::parse(expr),
            Rule::error => AstNode::Error(RuntimeError),
            _ => unreachable!(
                "Failed to recognize rule {:?} in {:?}",
                expr.as_rule(),
                expr.as_str()
            ),
        }
    }
}

impl InnerParse for Literal {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let lit_exp = Literal(expr.as_str().parse().unwrap());
        AstNode::Literal(lit_exp)
    }
}

impl<'a> InnerParse for Variable<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let var_exp = Variable(expr.as_str());
        AstNode::Variable(var_exp)
    }
}

impl<'a> InnerParse for Binary<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let binary_operator = expr.next().unwrap().as_str();
        let lhs = Box::new(AstNode::parse(expr.next().unwrap()));
        let rhs = Box::new(AstNode::parse(expr.next().unwrap()));
        let bin_expr = Binary(binary_operator, lhs, rhs);
        AstNode::Binary(bin_expr)
    }
}

impl<'a> InnerParse for Unary<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let unary_operator = expr.next().unwrap().as_str();
        let arg = Box::new(AstNode::parse(expr.next().unwrap()));
        let unary_expr = Unary(unary_operator, arg);
        AstNode::Unary(unary_expr)
    }
}

impl<'a> InnerParse for Call<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let function_name = expr.next().unwrap().as_str();
        let args = expr.map(AstNode::parse).collect();
        let call_expr = Call(function_name, args);
        AstNode::Call(call_expr)
    }
}

impl<'a> InnerParse for Function<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let function_name = expr.next().unwrap().as_str();
        let (param_exprs, body_expr): (Vec<_>, Vec<_>) =
            expr.partition(|e| e.as_rule() == Rule::parameter);
        let body = Box::new(AstNode::parse(body_expr.into_iter().next().unwrap()));
        let function_expr = Function(
            function_name,
            param_exprs.iter().map(|e| e.as_str()).collect(),
            body,
        );
        AstNode::Function(function_expr)
    }
}

impl<'a> InnerParse for If<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let condition = Box::new(AstNode::parse(expr.next().unwrap()));
        let true_expr = Box::new(AstNode::parse(expr.next().unwrap()));
        let false_expr = Box::new(AstNode::parse(expr.next().unwrap()));
        let if_expr = If(condition, true_expr, false_expr);
        AstNode::If(if_expr)
    }
}

impl<'a> InnerParse for While<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let condition = Box::new(AstNode::parse(expr.next().unwrap()));
        let body = Box::new(AstNode::parse(expr.next().unwrap()));
        let while_expr = While(condition, body);
        AstNode::While(while_expr)
    }
}

impl<'a> InnerParse for Begin<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let begin_expr = Begin(expr.into_inner().map(AstNode::parse).collect());
        AstNode::Begin(begin_expr)
    }
}

impl<'a> InnerParse for Assign<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let variable_name = expr.next().unwrap().as_str();
        let arg = Box::new(AstNode::parse(expr.next().unwrap()));
        let set_expr = Assign(variable_name, arg);
        AstNode::Assign(set_expr)
    }
}

impl<'a> InnerParse for NewGlobal<'a> {
    fn parse(expr: Pair<Rule>) -> AstNode {
        let mut expr = expr.into_inner();
        let variable_name = expr.next().unwrap().as_str();
        let arg = Box::new(AstNode::parse(expr.next().unwrap()));
        let val_expr = NewGlobal(variable_name, arg);
        AstNode::NewGlobal(val_expr)
    }
}
