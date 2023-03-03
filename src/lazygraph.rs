use crate::{
    ast::{AstDef, AstExpr},
    jit::Compiler,
};
use petgraph::Graph;
use std::collections::HashMap;

#[derive(Hash, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
enum LazyDep<'a> {
    Function(&'a str),
    Global(&'a str),
}

#[derive(Default)]
pub struct LazyGraph<'a> {
    defs: HashMap<LazyDep<'a>, &'a AstDef<'a>>,
    graph: Graph<LazyDep<'a>, i32>,
}

impl<'a> LazyGraph<'a> {
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
            graph: Graph::new(),
        }
    }

    pub fn why(&self, name: &str) -> String {
        format!("TODO! Unbound function {}", name)
    }

    pub fn eval(&mut self, def: &'a AstDef<'a>, compiler: &Compiler<'a>) -> Vec<&'a AstDef<'a>> {
        let dependee = match &def {
            AstDef::Global(name, ..) => LazyDep::Global(name),
            AstDef::Function(name, ..) => LazyDep::Function(name),
            _ => return vec![def],
        };

        self.add((dependee, def), def.get_dependencies(compiler));
        self.resolve(dependee)
    }

    fn add(&mut self, from: (LazyDep<'a>, &'a AstDef), to: Vec<LazyDep<'a>>) {
        let (dependee, def) = from;
        if let LazyDep::Function(..) = dependee {
            let dependee_node = self.graph.add_node(dependee);
            self.defs.insert(dependee, def);
            for depenency in to {
                let dependency_node = self.graph.add_node(depenency);
                self.graph.add_edge(dependee_node, dependency_node, 1);
            }
        }
    }

    fn resolve(&mut self, dependee: LazyDep<'a>) -> Vec<&'a AstDef<'a>> {
        let mut resolved_defs = vec![];
        let mut resolved_dependency = self
            .graph
            .node_indices()
            .find(|i| self.graph[*i] == dependee)
            .unwrap();
        loop {
            self.graph.remove_node(resolved_dependency);
            if let Some(def) = self.defs.remove(&self.graph[resolved_dependency]) {
                resolved_defs.push(def);
            }
            resolved_dependency = match self.graph.node_indices().find_map(|n| {
                match self.graph.neighbors_undirected(n).count() {
                    0 => Some(n),
                    _ => None,
                }
            }) {
                Some(resolved_dependee) => resolved_dependee,
                None => break,
            }
        }
        resolved_defs
    }
}

impl<'a> AstDef<'a> {
    fn get_dependencies(&self, compiler: &Compiler<'a>) -> Vec<LazyDep<'a>> {
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
            dependencies
        } else {
            vec![]
        }
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
