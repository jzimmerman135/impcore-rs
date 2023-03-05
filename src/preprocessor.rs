use std::{
    collections::HashMap,
    fs, mem,
    path::{Path, PathBuf},
};

use crate::{
    ast::{Ast, AstDef, AstExpr, AstMacro},
    errors::MACRO_LOOP,
    MAX_MACRO_DEPTH,
};
use rayon::prelude::*;
use regex::Regex;

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
        self.defs = self
            .defs
            .into_iter()
            .filter_map(|def| match macro_env.try_push(def) {
                Ok(_) => None,
                Err(def) => Some(def.reconstruct(&|e| e.try_expand_macros(&macro_env))),
            })
            .collect::<Result<_, _>>()?;
        Ok(self)
    }
}

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

    // Will either add a macrodef to the environment, pop a macrodef from the environment.
    // If def isn't a macro then it will be returned as in the error
    pub fn try_push(&mut self, def: AstDef<'a>) -> Result<(), AstDef<'a>> {
        match def {
            AstDef::MacroDef(m) => match m {
                AstMacro::ImportFile(_) => Ok(()),
                AstMacro::Inliner(name, args, body) => {
                    self.functions.insert(name, (args, body));
                    Ok(())
                }
                AstMacro::Replacer(macroval, expr) => {
                    self.replacers.insert(macroval, expr);
                    Ok(())
                }
                AstMacro::Undef(macroval) => {
                    let replacer_macro = AstExpr::MacroVal(macroval);
                    self.replacers.remove(&replacer_macro);
                    self.functions.remove(&macroval);
                    Ok(())
                }
            },
            _ => Err(def),
        }
    }
}

impl<'a> AstExpr<'a> {
    fn try_expand_macros(self, macro_env: &MacroEnv<'a>) -> Result<AstExpr<'a>, String> {
        let macroname = match self {
            AstExpr::MacroVal(name, ..) => name,
            AstExpr::Call(name, ..) if name.starts_with('\'') => name,
            _ => return Ok(self),
        };
        self.try_expand_macros_recursive(macro_env, 0)
            .map_err(|mut s| {
                if s.starts_with(MACRO_LOOP) {
                    s = format!(
                        "Recursive macro, depth {} exceeded on {}",
                        MAX_MACRO_DEPTH, &macroname
                    );
                }
                s
            })
    }

    fn try_expand_macros_recursive(
        self,
        macro_env: &MacroEnv<'a>,
        depth: u32,
    ) -> Result<AstExpr<'a>, String> {
        if depth > MAX_MACRO_DEPTH {
            return Err(MACRO_LOOP.to_string());
        }

        match &self {
            AstExpr::MacroVal(name) => macro_env
                .replacers
                .get(&self)
                .ok_or(format!("Macro {} not found", name))
                .cloned()?
                .try_expand_macros_recursive(macro_env, depth + 1),
            AstExpr::Call(name, args) if name.starts_with('\'') => {
                let (formals, body) = macro_env
                    .functions
                    .get(name)
                    .ok_or(format!("Inline Function {} not found", name))?;
                let argmap = formals
                    .iter()
                    .zip(args)
                    .collect::<HashMap<&AstExpr, &AstExpr>>();
                let newbody = body.clone().reconstruct(&|e| match &argmap.get(&e) {
                    Some(&v) => Ok(v.clone()),
                    None => Ok(e),
                })?;
                newbody.try_expand_macros_recursive(macro_env, depth + 1)
            }
            _ => Ok(self),
        }
    }
}
