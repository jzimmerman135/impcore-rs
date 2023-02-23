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
    let mut param_exprs = inner_expr.map(AstExpr::parse).collect::<Vec<_>>();
    let body = param_exprs.pop().unwrap();
    let params = param_exprs
        .into_iter()
        .filter_map(|e| {
            if let AstExpr::Variable(name, _) = e {
                return Some(match name.strip_suffix(']') {
                    Some(ptrname) => (ptrname, AstType::Pointer),
                    None => (name, AstType::Integer),
                });
            }
            None
        })
        .collect();
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
