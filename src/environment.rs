use crate::{
    // functions::{Formals, Functions},
    globals::Globals,
    tests::TestExpr,
};

pub struct Env {
    // pub globals: Globals,
    // pub functions: Functions,
    // pub formals: Formals,
    // pub tests: Vec<TestExpr>,
    // pub memory: Vec<i32>,
}

impl Env {
    pub fn new(vram_bytes: usize) -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn run_tests(&self) {
        todo!()
    }
}
