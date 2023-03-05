/**
 * STATIC_ANALYSIS
 * This module doesn't do much yet, it only really adds a cleanup instruction at the bottom of ast
 * */
use crate::ast::*;
use std::collections::HashSet;
impl<'a> Ast<'a> {
    pub fn prepare(mut self) -> Self {
        append_garbage_collector(&mut self);
        self
    }
}

fn append_garbage_collector(ast: &mut Ast) {
    ast.defs.push(AstDef::FreeAll);
}

#[allow(dead_code)]
fn predefine_globals(ast: &mut Ast) {
    let mut global_names = HashSet::new();
    let mut declarations = vec![];
    let mut defs = std::mem::take(&mut ast.defs)
        .into_iter()
        .map(|e| match e {
            AstDef::Global(n, _, var_type) if global_names.insert(n) => {
                declarations.push(AstDef::DeclareGlobal(n, var_type));
                e
            }
            _ => e,
        })
        .collect();

    declarations.append(&mut defs);
    ast.defs = declarations;
}
