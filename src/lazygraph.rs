use crate::{
    ast::{AstDef, AstExpr},
    errors::UNBOUND_FUNCTION,
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

impl<'a> LazyDep<'a> {
    /// given an error message, check if it starts with the global UNBOUND_FUNCTION error prefix
    /// and if it does, make it into a lazy dep, otherwise fail with no messsage
    /// This is really dangerous and stupid way to handle errors
    fn from_errstr(err: &'a str) -> Option<Self> {
        if let Some(message) = err.strip_prefix(UNBOUND_FUNCTION) {
            return Some(LazyDep::Function(message));
        }
        None
    }

    fn name(&self) -> &'a str {
        match self {
            LazyDep::Global(name) => name,
            LazyDep::Function(name) => name,
        }
    }

    fn typename(&self) -> &str {
        match self {
            LazyDep::Global(..) => "Global",
            LazyDep::Function(..) => "Function",
        }
    }
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

    /// Trace unresolved dependencies and return an appropriate error message
    /// Errstring should be given from a failed call to codegen::codegen_call
    pub fn why_cant(&self, errstring: String) -> String {
        let dependee = match LazyDep::from_errstr(&errstring) {
            Some(x) => x,
            None => return errstring,
        };

        let node = match self.find(dependee) {
            Some(node) => node,
            None => {
                return format!(
                    "Unbound {} {}",
                    dependee.typename().to_lowercase(),
                    dependee.name()
                )
            }
        };

        let mut needs = self.graph.neighbors_directed(node, Outgoing);
        let unmet_dependency = self.graph[needs.next().unwrap()];
        format!(
            "Unbound {} {}",
            unmet_dependency.typename().to_lowercase(),
            unmet_dependency.name()
        )
    }

    // Given a definition, return all other definitions that are ready to be compiled
    pub fn eval(&mut self, def: &'a AstDef<'a>, compiler: &Compiler<'a>) -> Vec<&'a AstDef<'a>> {
        let dependencies = def.get_unmet_dependencies(compiler);
        let res = match &def {
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
                self.add((LazyDep::Function(name), def), dependencies.clone());
                vec![]
            }
            _ => return vec![def],
        };
        res
    }

    /// Adds function to the dependency graph with edges to all needs, also stores the def in the lazy table
    fn add(&mut self, function: (LazyDep<'a>, &'a AstDef), needs: Vec<LazyDep<'a>>) {
        let (dependee, def) = function;
        if let LazyDep::Function(..) = dependee {
            let dependee_node = self.graph.add_node(dependee);
            self.def_table.insert(dependee, def);
            for dependency in needs {
                let dependency_node = self.graph.add_node(dependency);
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
                Some(next_resolved)
                    if matches!(self.graph[next_resolved], LazyDep::Function(..)) =>
                {
                    next_resolved
                }
                _ => break,
            };
        }
        resolved_defs
    }

    fn find_next_resolved(&self) -> Option<NodeIndex> {
        self.graph
            .node_indices()
            .find(|&n| self.graph.neighbors_directed(n, Outgoing).count() == 0)
    }

    fn find(&self, dep: LazyDep) -> Option<NodeIndex> {
        self.graph.node_indices().find(|&i| self.graph[i] == dep)
    }
}

impl<'a> AstDef<'a> {
    fn get_unmet_dependencies(&self, compiler: &Compiler<'a>) -> Vec<LazyDep<'a>> {
        if let Self::Function(parent_fname, args, _) = self {
            let mut dependencies = vec![];
            let params = args.iter().map(|a| a.0).collect::<Vec<_>>();
            self.for_each_child(&mut |e| {
                match e {
                    AstExpr::Call(name, ..)
                        if !compiler.has_function(name) && name != parent_fname =>
                    {
                        dependencies.push(LazyDep::Function(name))
                    }
                    AstExpr::Variable(name, ..)
                        if !compiler.has_global(name) && !params.contains(name) =>
                    {
                        dependencies.push(LazyDep::Global(name))
                    }
                    _ => {}
                };
                Ok(())
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
