use std::{
    collections::{hash_map, HashMap},
    hash::Hash,
};

use inkwell::values::{BasicValue, FunctionValue, GlobalValue, PointerValue};

use crate::compiler::Compiler;

pub type Name = i32;
pub const IT: Name = i32::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Type {
    Int,
    Pointer,
}

pub struct Env<'a> {
    pub tokens: Tokens<'a>,
    pub globaltys: HashMap<Name, Type>,
    pub funtys: HashMap<Name, Vec<(Name, Type)>>,
}

#[derive(Default)]
pub struct Values<'ctx> {
    functions: HashMap<Name, FunctionValue<'ctx>>,
    globals: HashMap<Name, GlobalValue<'ctx>>,
    params: HashMap<Name, PointerValue<'ctx>>,
    pub curr_function: Option<FunctionValue<'ctx>>,
}

// Static environment stuff
impl<'a> Env<'a> {
    pub fn new(tokens: Tokens<'a>) -> Self {
        Env {
            tokens,
            funtys: HashMap::new(),
            globaltys: HashMap::new(),
        }
    }

    pub fn is_pointer(&self, name: Name) -> bool {
        matches!(self.globaltys.get(&name), Some(Type::Pointer))
    }
}

impl<'ctx> Values<'ctx> {
    pub fn clear_formals(&mut self) {
        self.params.clear();
        self.curr_function = None;
    }

    pub fn add_var(&mut self, n: Name, g: GlobalValue<'ctx>) {
        self.globals.insert(n, g);
    }

    pub fn add_param(&mut self, n: Name, g: PointerValue<'ctx>) {
        self.params.insert(n, g);
    }

    pub fn add_function(&mut self, n: Name, f: FunctionValue<'ctx>) {
        self.functions.insert(n, f);
    }

    pub fn function(&self, name: &Name, env: &Env) -> Result<FunctionValue<'ctx>, String> {
        self.functions
            .get(name)
            .ok_or(format!(
                "Compiler bug, missing function {}",
                env.tokens.translate(name)
            ))
            .copied()
    }

    pub fn var(
        &self,
        name: &Name,
        env: &Env,
        compiler: &Compiler<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        if self.curr_function.is_none() {
            panic!("Trying to build var access outside of function");
        }

        let ptr_type = compiler.itype.ptr_type(compiler.addr_spc);
        let varty = env.varty(name)?;
        match self.params.get(name) {
            Some(&local) => match varty {
                Type::Pointer => Some(
                    compiler
                        .builder
                        .build_load(ptr_type, local, "load")
                        .into_pointer_value(),
                ),
                Type::Int => Some(local),
            },
            None => self.globals.get(name).map(|&g| match varty {
                Type::Pointer => compiler
                    .builder
                    .build_load(
                        ptr_type,
                        g.as_basic_value_enum().into_pointer_value(),
                        "load",
                    )
                    .into_pointer_value(),
                Type::Int => g.as_pointer_value(),
            }),
        }
        .ok_or(format!(
            "Compiler bug, missing variable {} in vals",
            env.tokens.translate(name)
        ))
    }
    pub fn globals(&self) -> hash_map::Iter<'_, i32, GlobalValue<'_>> {
        self.globals.iter()
    }
}

/// Does not check if name is actually bound
/// This should be a band aid for a more sophisticated type system

// Error messages

pub struct ErrorInfo {
    pub line: u32,
    pub col: u32,
}

// Tokenizer

#[derive(Default)]
pub struct Tokens<'a> {
    generator: Name,
    pub symbols: HashMap<Name, &'a str>,
    pub names: HashMap<&'a str, Name>,
}

pub trait TokenGen {
    fn tokengen(&mut self) -> Name;
}

impl TokenGen for Name {
    fn tokengen(&mut self) -> Name {
        let t = *self;
        *self += 1;
        t
    }
}

pub trait Tokenizer<'a, T>
where
    T: TokenGen,
{
    fn tokenize(&mut self, name: &'a str) -> T;
}

impl<'a> Tokenizer<'a, Name> for Tokens<'a> {
    fn tokenize(&mut self, name: &'a str) -> Name {
        if let Some(&token) = self.names.get(name) {
            token
        } else {
            let token = self.generator.tokengen();
            self.symbols.insert(token, name.trim());
            self.names.insert(name, token);
            token
        }
    }
}

impl<'a> Tokens<'a> {
    pub fn new() -> Self {
        Self {
            generator: 0,
            symbols: HashMap::new(),
            names: HashMap::new(),
        }
    }

    pub fn get(&self, name: &'a str) -> Name {
        *self
            .names
            .get(name)
            .unwrap_or_else(|| panic!("Unbound token {}", name))
    }

    pub fn translate(&self, token: &Name) -> &'a str {
        self.symbols
            .get(token)
            .unwrap_or_else(|| panic!("Unbound token {}", token))
    }
}

// environment utils

pub fn map_swap<K, V>(hashmap: &mut HashMap<K, V>, newpairs: HashMap<K, V>) -> Vec<(K, Option<V>)>
where
    K: Eq + Hash + Copy,
{
    let mut old = Vec::with_capacity(newpairs.len());
    for (k, v) in newpairs.into_iter() {
        old.push((k, hashmap.insert(k, v)));
    }
    old
}

pub fn map_restore<K, V>(hashmap: &mut HashMap<K, V>, oldpairs: Vec<(K, Option<V>)>)
where
    K: Eq + Hash,
{
    for (k, v) in oldpairs.into_iter() {
        if let Some(oldv) = v {
            hashmap.insert(k, oldv);
        } else {
            hashmap.remove(&k);
        }
    }
}
