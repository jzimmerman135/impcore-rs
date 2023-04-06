use crate::parser::ImpcoreParser;
use std::{
    mem,
    slice::{Iter, IterMut},
};

#[derive(Clone)]
pub struct Ast<'a> {
    pub defs: Vec<AstDef<'a>>,
}

pub struct AstDefWrap<'a>(AstDef<'a>, (u32, u32));

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    Match(
        Box<AstExpr<'a>>,
        Vec<(AstExpr<'a>, AstExpr<'a>)>,
        Box<AstExpr<'a>>,
    ),
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
    DeclareGlobal(&'a str, AstType),
    MacroDef(AstMacro<'a>),
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum AstMacro<'a> {
    ImportFile(&'a str),
    Replacer(AstExpr<'a>, AstExpr<'a>),
    Inliner(&'a str, Vec<AstExpr<'a>>, AstExpr<'a>),
    Undef(&'a str),
}

impl<'a> Ast<'a> {
    pub fn from(contents: &str) -> Result<Ast, String> {
        ImpcoreParser::generate_ast(contents)
    }
}

pub trait AstChildren<'a> {
    fn children(&self) -> Vec<&AstExpr<'a>>;
}

impl<'a> AstChildren<'a> for AstDef<'a> {
    fn children(&self) -> Vec<&AstExpr<'a>> {
        match self {
            Self::Function(_, _, body) => vec![body],
            Self::TopLevelExpr(body) => vec![body],
            Self::Global(_, body, _) => vec![body],
            Self::CheckAssert(body, _) | Self::CheckError(body, _) => vec![body],
            Self::CheckExpect(lhs, rhs, _) => vec![lhs, rhs],
            Self::MacroDef(..) | Self::ImportLib(..) | Self::DeclareGlobal(..) => {
                vec![]
            }
        }
    }
}

impl<'a> AstChildren<'a> for AstExpr<'a> {
    fn children(&self) -> Vec<&Self> {
        fn matchx_children<'a, 'b>(
            scrut: &'b AstExpr<'a>,
            arms: &'b [(AstExpr<'a>, AstExpr<'a>)],
            default: &'b AstExpr<'a>,
        ) -> Vec<&'b AstExpr<'a>> {
            let mut children = vec![scrut, default];
            children.append(
                &mut arms
                    .iter()
                    .flat_map(|(pred, then)| vec![pred, then])
                    .collect(),
            );
            children
        }

        match self {
            Self::Variable(_, Some(body)) => vec![body],
            Self::Binary(_, lhs, rhs) => vec![lhs, rhs],
            Self::Unary(_, body) | Self::Assign(_, body, None) => vec![body],
            Self::While(cond, body) => vec![cond, body],
            Self::Assign(_, body, Some(index)) => vec![body, index],
            Self::Begin(exprs) | Self::Call(_, exprs) => exprs.iter().collect::<Vec<_>>(),
            Self::Match(scrut, arms, default) => matchx_children(scrut, arms, default),
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
    pub fn is_test(&self) -> bool {
        matches!(self, AstDef::CheckAssert(..) | AstDef::CheckExpect(..))
    }

    pub fn contains<F>(&self, predicate: &mut F) -> bool
    where
        F: FnMut(&AstExpr<'a>) -> bool,
    {
        for child in self.children() {
            if child.contains(predicate) {
                return true;
            }
        }
        false
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

    // use this instead of children mut, very powerful function
    pub fn reconstruct<F>(mut self, construct: &F) -> Result<Self, String>
    where
        F: Fn(AstExpr<'a>) -> Result<AstExpr<'a>, String>,
    {
        match &mut self {
            Self::CheckAssert(body, _)
            | Self::CheckError(body, _)
            | Self::Global(_, body, _)
            | Self::Function(_, _, body)
            | Self::TopLevelExpr(body) => *body = mem::take(body).reconstruct(construct)?,
            Self::CheckExpect(lhs, rhs, _) => {
                *lhs = mem::take(lhs).reconstruct(construct)?;
                *rhs = mem::take(rhs).reconstruct(construct)?;
            }
            Self::MacroDef(..) | Self::ImportLib(..) | Self::DeclareGlobal(..) => {}
        };
        Ok(self)
    }
}

impl<'a> AstExpr<'a> {
    pub fn reconstruct<F>(mut self, construct: &F) -> Result<Self, String>
    where
        F: Fn(AstExpr<'a>) -> Result<AstExpr<'a>, String>,
    {
        self = construct(self)?;
        match &mut self {
            Self::Unary(_, body) | Self::Assign(_, body, None) | Self::Variable(_, Some(body)) => {
                *body = Box::new(mem::take(body).reconstruct(construct)?);
            }
            Self::Binary(_, x, y) | Self::While(x, y) | Self::Assign(_, x, Some(y)) => {
                *x = Box::new(mem::take(x).reconstruct(construct)?);
                *y = Box::new(mem::take(y).reconstruct(construct)?);
            }
            Self::Begin(exprs) | Self::Call(_, exprs) => {
                *exprs = mem::take(exprs)
                    .into_iter()
                    .map(|e| e.reconstruct(construct))
                    .collect::<Result<Vec<_>, String>>()?;
            }
            Self::If(cond, truecase, falsecase) => {
                *cond = Box::new(mem::take(cond).reconstruct(construct)?);
                *truecase = Box::new(mem::take(truecase).reconstruct(construct)?);
                *falsecase = Box::new(mem::take(falsecase).reconstruct(construct)?);
            }
            Self::Match(cond, arms, default) => {
                *cond = Box::new(mem::take(cond).reconstruct(construct)?);
                *arms = mem::take(arms)
                    .into_iter()
                    .map(|(e1, e2)| Ok((e1.reconstruct(construct)?, e2.reconstruct(construct)?)))
                    .collect::<Result<Vec<_>, String>>()?;
                *default = Box::new(mem::take(default).reconstruct(construct)?);
            }
            Self::MacroVal(_)
            | Self::Error
            | Self::Variable(_, None)
            | Self::Literal(..)
            | Self::Pointer(..) => {}
        };
        Ok(self)
    }
}

impl<'a> Default for AstExpr<'a> {
    fn default() -> Self {
        AstExpr::Error
    }
}

impl<'a> Default for AstMacro<'a> {
    fn default() -> Self {
        AstMacro::Replacer(AstExpr::Error, AstExpr::Error)
    }
}

impl<'a> AstExpr<'a> {
    pub fn contains<F>(&self, predicate: &mut F) -> bool
    where
        F: FnMut(&AstExpr<'a>) -> bool,
    {
        if predicate(self) {
            return true;
        }
        for child in self.children() {
            if child.contains(predicate) {
                return true;
            }
        }
        false
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

pub fn get_variable_name<'a>(exp: &AstExpr<'a>) -> &'a str {
    if let AstExpr::Pointer(name) | AstExpr::Variable(name, ..) = exp {
        name
    } else {
        panic!("Called get variable name on non-variable {:?}", exp)
    }
}
