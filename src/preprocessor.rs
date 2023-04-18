use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use crate::{
    ast::{Ast, Def, Exp},
    env::{Name, Tokens},
};

const MAX_MACRO_DEPTH: u32 = 15;

enum MacroError {
    RecursiveDepthError,
    MismatchError(Name, usize, usize),
    NotAnInliner(Name),
    UnboundMacro(Name),
    ShouldntBePtr(Name, Name, Name),
    ShouldBePtr(Name, Name, Exp),
}

enum CodeCollectError {
    CannotOpen(String),
    AlreadyIncluded,
}

use regex::Regex;
use CodeCollectError::*;
use MacroError::*;

// Collect code

type IncludedFileEnv = HashSet<u64>;

// Returns true if filename was present in env
fn contains_or_add(env: &mut IncludedFileEnv, filename: &str) -> bool {
    let mut hasher = DefaultHasher::new();
    filename.trim().hash(&mut hasher);
    let filehash = hasher.finish();
    !env.insert(filehash)
}

pub fn collect_code(entry_filename: &str, dirs: &[&str]) -> Result<String, String> {
    let import_regex = Regex::new(r###"#\(import\s+"\s*(.*)\s*"\)"###).unwrap();

    collect_code_recursive(
        entry_filename,
        &import_regex,
        dirs,
        &mut IncludedFileEnv::new(),
    )
    .map_err(|e| match e {
        CannotOpen(badfile) => format!("Failed to open file '{}'", badfile),
        AlreadyIncluded => {
            panic!("Compiler bug: weird already included {}", entry_filename)
        }
    })
}

fn collect_code_recursive(
    filename: &str,
    import_re: &Regex,
    dirs: &[&str],
    included: &mut IncludedFileEnv,
) -> Result<String, CodeCollectError> {
    if contains_or_add(included, filename) {
        return Err(AlreadyIncluded);
    }

    let contents = try_open(filename, dirs)?;

    let mut imported = import_re
        .captures_iter(&contents)
        .map(|c| c.iter().nth(1).unwrap().unwrap().as_str())
        .filter_map(
            |f| match collect_code_recursive(f, import_re, dirs, included) {
                Err(AlreadyIncluded) => None,
                e @ Err(CannotOpen(..)) => Some(e),
                ok => Some(ok),
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    let mut newcontents = String::new();
    for section in import_re.split(&contents) {
        newcontents += section;
        if let Some(imported_contents) = imported.next() {
            newcontents += &imported_contents;
        }
    }

    Ok(newcontents)
}

fn try_open(filename: &str, dirs: &[&str]) -> Result<String, CodeCollectError> {
    for dir in [""].iter().chain(dirs) {
        let mut path = PathBuf::from(dir);
        path.push(filename);
        if let Ok(contents) = fs::read_to_string(path) {
            return Ok(contents);
        }
    }
    Err(CannotOpen(filename.to_string()))
}

// Ast-level macro expansion

pub type MacroEnv = HashMap<Name, (Vec<Name>, Exp)>;

pub fn preprocessor(ast: Ast, tokens: &Tokens) -> Result<Ast, String> {
    let mut macros = MacroEnv::new();
    ast.into_iter()
        .filter_map(|def| match def {
            Def::Alias(name, exp) => {
                macros.insert(name, (vec![], exp));
                None
            }
            Def::Inline(name, params, exp) => {
                macros.insert(name, (params, exp));
                None
            }
            Def::Undef(name) => {
                macros.remove(&name);
                None
            }
            Def::Import(..) => Some(Ok(def)),
            _ => Some(def.expand_macros(&macros, tokens)),
        })
        .collect::<Result<_, _>>()
}

impl Def {
    fn expand_macros(self, macros: &MacroEnv, tokens: &Tokens) -> Result<Self, String> {
        let newdef = match self {
            Def::Define(n, a, e) => Def::Define(n, a, e.expand_macros(macros, tokens)?),
            Def::Val(n, e) => Def::Val(n, e.expand_macros(macros, tokens)?),
            Def::CheckAssert(e) => Def::CheckAssert(e.expand_macros(macros, tokens)?),
            Def::CheckExpect(l, r) => Def::CheckExpect(
                l.expand_macros(macros, tokens)?,
                r.expand_macros(macros, tokens)?,
            ),
            Def::Exp(e) => Def::Exp(e.expand_macros(macros, tokens)?),
            Def::Alias(..) | Def::Undef(..) | Def::Inline(..) | Def::Import(..) => unreachable!(),
        };
        Ok(newdef)
    }
}

impl Exp {
    fn expand_macros(self, macros: &MacroEnv, tokens: &Tokens) -> Result<Self, String> {
        let mut formals = MacroEnv::new();
        let res = expand_macros(self, macros, &mut formals, tokens, 0);
        match res {
            Ok(e) => Ok(e),
            Err(RecursiveDepthError) => Err(format!(
                "Preprocessor recursive depth {} exceeded",
                MAX_MACRO_DEPTH
            )),
            Err(MismatchError(name, expected, got)) => Err(format!(
                "Inline argument mismatch on macro ({} ...) expected {} args, got {}",
                tokens.translate(&name),
                expected,
                got
            )),
            Err(NotAnInliner(name)) => Err(format!(
                "Macro {} cannot be used as inline ('{} ...)",
                tokens.translate(&name),
                tokens.translate(&name),
            )),
            Err(UnboundMacro(name)) => {
                Err(format!("Macro {} is not bound", tokens.translate(&name),))
            }
            Err(ShouldntBePtr(name, expected, got)) => Err(format!(
                "In macro ({} ...) got unexpected pointer argument '{}]' for param '{}'",
                tokens.translate(&name),
                tokens.translate(&got),
                tokens.translate(&expected),
            )),
            Err(ShouldBePtr(name, expected, got)) => Err(format!(
                "In macro ({} ...) got unexpected non-pointer argument '{}' for pointer param '{}]'",
                tokens.translate(&name),
                got.to_string(tokens),
                tokens.translate(&expected),
            )),
        }
    }
}

fn expand_macros(
    exp: Exp,
    macros: &MacroEnv,
    formals: &mut MacroEnv,
    tokens: &Tokens,
    depth: u32,
) -> Result<Exp, MacroError> {
    if depth > MAX_MACRO_DEPTH {
        return Err(RecursiveDepthError);
    }
    match exp {
        Exp::Var(name, None) if formals.contains_key(&name) => {
            let (_, argbody) = formals.get(&name).unwrap();
            Ok(argbody.clone())
        }
        Exp::Var(name, Some(i)) if formals.contains_key(&name) => {
            let (_, argbody) = formals.get(&name).unwrap();
            if let Exp::Var(newname, ..) = argbody {
                expand_macros(Exp::Var(*newname, Some(i)), macros, formals, tokens, depth)
            } else {
                panic!()
            }
        }
        Exp::Var(name, None) if macros.contains_key(&name) => {
            let (params, macrobody) = macros.get(&name).unwrap();
            if !params.is_empty() {
                Err(NotAnInliner(name))
            } else {
                Ok(expand_macros(
                    macrobody.clone(),
                    macros,
                    formals,
                    tokens,
                    depth + 1,
                )?)
            }
        }
        Exp::Apply(name, args) if macros.contains_key(&name) => {
            let (params, macrobody) = macros.get(&name).unwrap();
            if params.len() != args.len() {
                Err(MismatchError(name, params.len(), args.len()))
            } else {
                let oldargs = params
                    .iter()
                    .zip(args)
                    .map(|(param, arg)| match &arg {
                        Exp::Var(n, None)
                            if tokens.translate(n).ends_with('[')
                                && !tokens.translate(param).ends_with('[') =>
                        {
                            Err(ShouldntBePtr(name, *param, *n))
                        }
                        Exp::Var(n, None)
                            if tokens.translate(param).ends_with('[')
                                && !tokens.translate(n).ends_with('[') =>
                        {
                            Err(ShouldBePtr(name, *param, arg))
                        }
                        Exp::Var(n, Some(..))
                            if tokens.translate(param).ends_with('[')
                                && tokens.translate(n).ends_with('[') =>
                        {
                            Err(ShouldBePtr(name, *param, arg))
                        }
                        _ => Ok((param, arg)),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .map(|(param, arg)| (param, formals.insert(*param, (vec![], arg))))
                    .collect::<Vec<_>>();
                let finalexp =
                    expand_macros(macrobody.clone(), macros, formals, tokens, depth + 1)?;
                for (param, arg) in oldargs {
                    if let Some(oldarg) = arg {
                        formals.insert(*param, oldarg);
                    } else {
                        formals.remove(param);
                    }
                }
                Ok(finalexp)
            }
        }
        Exp::Var(n, ..) if tokens.translate(&n).starts_with('\'') => Err(UnboundMacro(n)),
        Exp::Apply(n, ..) if tokens.translate(&n).starts_with('\'') => Err(UnboundMacro(n)),
        _ => expand_macros_in_children(exp, macros, formals, tokens, depth),
    }
}

fn expand_macros_in_children(
    exp: Exp,
    macros: &MacroEnv,
    formals: &mut MacroEnv,
    tokens: &Tokens,
    depth: u32,
) -> Result<Exp, MacroError> {
    match exp {
        Exp::Literal(_) => Ok(exp),
        Exp::Var(_, None) => Ok(exp),
        Exp::Var(n, Some(mut e)) => {
            *e = expand_macros(*e, macros, formals, tokens, depth)?;
            Ok(Exp::Var(n, Some(e)))
        }
        Exp::Set(n, Some(mut e), mut v) => {
            *e = expand_macros(*e, macros, formals, tokens, depth)?;
            *v = expand_macros(*v, macros, formals, tokens, depth)?;
            Ok(Exp::Set(n, Some(e), v))
        }
        Exp::Set(n, None, mut v) => {
            *v = expand_macros(*v, macros, formals, tokens, depth)?;
            Ok(Exp::Set(n, None, v))
        }
        Exp::Binary(p, mut l, mut r) => {
            *l = expand_macros(*l, macros, formals, tokens, depth)?;
            *r = expand_macros(*r, macros, formals, tokens, depth)?;
            Ok(Exp::Binary(p, l, r))
        }
        Exp::Unary(p, mut e) => {
            *e = expand_macros(*e, macros, formals, tokens, depth)?;
            Ok(Exp::Unary(p, e))
        }
        Exp::Apply(n, es) => Ok(Exp::Apply(
            n,
            es.into_iter()
                .map(|e| expand_macros(e, macros, formals, tokens, depth))
                .collect::<Result<_, _>>()?,
        )),
        Exp::If(mut c, mut t, mut e) => {
            *c = expand_macros(*c, macros, formals, tokens, depth)?;
            *t = expand_macros(*t, macros, formals, tokens, depth)?;
            *e = expand_macros(*e, macros, formals, tokens, depth)?;
            Ok(Exp::If(c, t, e))
        }
        Exp::While(mut g, mut b) => {
            *g = expand_macros(*g, macros, formals, tokens, depth)?;
            *b = expand_macros(*b, macros, formals, tokens, depth)?;
            Ok(Exp::While(g, b))
        }
        Exp::Begin(es) => Ok(Exp::Begin(
            es.into_iter()
                .map(|e| expand_macros(e, macros, formals, tokens, depth))
                .collect::<Result<_, _>>()?,
        )),
        Exp::Match(mut p, cs, mut d) => {
            *p = expand_macros(*p, macros, formals, tokens, depth)?;
            *d = expand_macros(*d, macros, formals, tokens, depth)?;
            Ok(Exp::Match(
                p,
                cs.into_iter()
                    .map(|(l, r)| {
                        Ok((
                            expand_macros(l, macros, formals, tokens, depth)?,
                            expand_macros(r, macros, formals, tokens, depth)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
                d,
            ))
        }
    }
}
