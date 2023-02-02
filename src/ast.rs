use std::fmt::Debug;

use crate::jit::{CodeGen, Compiler};
use inkwell::values::IntValue;

// premature optimization on box dyn trait
pub enum AstNode<'a> {
    Literal(Literal),
    Variable(Variable<'a>),
    Binary(Binary<'a>),
    Unary(Unary<'a>),
    Call(Call<'a>),
    Function(Function<'a>),
    If(If<'a>),
    While(While<'a>),
    Begin(Begin<'a>),
    Assign(Assign<'a>),
    NewGlobal(NewGlobal<'a>),
    Error(RuntimeError),
}

impl<'a> CodeGen for AstNode<'a> {
    fn codegen<'c>(&self, compiler: &'c Compiler) -> Result<IntValue<'c>, String> {
        match self {
            AstNode::Literal(inner) => inner.codegen(compiler),
            AstNode::Variable(inner) => inner.codegen(compiler),
            AstNode::Binary(inner) => inner.codegen(compiler),
            AstNode::Unary(inner) => inner.codegen(compiler),
            _ => todo!(),
            // AstNode::Call(inner) => inner.codegen(compiler),
            // AstNode::Function(inner) => inner.codegen(compiler),
            // AstNode::If(inner) => inner.codegen(compiler),
            // AstNode::While(inner) => inner.codegen(compiler),
            // AstNode::Begin(inner) => inner.codegen(compiler),
            // AstNode::Assign(inner) => inner.codegen(compiler),
            // AstNode::NewGlobal(inner) => inner.codegen(compiler),
            // AstNode::Error(inner) => inner.codegen(compiler),
        }
    }
}

impl<'a> Debug for AstNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Literal(inner) => inner.fmt(f),
            AstNode::Variable(inner) => inner.fmt(f),
            AstNode::Binary(inner) => inner.fmt(f),
            AstNode::Unary(inner) => inner.fmt(f),
            AstNode::Call(inner) => inner.fmt(f),
            AstNode::Function(inner) => inner.fmt(f),
            AstNode::If(inner) => inner.fmt(f),
            AstNode::While(inner) => inner.fmt(f),
            AstNode::Begin(inner) => inner.fmt(f),
            AstNode::Assign(inner) => inner.fmt(f),
            AstNode::NewGlobal(inner) => inner.fmt(f),
            AstNode::Error(inner) => inner.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Literal(pub i32); // leaf
#[derive(Debug)]
pub struct Variable<'a>(pub &'a str); // leaf
#[derive(Debug)]
pub struct Binary<'a>(pub &'a str, pub Box<AstNode<'a>>, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct Unary<'a>(pub &'a str, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct Call<'a>(pub &'a str, pub Vec<AstNode<'a>>);
#[derive(Debug)]
pub struct Function<'a>(pub &'a str, pub Vec<&'a str>, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct If<'a>(
    pub Box<AstNode<'a>>,
    pub Box<AstNode<'a>>,
    pub Box<AstNode<'a>>,
);
#[derive(Debug)]
pub struct While<'a>(pub Box<AstNode<'a>>, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct Begin<'a>(pub Vec<AstNode<'a>>);
#[derive(Debug)]
pub struct Assign<'a>(pub &'a str, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct NewGlobal<'a>(pub &'a str, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct RuntimeError;
