use crate::parser::Rule;
use pest::iterators::Pair;

pub struct Literal<'a>(&'a str); // leaf
pub struct GlobalVar<'a>(&'a str); // leaf
pub struct Prototype<'a>(&'a str, Vec<&'a str>, Box<AstNode<'a>>);
pub struct NewVar<'a>(&'a str, Box<AstNode<'a>>);
pub struct GlobalArray<'a>(&'a str, Box<AstNode<'a>>);
pub struct NewArray<'a>(&'a str, Box<AstNode<'a>>);
pub struct Call<'a>(&'a str, Vec<AstNode<'a>>);
pub struct Assign<'a>(&'a str, Box<AstNode<'a>>);
pub struct Incr<'a>(Box<AstNode<'a>>);
pub struct Decr<'a>(Box<AstNode<'a>>);
pub struct Not<'a>(Box<AstNode<'a>>);
pub struct Eq<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Lt<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Le<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Gt<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Ge<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct And<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Or<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Add<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Sub<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Mul<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Div<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Mod<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct BitAnd<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct BitOr<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Xor<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct ShiftLeft<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct ShiftRight<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct If<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct While<'a>(Box<AstNode<'a>>, Box<AstNode<'a>>);
pub struct Begin<'a>(Vec<AstNode<'a>>);
pub struct Print<'a>(&'a str, Box<AstNode<'a>>);
pub struct Test<'a>(Box<AstNode<'a>>);
pub struct Error;

pub trait RuleParse {
    fn parse_rule(pair: Pair<Rule>) -> AstNode;
}

#[enum_dispatch]
#[derive(Debug)]
pub enum AstNode {
    Literal(&'a str),   // leaf
    GlobalVar(&'a str), // leaf

    Prototype(&'a str, Vec<&'a str>, Box<AstNode<'a>>),
    NewVar(&'a str, Box<AstNode<'a>>),

    GlobalArray(&'a str, Box<AstNode<'a>>),
    NewArray(&'a str, Box<AstNode<'a>>),

    Call(&'a str, Vec<AstNode<'a>>),
    Assign(&'a str, Box<AstNode<'a>>),

    Incr(Box<AstNode<'a>>),
    Decr(Box<AstNode<'a>>),
    Not(Box<AstNode<'a>>),

    Eq(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Lt(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Le(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Gt(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Ge(Box<AstNode<'a>>, Box<AstNode<'a>>),

    And(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Or(Box<AstNode<'a>>, Box<AstNode<'a>>),

    Add(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Sub(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Mul(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Div(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Mod(Box<AstNode<'a>>, Box<AstNode<'a>>),

    BitAnd(Box<AstNode<'a>>, Box<AstNode<'a>>),
    BitOr(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Xor(Box<AstNode<'a>>, Box<AstNode<'a>>),
    ShiftLeft(Box<AstNode<'a>>, Box<AstNode<'a>>),
    ShiftRight(Box<AstNode<'a>>, Box<AstNode<'a>>),

    If(Box<AstNode<'a>>, Box<AstNode<'a>>, Box<AstNode<'a>>),
    While(Box<AstNode<'a>>, Box<AstNode<'a>>),
    Begin(Vec<AstNode<'a>>),
    Print(&'a str, Box<AstNode<'a>>),

    Test(Box<AstNode<'a>>),
    Error,
}
