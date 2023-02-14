use crate::{
    jit::{codegen, defgen, Compiler, NativeTopLevel},
    parser::{def_parse, expr_parse, *},
};
use inkwell::values::IntValue;
use std::slice::{Iter, IterMut};
pub struct Ast<'a>(pub Vec<AstDef<'a>>);

#[derive(Debug, PartialEq, Clone)]
pub enum AstScope {
    Unknown,
    Local,
    Global,
    Param,
    Constant,
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

#[derive(Debug, PartialEq)]
pub enum AstDef<'a> {
    TopLevelExpr(AstExpr<'a>),
    Function(&'a str, Vec<&'a str>, Vec<&'a str>, AstExpr<'a>),
    Global(&'a str, AstExpr<'a>, AstScope),
    CheckExpect(AstExpr<'a>, AstExpr<'a>, &'a str),
    CheckAssert(AstExpr<'a>, &'a str),
    CheckError(AstExpr<'a>, &'a str),
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
            Self::Global(name, value, _) => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_global(name, value, compiler)?)
            }
            _ => unreachable!("Unreachable defgen {:?}", self),
        })
    }

    pub fn children_mut(&mut self) -> Vec<&mut AstExpr<'a>> {
        match self {
            Self::Function(_, _, _, body) => body.children_mut(),
            Self::TopLevelExpr(body) => body.children_mut(),
            Self::Global(_, body, _) => body.children_mut(),
            Self::CheckAssert(body, _) => body.children_mut(),
            Self::CheckExpect(lhs, rhs, _) => {
                let mut lchildren = lhs.children_mut();
                lchildren.append(&mut rhs.children_mut());
                lchildren
            }
            _ => unreachable!(),
        }
    }

    pub fn children(&self) -> Vec<&AstExpr<'a>> {
        match self {
            Self::Function(_, _, _, body) => body.children(),
            Self::TopLevelExpr(body) => body.children(),
            Self::Global(_, body, _) => body.children(),
            Self::CheckAssert(body, _) => body.children(),
            Self::CheckExpect(lhs, rhs, _) => {
                let mut lchildren = lhs.children();
                lchildren.append(&mut rhs.children());
                lchildren
            }
            _ => unreachable!(),
        }
    }

    pub fn apply_to_children<F>(&mut self, predicate: &mut F) -> Result<(), String>
    where
        F: FnMut(&mut AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children_mut() {
            child.apply_mut(predicate)?;
        }
        Ok(())
    }

    pub fn for_each_child<F>(&self, predicate: &mut F) -> Result<(), String>
    where
        F: FnMut(&AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children() {
            child.for_each(predicate)?;
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

    pub fn children_mut(&mut self) -> Vec<&mut Self> {
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

    pub fn children(&self) -> Vec<&Self> {
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

    pub fn apply_mut<F>(&mut self, apply: &mut F) -> Result<(), String>
    where
        F: FnMut(&mut AstExpr<'a>) -> Result<(), String>,
    {
        // leafs have no children
        for child in self.children_mut() {
            child.apply_mut(apply)?;
        }
        apply(self)
    }

    pub fn for_each<F>(&self, predicate: &mut F) -> Result<(), String>
    where
        F: FnMut(&AstExpr<'a>) -> Result<(), String>,
    {
        // since error, variable, literal never have children
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

    pub fn get_globals<'a>(ast: &Ast<'a>) -> HashSet<&'a str> {
        ast.0
            .iter()
            .filter_map(|e| match e {
                AstDef::Global(name, ..) => Some(&**name),
                _ => None,
            })
            .collect()
    }

    pub fn build_scopes(ast: &mut Ast) -> Result<(), String> {
        let globals = get_globals(ast);
        let mut mutables = HashSet::new();

        for root in ast.iter_mut() {
            match root {
                AstDef::Function(_, params, locals, body) => {
                    // find all assignments within function and discover local variables
                    locals.clear();
                    body.apply_mut(&mut |e| {
                        Ok(match e {
                            AstExpr::Assign(name, value, AstScope::Unknown)
                                if params.contains(name) =>
                            {
                                if !locals.contains(name) {
                                    locals.push(&**name);
                                }
                                *e = AstExpr::Assign(name, value.to_owned(), AstScope::Local);
                            }
                            AstExpr::Assign(name, value, AstScope::Unknown)
                                if globals.contains(name) =>
                            {
                                mutables.insert(&**name);
                                *e = AstExpr::Assign(name, value.to_owned(), AstScope::Global);
                            }
                            _ => (),
                        })
                    })?;

                    // add scopes to locals, param and global variables
                    body.apply_mut(&mut |e| {
                        Ok(match e {
                            AstExpr::Variable(name, AstScope::Unknown) if locals.contains(name) => {
                                *e = AstExpr::Variable(name, AstScope::Local);
                            }
                            AstExpr::Variable(name, AstScope::Unknown) if params.contains(name) => {
                                *e = AstExpr::Variable(name, AstScope::Param);
                            }
                            AstExpr::Variable(name, AstScope::Unknown)
                                if globals.contains(name) =>
                            {
                                *e = AstExpr::Variable(name, AstScope::Global);
                            }
                            _ => (),
                        })
                    })?;
                }
                // non functions have only global variables, but first find mutables
                _ => root.apply_to_children(&mut |e| {
                    Ok(match e {
                        AstExpr::Assign(name, value, AstScope::Unknown)
                            if globals.contains(name) =>
                        {
                            mutables.insert(&**name);
                            *e = AstExpr::Assign(name, value.to_owned(), AstScope::Global);
                        }
                        AstExpr::Variable(name, AstScope::Unknown) if globals.contains(name) => {
                            *e = AstExpr::Variable(name, AstScope::Global);
                        }
                        _ => (),
                    })
                })?,
            }
        }

        for root in ast.iter_mut() {
            match root {
                AstDef::Global(name, value, _) if !mutables.contains(name) => {
                    *root = AstDef::Global(name, value.to_owned(), AstScope::Constant);
                }
                AstDef::Global(name, value, _) => {
                    *root = AstDef::Global(name, value.to_owned(), AstScope::Global);
                }
                _ => continue,
            }
        }

        for root in ast.iter_mut() {
            root.apply_to_children(&mut |e| {
                Ok(match e {
                    AstExpr::Variable(name, AstScope::Global)
                        if globals.contains(name) && !mutables.contains(name) =>
                    {
                        *e = AstExpr::Variable(name, AstScope::Constant);
                    }
                    _ => (),
                })
            })?;
        }
        Ok(())
    }
}
