#[allow(unused)]
struct MacroEnv<'a> {
    pub variables: Vec<&'a str>,
    pub functions: Vec<&'a str>,
    pub includes: Vec<&'a str>,
}

impl<'a> MacroEnv<'a> {
    #[allow(unused)]
    pub fn new(&self, translation_unit: &str) -> Self {
        let variables = Vec::new();
        let functions = Vec::new();
        let includes = Vec::new();

        Self {
            variables,
            functions,
            includes,
        }
    }
}
