use crate::ast::*;
use std::collections::{HashMap, HashSet};

#[allow(unused)]
pub struct SymbolEnv<'a> {
    functions: HashMap<&'a str, Vec<AstType>>,
    globals: HashSet<(&'a str, AstType)>,
    formals: HashSet<(&'a str, AstType)>,
}

pub fn append_garbage_collector(ast: &mut Ast) {
    ast.0.push(AstDef::FreeAll);
}

pub fn predefine_globals(ast: &mut Ast) {
    let mut global_names = HashSet::new();
    let mut declarations = vec![];
    let mut defs = std::mem::take(&mut ast.0)
        .into_iter()
        .map(|e| {
            if let AstDef::Global(n, ..) = e {
                if global_names.insert(n) {
                    declarations.push(AstDef::DeclareGlobal(n));
                }
            }
            e
        })
        .collect();

    declarations.append(&mut defs);
    *ast = Ast(declarations)
}

pub fn check_lazy_functions(ast: &mut Ast, env: &SymbolEnv) {
    let _ = ast;
    let _ = env;
    todo!();
}
