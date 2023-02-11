use inkwell::values::IntValue;

use crate::{
    jit::{codegen, defgen, Compiler, NativeTopLevel},
    parser::{def_parse, expr_parse, *},
};

#[derive(Debug, PartialEq)]
pub enum AstScope {
    Unknown,
    Global,
    Formal,
}

#[derive(Debug, PartialEq)]
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
    Function(&'a str, Vec<&'a str>, AstExpr<'a>),
    Global(&'a str, AstExpr<'a>),
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
}

impl<'a> AstExpr<'a> {
    pub fn codegen(&self, compiler: &mut Compiler<'a>) -> Result<IntValue<'a>, String> {
        match self {
            Self::Binary(op, lhs, rhs) => codegen::codegen_binary(op, lhs, rhs, compiler),
            Self::Unary(op, body) => codegen::codegen_unary(op, body, compiler),
            Self::If(cond, t, f) => codegen::codegen_if(cond, t, f, compiler),
            Self::While(cond, body) => codegen::codegen_while(cond, body, compiler),
            Self::Call(name, args) => codegen::codegen_call(name, args, compiler),
            Self::Literal(value) => codegen::codegen_literal(*value, compiler),
            Self::Variable(name, AstScope::Formal) => codegen::codegen_formal(name, compiler),
            Self::Variable(name, ..) => codegen::codegen_variable(name, compiler),
            Self::Error => codegen::codegen_literal(1, compiler),
            Self::Begin(exprs) => codegen::codegen_begin(exprs, compiler),
            _ => unreachable!(),
        }
    }
}

impl<'a> AstDef<'a> {
    pub fn defgen(&self, compiler: &mut Compiler<'a>) -> Result<NativeTopLevel<'a>, String> {
        Ok(match self {
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
            Self::Global(name, value) => {
                NativeTopLevel::TopLevelExpr(defgen::defgen_global(name, value, compiler)?)
            }
            _ => unreachable!(),
        })
    }
}
