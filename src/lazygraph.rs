use petgraph::csr::Csr;
use std::collections::HashMap;

use crate::{
    ast::{AstDef, AstExpr},
    jit::Compiler,
};

#[derive(Hash, Clone, Copy)]
enum LazyDep<'a> {
    Function(&'a str),
    Global(&'a str),
}

pub struct LazyGraph<'a> {
    defs: HashMap<&'a str, &'a AstDef<'a>>,
    graph: Csr,
}

impl<'a> LazyGraph<'a> {
    fn eval(&mut self, def: &'a AstDef, compiler: &Compiler) -> Vec<&'a AstDef> {
        let dependee = match &def {
            AstDef::Global(name, ..) => LazyDep::Global(name),
            AstDef::Function(name, ..) => LazyDep::Function(name),
            _ => return vec![def],
        };

        self.push((dependee, def), def.get_dependencies(compiler));
        self.pop(dependee)
    }

    fn push(&mut self, from: (LazyDep, &AstDef), to: Vec<LazyDep>) {
        // Pseudocode
        // from = lazygraph.find_or_insert(from)
        // for dep in to {
        //     lazygraph.connect_or_insert(from, to)
        // }
        todo!();
    }

    fn pop(&mut self, dep: LazyDep) -> Vec<&AstDef> {
        // Pseudocode
        // remove def from the graph
        // while lazygraph.pop_unconnected_nodes() is not None {
        //
        // }
        todo!();
    }
}

impl<'a> AstDef<'a> {
    fn get_dependencies(&self, compiler: &Compiler) -> Vec<LazyDep<'a>> {
        if let Self::Function(_, args, _) = self {
            let mut dependencies = vec![];
            let params = args.iter().map(|a| a.0).collect::<Vec<_>>();
            self.for_each_child(&mut |e| {
                Ok(match e {
                    AstExpr::Call(name, ..) if !compiler.has_function(name) => {
                        dependencies.push(LazyDep::Function(name));
                    }
                    AstExpr::Variable(name, ..)
                        if !compiler.has_global(name) && !params.contains(name) =>
                    {
                        dependencies.push(LazyDep::Global(name));
                    }
                    _ => (),
                })
            })
            .unwrap();
            return dependencies;
        }
        vec![]
    }
}

impl<'a> Compiler<'a> {
    fn has_function(&self, function: &str) -> bool {
        self.module.get_function(function).is_some() || self.lib.contains_key(function)
    }

    fn has_global(&self, global: &str) -> bool {
        self.global_table.contains_key(global)
    }
}
