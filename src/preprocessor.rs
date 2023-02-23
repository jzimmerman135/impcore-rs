use crate::ast::{Ast, AstMacro};

#[allow(unused)]
struct MacroEnv<'a> {
    pub replacers: Vec<AstMacro<'a>>,
    pub imports: Vec<AstMacro<'a>>,
    pub functions: Vec<AstMacro<'a>>,
    pub depth: u32,
}

impl<'a> MacroEnv<'a> {
    pub fn new() -> Self {
        Self {
            replacers: vec![],
            imports: vec![],
            functions: vec![],
            depth: 0,
        }
    }
}

impl<'a> Ast<'a> {
    pub fn preprocess(self) -> Self {
        let _macro_env = MacroEnv::new();
        self
    }
}
