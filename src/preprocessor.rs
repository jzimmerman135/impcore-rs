use std::{
    collections::HashMap,
    fs, mem,
    path::{Path, PathBuf},
};

use crate::ast::{Ast, AstDef, AstExpr, AstMacro};
use rayon::prelude::*;
use regex::Regex;

#[derive(Debug)]
struct MacroEnv<'a> {
    pub replacers: HashMap<AstExpr<'a>, AstExpr<'a>>,
    pub functions: HashMap<&'a str, (Vec<AstExpr<'a>>, AstExpr<'a>)>,
}

impl<'a> MacroEnv<'a> {
    pub fn new() -> Self {
        Self {
            replacers: HashMap::new(),
            functions: HashMap::new(),
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

    pub fn take(&mut self, ast: Ast<'a>) -> Result<Ast<'a>, String> {
        let mut defs = vec![];
        for def in ast.defs.into_iter() {
            match def {
                AstDef::MacroDef(AstMacro::ImportFile(_)) => defs.push(def),
                AstDef::MacroDef(m) => {
                    match m {
                        AstMacro::Inliner(name, args, body) => {
                            self.functions.insert(name, (args, body));
                        }
                        AstMacro::Replacer(macroval, expr)
                            if !expr.contains(&mut |e| matches!(e, AstExpr::MacroVal(..))) =>
                        {
                            self.replacers.insert(macroval, expr);
                        }
                        AstMacro::Replacer(AstExpr::MacroVal(name), expr) => {
                            return Err(format!(
                                "Recursive macro #(replace {} ({:?})) not allowed",
                                name, expr
                            ))
                        }
                        _ => return Err("Something wrong in getting macros".to_string()),
                    };
                }
                _ => defs.push(def),
            }
        }
        Ok(Ast { defs })
    }
}

pub struct CodeBase(HashMap<String, String>);
impl CodeBase {
    pub fn get(&self, filepath: &str) -> Result<&String, String> {
        self.0
            .get(filepath)
            .ok_or(format!("Could not locate file {}", filepath))
    }

    fn parse_asts(&self) -> Result<HashMap<AstMacro, Ast>, String> {
        let map = self
            .0
            .par_iter()
            .map(|(name, contents)| Ok((AstMacro::ImportFile(name.as_str()), Ast::from(contents)?)))
            .collect::<Result<HashMap<AstMacro, Ast>, String>>()?;
        Ok(map)
    }

    pub fn build_ast<'a>(&'a self, entry_filepath: &'a Path) -> Result<Ast<'a>, String> {
        let mut asts = self.parse_asts()?;
        let entry_import =
            AstMacro::ImportFile(entry_filepath.file_name().unwrap().to_str().unwrap());
        let ast = join_trees(&mut asts, entry_import)?;
        Ok(ast.expand_macros()?.prepare())
    }

    pub fn collect(entry_filepath: &PathBuf) -> Result<Self, String> {
        let mut path = PathBuf::from(entry_filepath);
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop();
        let import_pattern =
            Regex::new(r#"#\(import\s+"(?P<filename>\S*)"\s*\)"#).expect("Failed regex build");
        let map = collect_code_recurse(&filename, &path, HashMap::new(), &import_pattern)?;
        Ok(CodeBase(map))
    }
}

fn collect_code_recurse(
    filename: &str,
    basedir: &PathBuf,
    mut included_files: HashMap<String, String>,
    import_pattern: &Regex,
) -> Result<HashMap<String, String>, String> {
    let mut dirclone = basedir.clone();
    dirclone.push(filename);
    let pathstring = dirclone.to_str().unwrap().to_string();
    let filename = filename.to_string();

    if included_files.contains_key(&filename) {
        return Ok(included_files);
    }

    let contents = fs::read_to_string(&pathstring)
        .map_err(|_| format!("Failed to open file '{}'", pathstring))?;

    included_files.insert(filename.clone(), String::new());

    for capture in import_pattern.captures_iter(&contents) {
        let filename = &capture["filename"];
        included_files = collect_code_recurse(filename, basedir, included_files, import_pattern)?;
    }

    included_files.insert(filename, contents);
    Ok(included_files)
}

fn join_trees<'a>(
    asts: &mut HashMap<AstMacro<'a>, Ast<'a>>,
    entrypoint: AstMacro<'a>,
) -> Result<Ast<'a>, String> {
    let base = asts
        .remove(&entrypoint)
        .ok_or(format!("Missing import {:?}", &entrypoint))?;

    let defs = base
        .defs
        .into_iter()
        .flat_map(|d| match d {
            AstDef::MacroDef(mut import @ AstMacro::ImportFile(..))
                if asts.contains_key(&import) =>
            {
                join_trees(asts, mem::take(&mut import))
                    .unwrap_or(Ast { defs: vec![] })
                    .defs
            }
            AstDef::MacroDef(AstMacro::ImportFile(..)) => vec![],
            _ => vec![d],
        })
        .collect();
    Ok(Ast { defs })
}

impl<'a> Ast<'a> {
    pub fn expand_macros(mut self) -> Result<Self, String> {
        let mut macro_env = MacroEnv::new();
        self = macro_env.take(self)?;
        self.defs = self
            .defs
            .into_iter()
            .map(|def| def.reconstruct(&|e| macro_env.try_replace(e)))
            .collect::<Result<_, _>>()
            .unwrap();
        Ok(self)
    }
}
