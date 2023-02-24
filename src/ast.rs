use crate::parser::ImpcoreParser;
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

impl<'a> Ast<'a> {
    pub fn from(contents: &str) -> Result<Ast, String> {
        Ok(ImpcoreParser::generate_ast(contents)?
            .preprocess()
            .prepare())
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
