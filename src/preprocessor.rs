use std::{collections::HashSet, fs, mem};

use crate::{
    ast::{Ast, AstDef, AstMacro},
    parser::ImpcoreParser,
};

#[allow(unused)]
struct MacroEnv<'a> {
    pub replacers: Vec<AstMacro<'a>>,
    pub imports: Vec<(AstMacro<'a>, Ast<'a>)>,
    pub functions: Vec<AstMacro<'a>>,
    pub included: HashSet<&'a str>,
    pub depth: u32,
}

impl<'a> MacroEnv<'a> {
    pub fn new() -> Self {
        let replacers = vec![];
        let imports = vec![];
        let functions = vec![];
        Self {
            replacers,
            imports,
            functions,
            included: HashSet::new(),
            depth: 0,
        }
    }

    pub fn take(&mut self, ast: Ast<'a>) -> Ast<'a> {
        let mut defs = vec![];
        for def in ast.0.into_iter() {
            match def {
                AstDef::MacroDef(m) => {
                    match m {
                        AstMacro::Inliner(..) => self.functions.push(m),
                        AstMacro::Replacer(..) => self.replacers.push(m),
                        AstMacro::ImportFile(filename) => {
                            self.imports.push((m, Ast(vec![])));
                            defs.push(AstDef::MacroDef(AstMacro::ImportFile(*&&filename)));
                        }
                    };
                }
                _ => defs.push(def),
            }
        }
        Ast(defs)
    }

    #[allow(dead_code)]
    pub fn place_files(&'a mut self, ast: &mut Ast<'a>) {
        let defs_with_imports = mem::take(&mut ast.0)
            .into_iter()
            .map(|def| match &def {
                AstDef::MacroDef(AstMacro::ImportFile(filename))
                    if !self.included.insert(filename) =>
                {
                    // let contents = fs::read_to_string(filename).unwrap();
                    // ImpcoreParser::generate_ast(contents.as_str())
                    //     .unwrap()
                    //     .preprocess()
                    //     .prepare()
                    //     .0
                    vec![def]
                }

                AstDef::MacroDef(AstMacro::ImportFile(_)) => vec![],
                _ => vec![def],
            })
            .collect::<Vec<_>>();

        *ast = Ast(defs_with_imports.into_iter().flatten().collect());
    }
}

impl<'a> Ast<'a> {
    pub fn preprocess(mut self) -> Self {
        let mut macro_env = MacroEnv::new();
        self = macro_env.take(self);
        self
    }
}
