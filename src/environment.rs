use std::collections::HashMap;

use crate::{
    functions::{Formal, Function},
    globals::Global,
    tests::TestExpr,
};

pub struct Env<'a> {
    pub globals: HashMap<&'a str, Global>,
    pub functions: HashMap<&'a str, Function>,
    pub formals: HashMap<&'a str, Formal>,
    pub tests: Vec<TestExpr>,
    pub memory: Vec<i32>,
}

impl<'a> Env<'a> {
    pub fn new(vram_bytes: usize) -> Self {
        Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            formals: HashMap::new(),
            tests: Vec::new(),
            memory: Vec::with_capacity(vram_bytes),
        }
    }

    #[allow(dead_code)]
    pub fn run_tests(&self) {
        todo!()
    }
}
