use crate::ast::*;
use std::collections::HashSet;

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
