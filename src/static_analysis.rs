#[allow(dead_code)]
/**
 * STATIC_ANALYSIS
 * This module is deprecated (for now)
 * */
use crate::ast::*;
use std::collections::HashSet;
impl<'a> Ast<'a> {
    #[allow(unused_mut)]
    pub fn prepare(mut self) -> Self {
        self
    }
}

fn _predefine_globals(ast: &mut Ast) {
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
