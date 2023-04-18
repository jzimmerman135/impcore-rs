use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{
    ast::{Def, Exp, TokenString},
    env::{Name, Tokens, IT},
    implib, Rip,
};
use colored::Colorize;
use itertools::{Either, Itertools};
use std::hash::Hash;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Dep {
    Function(Name),
    Global(Name),
}

#[derive(Debug)]
enum LazyGraphError<T> {
    CannotResolve(T, Vec<T>),
    NotFound(T),
}

use LazyGraphError::*;

pub struct LazyGraph<D, E>
where
    D: Hash + PartialEq + Eq,
{
    pub still_unresolved: HashMap<D, HashSet<D>>,
    already_resolved: HashSet<D>,
    defs: HashMap<D, E>,
}

impl<D, T> Default for LazyGraph<D, T>
where
    D: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D, T> LazyGraph<D, T>
where
    D: Eq + Hash,
{
    pub fn new() -> Self {
        LazyGraph {
            still_unresolved: HashMap::new(),
            already_resolved: HashSet::new(),
            defs: HashMap::new(),
        }
    }
}

impl<D, T> LazyGraph<D, T>
where
    D: Hash + Eq + Clone + Copy + Debug + TheIt + TokenString,
    T: GetDeps<D>,
{
    /// Ok(ready_to_compile_defs) || Err(failed_def, unresolved_dependencies)
    pub fn bump(&mut self, def: T, tokens: &Tokens) -> Result<Vec<T>, (T, Vec<D>)> {
        if let Some(libname) = def.is_import() {
            self.resolve_lib(libname, tokens).rip();
            return Ok(vec![def]);
        }

        let (depee, depcys) = match def.deps() {
            None => return Ok(vec![def]),
            Some((d, depcys)) if d.is_the_it() => {
                if !depcys.iter().all(|d| self.already_resolved.contains(d)) {
                    return Err((
                        def,
                        depcys
                            .into_iter()
                            .filter(|d| !self.already_resolved.contains(d))
                            .collect(),
                    ));
                } else {
                    return Ok(vec![def]);
                }
            }
            Some(d) => d,
        };

        self.defs.insert(depee, def);
        self.add(&depee, depcys);
        Ok(match self.try_resolve(&depee) {
            Ok(ready) => ready.iter().filter_map(|d| self.defs.remove(d)).collect(),
            Err(NotFound(..)) => unreachable!("This is a compiler bug in LazyGraph"),
            Err(..) => vec![],
        })
    }

    pub fn resolve_lib(&mut self, libname: &str, tokens: &Tokens) -> Result<(), String> {
        let (_, fnmappings) = implib::libmapping(libname)?;
        fnmappings
            .iter()
            .map(|&(name, _, _)| T::as_function(&tokens.get(name)))
            .for_each(|d| {
                self.already_resolved.insert(d);
            });
        Ok(())
    }
}

impl<T> LazyGraph<Dep, T> {
    pub fn already_resolved(&self, tokens: &Tokens) -> String {
        self.already_resolved
            .iter()
            .map(|d| match d {
                Dep::Global(d) => format!("val({}, {})", tokens.translate(d), d),
                Dep::Function(d) => format!("fun({})", tokens.translate(d)),
            })
            .join(" ")
    }

    pub fn explain(&self, (def, deps): &(Def, Vec<Dep>), tokens: &Tokens) -> String {
        fn diagnose<'a, T>(
            graph: &'a LazyGraph<Dep, T>,
            d: &'a Dep,
            diagnosed: &mut HashSet<&'a Dep>,
        ) {
            match &d {
                Dep::Global(..) => {
                    diagnosed.insert(d);
                }
                Dep::Function(..) => {
                    if let Some(deps) = graph.still_unresolved.get(d) {
                        deps.iter().for_each(|d| diagnose(graph, d, diagnosed));
                    } else {
                        diagnosed.insert(d);
                    }
                }
            };
        }

        let mut unresolved = HashSet::new();
        deps.iter().for_each(|d| diagnose(self, d, &mut unresolved));

        let (missing_fns, missing_gs): (Vec<_>, Vec<_>) =
            unresolved.iter().partition_map(|d| match &d {
                Dep::Function(n) => Either::Left(String::from("'") + tokens.translate(n) + "'"),
                Dep::Global(n) => {
                    let varstr = tokens.translate(n);
                    if varstr.ends_with('[') {
                        Either::Right(varstr.to_owned() + "]")
                    } else {
                        Either::Right(varstr.to_owned())
                    }
                }
            });

        let mut missing_fnstring = String::new();
        let mut missing_varstring = String::new();
        if !missing_fns.is_empty() {
            missing_fnstring =
                "Unbound functions: ".red().to_string() + &mut missing_fns.join(", ").cyan();
        }
        if !missing_gs.is_empty() {
            missing_varstring = "Unbound globals: ".red().to_string() + &mut missing_gs.join(", ");
        }
        format!(
            "Cannot compile {}: {}\n{}{}{}",
            if matches!(def, Def::Exp(..)) {
                "top level expression"
            } else {
                "definition"
            },
            def.to_string(tokens),
            missing_fnstring,
            if !missing_fns.is_empty() && !missing_gs.is_empty() {
                "\n"
            } else {
                ""
            },
            missing_varstring
        )
    }
}

trait DependencyGraph<T> {
    fn add(&mut self, depee: &T, depcys: HashSet<T>);
    fn try_resolve(&mut self, dependency: &T) -> Result<Vec<T>, LazyGraphError<T>>;
}

impl<D, T> DependencyGraph<D> for LazyGraph<D, T>
where
    D: Hash + PartialEq + Eq + Clone + Copy,
{
    fn add(&mut self, depee: &D, new_depcys: HashSet<D>) {
        self.still_unresolved
            .entry(*depee)
            .or_insert_with(HashSet::new)
            .extend(
                new_depcys
                    .into_iter()
                    .filter(|d| !self.already_resolved.contains(d)),
            );
    }

    fn try_resolve(&mut self, depee: &D) -> Result<Vec<D>, LazyGraphError<D>> {
        if self.already_resolved.contains(depee) {
            return Ok(Vec::new());
        }

        match self.still_unresolved.get(depee) {
            Some(depcys) if !depcys.is_empty() => {
                return Err(CannotResolve(*depee, depcys.iter().copied().collect()))
            }
            None => return Err(NotFound(*depee)),
            _ => {}
        };

        self.already_resolved.insert(*depee);
        self.still_unresolved.remove(depee);

        let also_ready = self
            .still_unresolved
            .iter_mut()
            .filter_map(|(otherdepee, otherdepcys)| {
                otherdepcys.remove(depee);
                if otherdepcys.is_empty() {
                    Some(*otherdepee)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut ready = also_ready
            .iter()
            .filter_map(|d| self.try_resolve(d).ok())
            .flatten()
            .collect::<Vec<_>>();
        ready.insert(0, *depee);
        Ok(ready)
    }
}

pub trait TheIt {
    fn is_the_it(&self) -> bool;
}

impl TheIt for Dep {
    fn is_the_it(&self) -> bool {
        matches!(&self, Dep::Global(IT))
    }
}

impl TokenString for Dep {
    fn to_string(&self, tokens: &Tokens) -> String {
        match self {
            Dep::Global(n) => format!("global {}", tokens.translate(n)),
            Dep::Function(n) => format!("function {}", tokens.translate(n)),
        }
    }
}

pub trait GetDeps<T>
where
    T: PartialEq + Eq + Hash,
{
    fn deps(&self) -> Option<(T, HashSet<T>)>;
    fn is_import(&self) -> Option<&str>;
    fn as_function(n: &Name) -> T;
}

impl GetDeps<Dep> for Def {
    fn deps(&self) -> Option<(Dep, HashSet<Dep>)> {
        let mut res = HashSet::new();
        match self {
            Def::Val(n, e) => {
                e.freevars(&mut res, &HashSet::new());
                Some((Dep::Global(*n), res))
            }
            Def::Define(n, bs, e) => {
                e.freevars(&mut res, &bs.iter().map(|n| Dep::Global(*n)).collect());
                Some((Dep::Function(*n), res))
            }
            Def::Exp(e) => {
                e.freevars(&mut res, &HashSet::new());
                Some((Dep::Global(IT), res))
            }
            _ => None,
        }
    }

    fn is_import(&self) -> Option<&str> {
        if let Def::Import(n) = self {
            Some(n)
        } else {
            None
        }
    }

    fn as_function(name: &Name) -> Dep {
        Dep::Function(*name)
    }
}

impl Exp {
    fn freevars(&self, deps: &mut HashSet<Dep>, formals: &HashSet<Dep>) {
        match self {
            Exp::Var(n, i) => {
                if !formals.contains(&Dep::Global(*n)) {
                    deps.insert(Dep::Global(*n));
                }
                if let Some(i) = i {
                    i.freevars(deps, formals);
                }
            }
            Exp::Set(n, i, v) => {
                if !formals.contains(&Dep::Global(*n)) {
                    deps.insert(Dep::Global(*n));
                }
                if let Some(i) = i {
                    i.freevars(deps, formals);
                }
                v.freevars(deps, formals);
            }
            Exp::Binary(_, l, r) => {
                l.freevars(deps, formals);
                r.freevars(deps, formals);
            }
            Exp::Unary(_, e) => {
                e.freevars(deps, formals);
            }
            Exp::If(c, t, e) => {
                c.freevars(deps, formals);
                t.freevars(deps, formals);
                e.freevars(deps, formals);
            }
            Exp::While(g, b) => {
                g.freevars(deps, formals);
                b.freevars(deps, formals);
            }
            Exp::Begin(es) => {
                es.iter().for_each(|e| e.freevars(deps, formals));
            }
            Exp::Match(p, bs, d) => {
                p.freevars(deps, formals);
                bs.iter().for_each(|(_, e)| e.freevars(deps, formals));
                d.freevars(deps, formals);
            }
            Exp::Apply(n, xs) => {
                if !formals.contains(&Dep::Function(*n)) {
                    deps.insert(Dep::Function(*n));
                }
                xs.iter().for_each(|e| e.freevars(deps, formals));
            }
            Exp::Literal(_) => {}
        }
    }
}
