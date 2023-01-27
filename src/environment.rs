use std::{collections::HashMap, hash::Hash, vec};

use crate::{functions::Function, globals::Global};

pub struct Env<'a> {
    pub globals: HashMap<&'a str, Global>,
    pub functions: HashMap<&'a str, Function>,
    pub formals: HashMap<&'a str, Formal>,
    pub memory: Vec<i32>,
}

impl<'a> Env<'a> {
    pub fn new(vram_bytes: usize) -> Self {
        Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            formals: HashMap::new(),
            memory: Vec::with_capacity(vram_bytes),
        }
    }
}

pub struct Formal {}
