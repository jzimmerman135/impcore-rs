use crate::{
    jit::{codegen, defgen, Compiler, NativeTopLevel},
    parser::{def_parse, expr_parse, *},
};
use inkwell::values::IntValue;
use std::{
    collections::HashSet,
    slice::{Iter, IterMut},
};

#[derive(Clone)]
pub struct Ast<'a>(pub Vec<AstDef<'a>>);

#[derive(Debug, PartialEq, Clone)]
pub enum AstScope {
    Unknown,
    Local,
    Global,
    Param,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstExpr<'a> {
    Literal(i32),
    Variable(&'a str, AstScope),
    Binary(&'a str, Box<AstExpr<'a>>, Box<AstExpr<'a>>),
    Unary(&'a str, Box<AstExpr<'a>>),
    Call(&'a str, Vec<AstExpr<'a>>),
    If(Box<AstExpr<'a>>, Box<AstExpr<'a>>, Box<AstExpr<'a>>),
    While(Box<AstExpr<'a>>, Box<AstExpr<'a>>),
    Begin(Vec<AstExpr<'a>>),
    Assign(&'a str, Box<AstExpr<'a>>, AstScope),
    Error,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstDef<'a> {
    TopLevelExpr(AstExpr<'a>),
    Function(&'a str, Vec<&'a str>, HashSet<&'a str>, AstExpr<'a>),
    Global(&'a str, AstExpr<'a>),
    CheckExpect(AstExpr<'a>, AstExpr<'a>, &'a str),
    CheckAssert(AstExpr<'a>, &'a str),
    CheckError(AstExpr<'a>, &'a str),
}

pub trait AstChildren<'a> {
    fn children(&self) -> Vec<&AstExpr<'a>>;
    fn children_mut(&mut self) -> Vec<&mut AstExpr<'a>>;
}

impl<'a> AstChildren<'a> for AstDef<'a> {
    fn children(&self) -> Vec<&AstExpr<'a>> {
        match self {
            Self::Function(_, _, _, body) => vec![body],
            Self::TopLevelExpr(body) => vec![body],
            Self::Global(_, body) => vec![body],
            Self::CheckAssert(body, _) => vec![body],
            Self::CheckExpect(lhs, rhs, _) => vec![lhs, rhs],
            _ => unreachable!(),
        }
    }

    fn children_mut(&mut self) -> Vec<&mut AstExpr<'a>> {
        match self {
            Self::Function(_, _, _, body) => vec![body],
            Self::TopLevelExpr(body) => vec![body],
            Self::Global(_, body) => vec![body],
            Self::CheckAssert(body, _) => vec![body],
            Self::CheckExpect(lhs, rhs, _) => vec![lhs, rhs],
            _ => unreachable!(),
        }
    }
}

impl<'a> AstChildren<'a> for AstExpr<'a> {
    fn children_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Self::Error | Self::Variable(..) | Self::Literal(..) => vec![],
            Self::Binary(_, lhs, rhs) => vec![lhs, rhs],
            Self::Unary(_, body) | Self::Assign(_, body, _) => vec![body],
            Self::While(cond, body) => vec![cond, body],
            Self::Begin(exprs) | Self::Call(_, exprs) => exprs.iter_mut().collect::<Vec<_>>(),
            Self::If(c, t, f) => {
                vec![c, t, f]
            }
        }
    }

    fn children(&self) -> Vec<&Self> {
        match self {
            Self::Error | Self::Variable(..) | Self::Literal(..) => vec![],
            Self::Binary(_, lhs, rhs) => vec![lhs, rhs],
            Self::Unary(_, body) | Self::Assign(_, body, _) => vec![body],
            Self::While(cond, body) => vec![cond, body],
            Self::Begin(exprs) | Self::Call(_, exprs) => exprs.iter().collect::<Vec<_>>(),
            Self::If(c, t, f) => {
                vec![c, t, f]
            }
        }
    }
}

impl<'a> AstDef<'a> {
    pub fn parse(def: Pair<Rule>) -> AstDef {
        match def.as_rule() {
            Rule::tle => AstDef::TopLevelExpr(AstExpr::parse(def.into_inner().next().unwrap())),
            Rule::val => def_parse::parse_val(def),
            Rule::check_assert => def_parse::parse_check_assert(def),
            Rule::check_expect => def_parse::parse_check_expect(def),
            Rule::check_error => def_parse::parse_check_error(def),
            Rule::define => def_parse::parse_define(def),
            _ => unreachable!("got unreachable def {:?}", def.as_rule()),
        }
    }

    pub fn defgen(&self, compiler: &mut Compiler<'a>) -> Result<NativeTopLevel<'a>, String> {
        Ok(match self {
            Self::Function(name, params, _, body) => NativeTopLevel::FunctionDef(
                defgen::defgen_function(name, params, body, compiler)?,
                name,
            ),
            Self::TopLevelExpr(body) => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_anonymous(body, compiler)?)
            }
            Self::CheckAssert(body, contents) => {
                NativeTopLevel::CheckAssert(defgen::defgen_anonymous(body, compiler)?, contents)
            }
            Self::CheckExpect(lhs, rhs, contents) => NativeTopLevel::CheckExpect(
                defgen::defgen_anonymous(lhs, compiler)?,
                defgen::defgen_anonymous(rhs, compiler)?,
                contents,
            ),
            Self::Global(name, value) => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_global(name, value, compiler)?)
            }
            _ => unreachable!("Unreachable defgen {:?}", self),
        })
    }

    pub fn apply_to_children<F>(&mut self, apply: &mut F) -> Result<(), String>
    where
        F: FnMut(&mut AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children_mut() {
            child.apply_mut(apply)?;
        }
        Ok(())
    }

    pub fn for_each_child<F>(&self, apply: &mut F) -> Result<(), String>
    where
        F: FnMut(&AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children() {
            child.for_each(apply)?;
        }
        Ok(())
    }
}

impl<'a> AstExpr<'a> {
    pub fn parse(expr: Pair<Rule>) -> AstExpr {
        match expr.as_rule() {
            Rule::literal => expr_parse::parse_literal(expr),
            Rule::variable => expr_parse::parse_variable(expr),
            Rule::binary => expr_parse::parse_binary(expr),
            Rule::unary => expr_parse::parse_unary(expr),
            Rule::user => expr_parse::parse_call(expr),
            Rule::ifx => expr_parse::parse_if(expr),
            Rule::whilex => expr_parse::parse_while(expr),
            Rule::begin => expr_parse::parse_begin(expr),
            Rule::set => expr_parse::parse_set(expr),
            Rule::error => AstExpr::Error,
            _ => unreachable!("got unreachable expr {:?}", expr.as_rule()),
        }
    }

    pub fn codegen(&self, compiler: &mut Compiler<'a>) -> Result<IntValue<'a>, String> {
        match self {
            Self::Binary(op, lhs, rhs) => codegen::codegen_binary(op, lhs, rhs, compiler),
            Self::Unary(op, body) => codegen::codegen_unary(op, body, compiler),
            Self::If(cond, t, f) => codegen::codegen_if(cond, t, f, compiler),
            Self::While(cond, body) => codegen::codegen_while(cond, body, compiler),
            Self::Call(name, args) => codegen::codegen_call(name, args, compiler),
            Self::Literal(value) => codegen::codegen_literal(*value, compiler),
            Self::Variable(name, AstScope::Param) => codegen::codegen_formal(name, compiler),
            Self::Variable(name, ..) => codegen::codegen_variable(name, compiler),
            Self::Error => codegen::codegen_literal(1, compiler),
            Self::Begin(exprs) => codegen::codegen_begin(exprs, compiler),
            _ => unimplemented!("Unimplemented codegen {:?}", self),
        }
    }

    pub fn apply_mut<F>(&mut self, apply: &mut F) -> Result<(), String>
    where
        F: FnMut(&mut AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children_mut() {
            child.apply_mut(apply)?;
        }
        apply(self)
    }

    pub fn for_each<F>(&self, predicate: &mut F) -> Result<(), String>
    where
        F: FnMut(&AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children() {
            child.for_each(predicate)?;
        }
        predicate(self)
    }
}

impl<'a> Ast<'a> {
    pub fn iter(&self) -> Iter<'_, AstDef> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, AstDef<'a>> {
        self.0.iter_mut()
    }
}

pub mod static_analysis {
    use std::collections::HashSet;

    use super::*;

    pub fn rebuild(mut ast: Ast) -> Result<Ast, String> {
        squash_globals(&mut ast);
        build_scopes(&mut ast)?;
        Ok(ast)
    }

    pub fn get_globals<'a>(ast: &Ast<'a>) -> HashSet<&'a str> {
        ast.0
            .iter()
            .filter_map(|e| match e {
                AstDef::Global(name, ..) => Some(&**name),
                _ => None,
            })
            .collect()
    }

    /// leaves unbounded variables scoped as `AstScope::Unknown`
    ///
    /// `AstScope::Local` means stack allocated for a function call,
    /// `AstScope::Param` params means SSA,
    /// `AstScope::Global` means constant pointer to heap variable,
    pub fn build_scopes(ast: &mut Ast) -> Result<(), String> {
        let globals = get_globals(ast);
        for def in ast.iter_mut() {
            match def {
                AstDef::Function(_, params, locals, body) => {
                    locals.clear();

                    // so we know where to store it
                    body.apply_mut(&mut |expr| {
                        match expr {
                            AstExpr::Assign(name, value, AstScope::Unknown)
                                if params.contains(name) =>
                            {
                                locals.insert(name);
                                *expr = AstExpr::Assign(name, value.to_owned(), AstScope::Local);
                            }
                            AstExpr::Assign(name, value, AstScope::Unknown)
                                if globals.contains(name) =>
                            {
                                *expr = AstExpr::Assign(name, value.to_owned(), AstScope::Global);
                            }
                            _ => (),
                        };
                        Ok(())
                    })?;

                    // so we know where to look it up
                    body.apply_mut(&mut |expr| {
                        match expr {
                            AstExpr::Variable(name, AstScope::Unknown) if locals.contains(name) => {
                                *expr = AstExpr::Variable(name, AstScope::Local);
                            }
                            AstExpr::Variable(name, AstScope::Unknown) if params.contains(name) => {
                                *expr = AstExpr::Variable(name, AstScope::Param);
                            }
                            AstExpr::Variable(name, AstScope::Unknown)
                                if globals.contains(name) =>
                            {
                                *expr = AstExpr::Variable(name, AstScope::Global);
                            }
                            _ => (),
                        };
                        Ok(())
                    })?;
                }
                _ => def.apply_to_children(&mut |expr| {
                    // we now know where to store these
                    match expr {
                        AstExpr::Assign(name, value, AstScope::Unknown)
                            if globals.contains(name) =>
                        {
                            *expr = AstExpr::Assign(name, value.to_owned(), AstScope::Global);
                        }
                        AstExpr::Variable(name, AstScope::Unknown) if globals.contains(name) => {
                            *expr = AstExpr::Variable(name, AstScope::Global);
                        }
                        _ => (),
                    };
                    Ok(())
                })?,
            }
        }
        Ok(())
    }

    /// Moves global variable definitions to the start of execution, replaces
    /// them with an assignment
    pub fn squash_globals(ast: &mut Ast) {
        use std::mem;
        use AstExpr::Assign;
        use AstScope::Global as GlobalScope;

        let defs = mem::take(&mut ast.0);
        let mut global_names = HashSet::new();
        let (globals, others): (Vec<_>, Vec<_>) = defs.into_iter().partition(|e| match e {
            AstDef::Global(name, ..) => global_names.insert(&**name),
            _ => false,
        });
        let mut defs = globals;
        defs.append(
            &mut others
                .into_iter()
                .map(|e| match &e {
                    AstDef::Global(n, v) => {
                        AstDef::TopLevelExpr(Assign(n, Box::new(v.to_owned()), GlobalScope))
                    }
                    _ => e,
                })
                .collect(),
        );
        *ast = Ast(defs)
    }
}
