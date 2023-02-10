use std::fmt::Debug;

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
    CheckAssert(CheckAssert<'a>),
    CheckExpect(CheckExpect<'a>),
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
            AstNode::CheckAssert(inner) => inner.fmt(f),
            AstNode::CheckExpect(inner) => inner.fmt(f),
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
pub struct NewGlobal<'a>(pub &'a str, pub Box<AstNode<'a>>, pub Box<AstNode<'a>>);
#[derive(Debug)]
pub struct RuntimeError;
#[derive(Debug)]
pub struct CheckAssert<'a>(pub Box<AstNode<'a>>, pub &'a str);
#[derive(Debug)]
pub struct CheckExpect<'a>(pub Box<AstNode<'a>>, pub Box<AstNode<'a>>, pub &'a str);
