use crate::ast::*;
use std::collections::{HashMap, HashSet};
impl<'a> Ast<'a> {
    pub fn prepare(mut self) -> Self {
        // predefine_globals(&mut self);
        append_garbage_collector(&mut self);
        self
    }
}

fn append_garbage_collector(ast: &mut Ast) {
    ast.defs.push(AstDef::FreeAll);
}

#[allow(unused)]
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

#[allow(dead_code)]
pub struct SymbolEnv<'a> {
    functions: HashMap<&'a str, Vec<AstType>>,
    globals: HashSet<(&'a str, AstType)>,
    formals: HashSet<(&'a str, AstType)>,
}

#[allow(dead_code)]
fn check_lazy_functions(ast: &mut Ast, env: &SymbolEnv) {
    let _ = ast;
    let _ = env;
    todo!();
}
