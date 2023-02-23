use super::*;
use crate::ast::AstMacro;

pub fn parse_macroval(expr: Pair<Rule>) -> AstExpr {
    let name = expr.as_str();
    AstExpr::MacroVal(name)
}

pub fn parse_importlib(def: Pair<Rule>) -> AstDef {
    let name = def.as_str();
    AstDef::ImportLib(name)
}

pub fn parse_importfile(def: Pair<Rule>) -> AstDef {
    let path = def.as_str();
    AstDef::MacroDef(AstMacro::ImportFile(path))
}

pub fn parse_replacer(def: Pair<Rule>) -> AstDef {
    let mut innerdef = def.into_inner();
    let identifier = innerdef.next().unwrap().as_str();
    let expr = AstExpr::parse(innerdef.next().unwrap());
    AstDef::MacroDef(AstMacro::Replacer(identifier, expr))
}

pub fn parse_inliner(def: Pair<Rule>) -> AstDef {
    let mut innerdef = def.into_inner();
    let identifier = innerdef.next().unwrap().as_str();
    let mut params = innerdef.map(AstExpr::parse).collect::<Vec<_>>();
    let replacement = params.pop().unwrap();
    AstDef::MacroDef(AstMacro::Inliner(identifier, params, replacement))
}
