use std::{
    collections::{HashMap, HashSet},
    fs, mem,
    path::PathBuf,
};

use regex::Regex;

use crate::ast::{Ast, AstDef, AstExpr, AstMacro};

#[allow(unused)]
#[derive(Debug)]
struct MacroEnv<'a> {
    pub replacers: HashMap<AstExpr<'a>, AstExpr<'a>>,
    pub functions: HashMap<&'a str, (Vec<AstExpr<'a>>, AstExpr<'a>)>,
    pub included: HashSet<&'a str>,
    pub depth: u32,
}

impl<'a> MacroEnv<'a> {
    pub fn new() -> Self {
        Self {
            replacers: HashMap::new(),
            functions: HashMap::new(),
            included: HashSet::new(),
            depth: 0,
        }
    }

    pub fn try_replace(&self, exp: AstExpr<'a>) -> Result<AstExpr<'a>, String> {
        match &exp {
            AstExpr::MacroVal(name) => self
                .replacers
                .get(&exp)
                .ok_or(format!("Macro not found: {}", name))
                .cloned(),
            AstExpr::Call(name, args) if name.starts_with('\'') => {
                let (formals, body) = self
                    .functions
                    .get(name)
                    .ok_or(format!("Inline Function: {} not found", name))?;
                let argmap = formals
                    .iter()
                    .zip(args)
                    .collect::<HashMap<&AstExpr, &AstExpr>>();
                let newbody = body.clone().reconstruct(&|e| match &argmap.get(&e) {
                    Some(&v) => Ok(v.clone()),
                    None => Ok(e),
                })?;
                Ok(newbody)
            }
            _ => Ok(exp),
        }
    }

    pub fn take(&mut self, mut ast: Ast<'a>) -> Ast<'a> {
        let mut defs = vec![];
        for def in ast.defs.into_iter() {
            match def {
                AstDef::MacroDef(m) => {
                    match m {
                        AstMacro::Inliner(name, args, body) => {
                            self.functions.insert(name, (args, body));
                        }
                        AstMacro::Replacer(macroval, expr) => {
                            self.replacers.insert(macroval, expr);
                        }
                        AstMacro::ImportFile(filename) => {
                            self.included.insert(filename);
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

pub struct CodeBase(HashMap<String, String>);
impl CodeBase {
    pub fn get(&self, filepath: &str) -> Result<&String, String> {
        self.0
            .get(filepath)
            .ok_or(format!("Could not locate file {}", filepath))
    }

    fn parse_asts<'a>(&'a self) -> Result<HashMap<&'a str, Ast<'a>>, String> {
        let mut map = HashMap::new();
        for (name, contents) in self.0.iter() {
            map.insert(name.as_str(), Ast::from(contents)?);
        }
        Ok(map)
    }

    pub fn build_ast<'a>(&'a self, entry_filepath: &str) -> Result<Ast<'a>, String> {
        let mut asts = self.parse_asts()?;
        let ast = asts
            .remove(entry_filepath)
            .ok_or(format!("Couldn't find ast for {}", entry_filepath))?;
        Ok(ast)
    }

    pub fn collect(entry_filepath: &str) -> Result<Self, String> {
        let mut path = PathBuf::from(entry_filepath);
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop();
        let import_pattern = Regex::new(r#"#\(import\s+"(?P<filename>\S*)"\s*\)"#).unwrap();
        let map = collect_code_recurse(&filename, &mut path, HashMap::new(), &import_pattern)?;
        Ok(CodeBase(map))
    }
}

fn collect_code_recurse(
    filename: &str,
    basedir: &mut PathBuf,
    mut included_files: HashMap<String, String>,
    import_pattern: &Regex,
) -> Result<HashMap<String, String>, String> {
    basedir.push(filename);
    let pathstring = basedir.to_str().unwrap().to_string();
    basedir.pop();

    if included_files.contains_key(&pathstring) {
        return Ok(included_files);
    }

    let contents = fs::read_to_string(&pathstring)
        .map_err(|_| format!("Failed to open filename {:?} {}", basedir, pathstring))?;

    included_files.insert(pathstring.clone(), String::new());

    for capture in import_pattern.captures_iter(&contents) {
        let filename = &capture["filename"];
        included_files = collect_code_recurse(filename, basedir, included_files, import_pattern)?;
    }

    included_files.insert(pathstring, contents);
    Ok(included_files)
}

impl<'a> Ast<'a> {
    pub fn expand_macros(mut self) -> Self {
        let mut macro_env = MacroEnv::new();
        self = macro_env.take(self);
        self.defs = self
            .defs
            .into_iter()
            .map(|def| def.reconstruct(&|e| macro_env.try_replace(e)))
            .collect::<Result<_, _>>()
            .unwrap();
        self
    }
}
