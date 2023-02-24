use crate::{
    jit::{codegen, defgen, Compiler, NativeTopLevel},
    parser::{def_parse, expr_parse, *},
};

use inkwell::values::IntValue;
use std::slice::{Iter, IterMut};

#[derive(Clone)]
pub struct Ast<'a> {
    pub defs: Vec<AstDef<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstExpr<'a> {
    Literal(i32),
    Variable(&'a str, Option<Box<AstExpr<'a>>>),
    Pointer(&'a str),
    Binary(&'a str, Box<AstExpr<'a>>, Box<AstExpr<'a>>),
    Unary(&'a str, Box<AstExpr<'a>>),
    Call(&'a str, Vec<AstExpr<'a>>),
    If(Box<AstExpr<'a>>, Box<AstExpr<'a>>, Box<AstExpr<'a>>),
    While(Box<AstExpr<'a>>, Box<AstExpr<'a>>),
    Begin(Vec<AstExpr<'a>>),
    Assign(&'a str, Box<AstExpr<'a>>, Option<Box<AstExpr<'a>>>),
    MacroVal(&'a str),
    Error,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AstType {
    Integer,
    Pointer,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstDef<'a> {
    ImportLib(&'a str),
    TopLevelExpr(AstExpr<'a>),
    Function(&'a str, Vec<(&'a str, AstType)>, AstExpr<'a>),
    Global(&'a str, AstExpr<'a>, AstType),
    CheckExpect(AstExpr<'a>, AstExpr<'a>, &'a str),
    CheckAssert(AstExpr<'a>, &'a str),
    CheckError(AstExpr<'a>, &'a str),
    DeclareGlobal(&'a str),
    MacroDef(AstMacro<'a>),
    FreeAll,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstMacro<'a> {
    ImportFile(&'a str),
    Replacer(&'a str, AstExpr<'a>),
    Inliner(&'a str, Vec<AstExpr<'a>>, AstExpr<'a>),
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
            Rule::alloc => def_parse::parse_alloc(def),
            Rule::lib => macro_parse::parse_importlib(def),
            Rule::file => macro_parse::parse_importfile(def),
            Rule::replacer => macro_parse::parse_replacer(def),
            Rule::inliner => macro_parse::parse_inliner(def),
            _ => unreachable!("got unreachable def rule {:?}", def.as_rule()),
        }
    }

    pub fn defgen(&self, compiler: &mut Compiler<'a>) -> Result<NativeTopLevel<'a>, String> {
        compiler.clear_curr_function();
        let native = match self {
            Self::Function(name, params, body) => NativeTopLevel::FunctionDef(
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
            Self::CheckError(..) => todo!(),
            Self::Global(name, value, var_type) => NativeTopLevel::TopLevelExpr(
                defgen::defgen_global(name, value, *var_type, compiler)?,
            ),
            Self::DeclareGlobal(name) => {
                defgen::declare_global(name, compiler);
                NativeTopLevel::Noop
            }
            Self::FreeAll => NativeTopLevel::FreeAll(defgen::defgen_cleanup(compiler)?),
            Self::ImportLib("stdin") => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_stdin(compiler)?)
            }
            Self::ImportLib(name) => {
                return Err(format!("Unbound library {}, got {:?}", name, self))
            }
            _ => unreachable!("unreacheable defgen {:?}", self),
        };
        Ok(native)
    }
}

impl<'a> AstExpr<'a> {
    pub fn parse(expr: Pair<Rule>) -> AstExpr {
        match expr.as_rule() {
            Rule::literal => expr_parse::parse_literal(expr),
            Rule::variable => expr_parse::parse_variable(expr),
            Rule::binary => expr_parse::parse_binary(expr),
            Rule::unary => expr_parse::parse_unary(expr),
            Rule::print => expr_parse::parse_unary(expr),
            Rule::user => expr_parse::parse_call(expr),
            Rule::fgetc => AstExpr::Call(expr.as_str(), vec![]),
            Rule::ifx => expr_parse::parse_if(expr),
            Rule::whilex => expr_parse::parse_while(expr),
            Rule::begin => expr_parse::parse_begin(expr),
            Rule::set => expr_parse::parse_set(expr),
            Rule::array_value => expr_parse::parse_indexer(expr),
            Rule::pointer => expr_parse::parse_pointer(expr),
            Rule::error => AstExpr::Error,
            Rule::macroval => macro_parse::parse_macroval(expr),
            Rule::parameter => expr_parse::parse_variable(expr),
            _ => unreachable!("got unreachable expr rule {:?}", expr.as_rule()),
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
            Self::Variable(name, index) => {
                codegen::codegen_variable(name, index.as_deref(), compiler)
            }
            Self::Assign(name, body, index) => {
                codegen::codegen_assign(name, index.as_deref(), body, compiler)
            }
            Self::Error => codegen::codegen_literal(1, compiler),
            Self::Begin(exprs) => codegen::codegen_begin(exprs, compiler),
            _ => unreachable!("cannot codegen from {:?}", self),
        }
    }
}

pub trait AstChildren<'a> {
    fn children(&self) -> Vec<&AstExpr<'a>>;
    fn children_mut(&mut self) -> Vec<&mut AstExpr<'a>>;
}

impl<'a> AstChildren<'a> for AstDef<'a> {
    fn children(&self) -> Vec<&AstExpr<'a>> {
        match self {
            Self::Function(_, _, body) => vec![body],
            Self::TopLevelExpr(body) => vec![body],
            Self::Global(_, body, _) => vec![body],
            Self::CheckAssert(body, _) | Self::CheckError(body, _) => vec![body],
            Self::CheckExpect(lhs, rhs, _) => vec![lhs, rhs],
            Self::MacroDef(..) | Self::ImportLib(..) | Self::DeclareGlobal(..) | Self::FreeAll => {
                vec![]
            }
        }
    }

    fn children_mut(&mut self) -> Vec<&mut AstExpr<'a>> {
        match self {
            Self::Function(_, _, body) => vec![body],
            Self::TopLevelExpr(body) => vec![body],
            Self::Global(_, body, _) => vec![body],
            Self::CheckAssert(body, _) | Self::CheckError(body, _) => vec![body],
            Self::CheckExpect(lhs, rhs, _) => vec![lhs, rhs],
            Self::MacroDef(..) | Self::ImportLib(..) | Self::DeclareGlobal(..) | Self::FreeAll => {
                vec![]
            }
        }
    }
}

impl<'a> AstChildren<'a> for AstExpr<'a> {
    fn children_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Self::Variable(_, Some(body)) => vec![body],
            Self::Binary(_, lhs, rhs) => vec![lhs, rhs],
            Self::Unary(_, body) | Self::Assign(_, body, None) => vec![body],
            Self::While(cond, body) => vec![cond, body],
            Self::Assign(_, body, Some(index)) => vec![body, index],
            Self::Begin(exprs) | Self::Call(_, exprs) => exprs.iter_mut().collect::<Vec<_>>(),
            Self::If(c, t, f) => {
                vec![c, t, f]
            }
            Self::MacroVal(_)
            | Self::Error
            | Self::Variable(_, None)
            | Self::Literal(..)
            | Self::Pointer(..) => vec![],
        }
    }

    fn children(&self) -> Vec<&Self> {
        match self {
            Self::Variable(_, Some(body)) => vec![body],
            Self::Binary(_, lhs, rhs) => vec![lhs, rhs],
            Self::Unary(_, body) | Self::Assign(_, body, None) => vec![body],
            Self::While(cond, body) => vec![cond, body],
            Self::Assign(_, body, Some(index)) => vec![body, index],
            Self::Begin(exprs) | Self::Call(_, exprs) => exprs.iter().collect::<Vec<_>>(),
            Self::If(c, t, f) => {
                vec![c, t, f]
            }
            Self::MacroVal(_)
            | Self::Error
            | Self::Variable(_, None)
            | Self::Literal(..)
            | Self::Pointer(..) => vec![],
        }
    }
}

impl<'a> AstDef<'a> {
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
    pub fn apply_mut<F>(&mut self, apply: &mut F) -> Result<(), String>
    where
        F: FnMut(&mut AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children_mut() {
            child.apply_mut(apply)?;
        }
        apply(self)
    }

    pub fn for_each<F>(&self, apply: &mut F) -> Result<(), String>
    where
        F: FnMut(&AstExpr<'a>) -> Result<(), String>,
    {
        for child in self.children() {
            child.for_each(apply)?;
        }
        apply(self)
    }
}

impl<'a> Ast<'a> {
    pub fn iter(&self) -> Iter<'_, AstDef> {
        self.defs.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, AstDef<'a>> {
        self.defs.iter_mut()
    }
}
