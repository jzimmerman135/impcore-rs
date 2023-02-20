use crate::ast::AstType;

use super::*;

pub fn parse_val(def: Pair<Rule>) -> AstDef {
    let mut def = def.into_inner();
    let name = def.next().unwrap().as_str();
    let value = AstExpr::parse(def.next().unwrap());
    AstDef::Global(name, value, AstType::Integer)
}

pub fn parse_alloc(def: Pair<Rule>) -> AstDef {
    let mut def = def.into_inner();
    let name = def.next().unwrap().as_str();
    let size = AstExpr::parse(def.next().unwrap());
    AstDef::Global(name, size, AstType::Pointer)
}

pub fn parse_define(def: Pair<Rule>) -> AstDef {
    let mut inner_expr = def.into_inner();
    let name = inner_expr.next().unwrap().as_str();
    let (param_exprs, body_expr): (Vec<_>, Vec<_>) =
        inner_expr.partition(|e| e.as_rule() == Rule::parameter);

    let mut params = vec![];
    for param in param_exprs {
        let inner_param = param.into_inner().next().unwrap();
        match inner_param.as_rule() {
            Rule::variable => params.push((inner_param.as_str(), AstType::Integer)),
            Rule::pointer => params.push((
                inner_param.into_inner().next().unwrap().as_str(),
                AstType::Pointer,
            )),
            _ => unreachable!("Unreacheable rule {:?}", inner_param.as_rule()),
        }
    }

    let body = AstExpr::parse(body_expr.into_iter().next().unwrap());
    AstDef::Function(name, params, body)
}

pub fn parse_check_expect(def: Pair<Rule>) -> AstDef {
    let contents = def.as_str();
    let mut inner_expr = def.into_inner();
    let lhs = AstExpr::parse(inner_expr.next().unwrap());
    let rhs = AstExpr::parse(inner_expr.next().unwrap());
    AstDef::CheckExpect(lhs, rhs, contents)
}

pub fn parse_check_assert(def: Pair<Rule>) -> AstDef {
    let contents = def.as_str();
    let body = AstExpr::parse(def.into_inner().next().unwrap());
    AstDef::CheckAssert(body, contents)
}

pub fn parse_check_error(def: Pair<Rule>) -> AstDef {
    // TODO: Get it really working
    let contents = def.as_str();
    let mut inner_expr = def.into_inner();
    let body = AstExpr::parse(inner_expr.next().unwrap());
    AstDef::CheckAssert(body, contents)
}
