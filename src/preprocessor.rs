use crate::ast::AstMacro;

#[allow(unused)]
struct MacroEnv<'a> {
    pub replacers: Vec<AstMacro<'a>>,
}

impl<'a> MacroEnv<'a> {
    #[allow(unused)]
    pub fn new(&self, translation_unit: &str) -> Self {
        Self { replacers: vec![] }
    }
}
