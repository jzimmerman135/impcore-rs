use super::*;

pub fn parse_val(def: Pair<Rule>) -> AstDef {
    let mut def = def.into_inner();
    let name = def.next().unwrap().as_str();
    let value = AstExpr::parse(def.next().unwrap());
    AstDef::Global(name, value)
}

pub fn parse_define(def: Pair<Rule>) -> AstDef {
    let mut inner_expr = def.into_inner();
    let name = inner_expr.next().unwrap().as_str();
    let (param_exprs, body_expr): (Vec<_>, Vec<_>) =
        inner_expr.partition(|e| e.as_rule() == Rule::parameter);
    let params = param_exprs.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let scopes = vec![];
    let body = AstExpr::parse(body_expr.into_iter().next().unwrap());
    AstDef::Function(name, params, scopes, body)
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
