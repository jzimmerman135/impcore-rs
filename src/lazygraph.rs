use crate::{
    ast::{AstDef, AstExpr},
    jit::Compiler,
};
use petgraph::graph::NodeIndex;
use petgraph::{Direction::Outgoing, Graph};
use std::collections::HashMap;

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
enum LazyDep<'a> {
    Function(&'a str),
    Global(&'a str),
}

#[derive(Default)]
pub struct LazyGraph<'a> {
    def_table: HashMap<LazyDep<'a>, &'a AstDef<'a>>,
    graph: Graph<LazyDep<'a>, i32>,
}

impl<'a> LazyGraph<'a> {
    pub fn new() -> Self {
        Self {
            def_table: HashMap::new(),
            graph: Graph::new(),
        }
    }

    pub fn why(&self, name: &str) -> String {
        format!("TODO! Unbound function {}", name)
    }

    pub fn eval(&mut self, def: &'a AstDef<'a>, compiler: &Compiler<'a>) -> Vec<&'a AstDef<'a>> {
        let dependencies = def.get_unmet_dependencies(compiler);
        match &def {
            AstDef::Global(name, ..) => {
                let mut ready_defs = vec![def];
                ready_defs.append(&mut self.resolve(LazyDep::Global(name)));
                ready_defs
            }
            AstDef::Function(name, ..) if dependencies.is_empty() => {
                let mut ready_defs = vec![def];
                ready_defs.append(&mut self.resolve(LazyDep::Function(name)));
                ready_defs
            }
            AstDef::Function(name, ..) => {
                self.add((LazyDep::Function(name), def), dependencies);
                vec![]
            }
            _ => vec![def],
        }
    }

    /// Adds function to the dependency graph with edges to all needs, also stores the def in the lazy table
    fn add(&mut self, function: (LazyDep<'a>, &'a AstDef), needs: Vec<LazyDep<'a>>) {
        let (dependee, def) = function;
        if let LazyDep::Function(..) = dependee {
            let dependee_node = self.graph.add_node(dependee);
            self.def_table.insert(dependee, def);
            for depenency in needs {
                let dependency_node = self.graph.add_node(depenency);
                self.graph.add_edge(dependee_node, dependency_node, 1);
            }
        }
    }

    /// Removes dependee from the graph and all other functions that are ready to be defined  
    fn resolve(&mut self, dependee: LazyDep<'a>) -> Vec<&'a AstDef<'a>> {
        let mut resolved_dependency = match self.find(dependee) {
            Some(found_dependee) => found_dependee,
            None => return vec![],
        };
        let mut resolved_defs = vec![];
        loop {
            let resolved_lazydep = self.graph[resolved_dependency];
            self.graph.remove_node(resolved_dependency);
            if let Some(def) = self.def_table.remove(&resolved_lazydep) {
                resolved_defs.push(def);
            }
            resolved_dependency = match self.find_next_resolved() {
                Some(next_resolved) => next_resolved,
                None => break,
            };
        }
        resolved_defs
    }

    fn find_next_resolved(&self) -> Option<NodeIndex> {
        self.graph.node_indices().find_map(|n| {
            match self.graph.neighbors_directed(n, Outgoing).count() {
                0 => Some(n),
                _ => None,
            }
        })
    }

    fn find(&self, dep: LazyDep) -> Option<NodeIndex> {
        self.graph.node_indices().find(|&i| self.graph[i] == dep)
    }
}

impl<'a> AstDef<'a> {
    fn get_unmet_dependencies(&self, compiler: &Compiler<'a>) -> Vec<LazyDep<'a>> {
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
            dependencies.dedup();
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
