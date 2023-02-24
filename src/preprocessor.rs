use std::{
    collections::{HashMap, HashSet},
    fs, mem,
};

use regex::Regex;

use crate::ast::{Ast, AstDef, AstMacro};

#[allow(unused)]
struct MacroEnv<'a> {
    pub replacers: Vec<AstMacro<'a>>,
    pub functions: Vec<AstMacro<'a>>,
    pub included: HashSet<&'a str>,
    pub depth: u32,
}

impl<'a> MacroEnv<'a> {
    pub fn new() -> Self {
        let replacers = vec![];
        let functions = vec![];
        Self {
            replacers,
            functions,
            included: HashSet::new(),
            depth: 0,
        }
    }

    pub fn take(&mut self, mut ast: Ast<'a>) -> Ast<'a> {
        let mut defs = vec![];
        for def in ast.defs.into_iter() {
            match def {
                AstDef::MacroDef(m) => {
                    match m {
                        AstMacro::Inliner(..) => self.functions.push(m),
                        AstMacro::Replacer(..) => self.replacers.push(m),
                        AstMacro::ImportFile(filename) => {
                            defs.push(AstDef::MacroDef(AstMacro::ImportFile(filename)));
                        }
                    };
                }
                _ => defs.push(def),
            }
        }
        ast.defs = defs;
        ast
    }

    #[allow(dead_code)]
    pub fn place_files(&'a mut self, ast: &mut Ast<'a>) {
        let defs_with_imports = mem::take(&mut ast.defs)
            .into_iter()
            .flat_map(|def| match &def {
                AstDef::MacroDef(AstMacro::ImportFile(filename))
                    if !self.included.insert(filename) =>
                {
                    vec![def]
                }

                AstDef::MacroDef(AstMacro::ImportFile(_)) => vec![],
                _ => vec![def],
            })
            .collect::<Vec<_>>();

        ast.defs = defs_with_imports;
    }
}

pub fn collect_code<'a>(entryfile: &'a str) -> Vec<(String, String)> {
    let mut files = HashMap::new();
    let import_pattern = Regex::new(r#"#\(import\s+"(\S*)"\s*\)"#).unwrap();
    collect_code_helper(entryfile, &mut files, &import_pattern);
    files.into_iter().collect()
}

fn collect_code_helper<'a>(
    filename: &'a str,
    included_files: &mut HashMap<String, String>,
    import_pattern: &Regex,
) {
    if included_files.contains_key(filename) {
        return;
    }
    let contents = fs::read_to_string(&filename).unwrap();
    for capture in import_pattern.captures_iter(&contents) {
        let filename = &capture["filename"];
        collect_code_helper(filename, included_files, import_pattern);
    }
    included_files.insert(filename.to_string(), contents);
}

impl<'a> Ast<'a> {
    pub fn preprocess(mut self) -> Self {
        let mut macro_env = MacroEnv::new();
        self = macro_env.take(self);
        self
    }
}
