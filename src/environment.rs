use std::vec;

pub struct Env {
    pub globals: Vec<Global>,
    pub functions: Vec<Function>,
    pub formals: Vec<Formal>,
    pub memory: Vec<i32>,
}

impl Env {
    pub fn new(vram_bytes: usize) -> Self {
        Self {
            globals: vec![],
            functions: vec![],
            formals: vec![],
            memory: Vec::with_capacity(vram_bytes),
        }
    }
}

pub struct Global {}

pub struct Function {}

pub struct Formal {}
