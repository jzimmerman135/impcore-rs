use std::{fmt::Debug, ops::Deref};

use crate::env::{Name, Tokens};
use colored::Colorize;

pub type Ast = Vec<Def>;

#[derive(Clone, Debug)]
pub enum Exp {
    Literal(i32),
    Var(Name, Option<Box<Exp>>),
    Binary(Primitive, Box<Exp>, Box<Exp>),
    Unary(Primitive, Box<Exp>),
    Apply(Name, Vec<Exp>),
    If(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Begin(Vec<Exp>),
    Set(Name, Option<Box<Exp>>, Box<Exp>),
    Match(Box<Exp>, Vec<(Exp, Exp)>, Box<Exp>),
}

#[derive(Clone)]
pub enum Def {
    Define(Name, Vec<Name>, Exp),
    Val(Name, Exp),
    Exp(Exp),
    CheckExpect(Exp, Exp),
    CheckAssert(Exp),
    Import(String),
    Alias(Name, Exp),
    Inline(Name, Vec<Name>, Exp),
    Undef(Name),
}

#[derive(Clone, Copy, Debug)]
pub enum Primitive {
    Add,
    Sub,
    Mul,
    Div,
    UDiv,
    Mod,
    LShift,
    RShift,
    URShift,
    And,
    Or,
    Eq,
    Neq,
    Lte,
    Lt,
    Gte,
    Gt,
    Not,
    BitXor,
    BitAnd,
    BitOr,
    Print,
    Println,
    Printc,
    Incr,
    Decr,
    Neg,
}

// String conversions

impl Primitive {
    pub fn from(op: &str) -> Option<Primitive> {
        Some(match op.trim() {
            "+" => Primitive::Add,
            "-" => Primitive::Sub,
            "*" => Primitive::Mul,
            "/" => Primitive::Div,
            "udiv" => Primitive::UDiv,
            "%" | "mod" => Primitive::Mod,
            ">>" => Primitive::RShift,
            ">>>" => Primitive::URShift,
            "<<" => Primitive::LShift,
            "&&" | "and" => Primitive::And,
            "||" | "or" => Primitive::Or,
            "=" => Primitive::Eq,
            "!=" => Primitive::Neq,
            "<=" => Primitive::Lte,
            "<" => Primitive::Lt,
            ">=" => Primitive::Gte,
            ">" => Primitive::Gt,
            "!" | "not" => Primitive::Not,
            "^" => Primitive::BitXor,
            "&" => Primitive::BitAnd,
            "|" => Primitive::BitOr,
            "print" => Primitive::Print,
            "println" => Primitive::Println,
            "printc" => Primitive::Printc,
            "++" => Primitive::Incr,
            "--" => Primitive::Decr,
            "~" => Primitive::Neg,
            _ => return None,
        })
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Primitive::Add => "+",
            Primitive::Sub => "-",
            Primitive::Mul => "*",
            Primitive::Div => "/",
            Primitive::UDiv => "udiv",
            Primitive::Mod => "%",
            Primitive::RShift => ">>",
            Primitive::URShift => ">>>",
            Primitive::LShift => "<<",
            Primitive::And => "&&",
            Primitive::Or => "||",
            Primitive::Eq => "=",
            Primitive::Neq => "!=",
            Primitive::Lte => "<=",
            Primitive::Lt => "<",
            Primitive::Gte => ">=",
            Primitive::Gt => ">",
            Primitive::Not => "!",
            Primitive::BitXor => "^",
            Primitive::BitAnd => "&",
            Primitive::BitOr => "|",
            Primitive::Print => "print",
            Primitive::Println => "println",
            Primitive::Printc => "printc",
            Primitive::Incr => "++",
            Primitive::Decr => "--",
            Primitive::Neg => "~",
        }
    }
}

pub trait TokenString {
    fn to_string(&self, tokens: &Tokens) -> String;
}

impl TokenString for Ast {
    fn to_string(&self, tokens: &Tokens) -> String {
        let ast = self;
        ast.iter()
            .map(|d| d.to_string(tokens) + "\n")
            .collect::<String>()
    }
}

impl Def {
    pub fn to_string(&self, tokens: &Tokens) -> String {
        match self {
            Def::Exp(e) => e.to_string(tokens),
            Def::Val(n, e) if tokens.translate(n).ends_with('[') => {
                format!(
                    "({} {}] {})",
                    "val".purple(),
                    tokens.translate(n),
                    e.to_string(tokens)
                )
            }
            Def::Val(n, e) => format!(
                "({} {} {})",
                "val".purple(),
                tokens.translate(n),
                e.to_string(tokens)
            ),
            Def::Define(n, ps, b) => format!(
                "({} {} ({}) {})",
                "define".purple(),
                tokens.translate(n).cyan(),
                ps.iter()
                    .flat_map(|n| translate_param(n, tokens))
                    .collect::<Vec<_>>()
                    .join(" "),
                b.to_string(tokens),
            ),
            Def::CheckExpect(l, r) => format!(
                "({} {} {})",
                "check-expect".purple(),
                l.to_string(tokens),
                r.to_string(tokens),
            ),
            Def::CheckAssert(e) => {
                format!("({} {})", "check-assert".purple(), e.to_string(tokens))
            }
            Def::Import(n) => format!("#({} {})", "replace".purple(), n.yellow()),
            Def::Alias(n, e) => {
                format!(
                    "#({} {} {})",
                    "replace".purple(),
                    tokens.translate(n).yellow(),
                    e.to_string(tokens)
                )
            }
            Def::Inline(n, ps, e) => format!(
                "#({} ({} {}) {})",
                "replace".purple(),
                tokens.translate(n).yellow(),
                ps.iter()
                    .flat_map(|n| translate_param(n, tokens))
                    .collect::<Vec<_>>()
                    .join(" "),
                e.to_string(tokens),
            ),
            Def::Undef(n) => format!("#({} {})", "undef".purple(), tokens.translate(n).yellow()),
        }
    }
}

impl Exp {
    pub fn to_string(&self, tokens: &Tokens) -> String {
        match self {
            Exp::Literal(l) => format!("{}", l).yellow().to_string(),
            Exp::Var(n, None) if tokens.translate(n).ends_with('[') => {
                format!("{}]", tokens.translate(n))
            }
            Exp::Var(n, None) => format!(
                "{}",
                match tokens.translate(n) {
                    x if x.starts_with('\'') => x.yellow(),
                    x => x.normal(),
                }
            ),
            Exp::Var(n, Some(e)) => {
                format!("{}{}]", tokens.translate(n), e.to_string(tokens))
            }
            Exp::Set(n, None, v) => {
                format!(
                    "({} {} {})",
                    "set".purple(),
                    tokens.translate(n),
                    v.to_string(tokens)
                )
            }
            Exp::Set(n, Some(e), v) => {
                format!(
                    "({} {}{}] {})",
                    "set".purple(),
                    tokens.translate(n),
                    e.to_string(tokens),
                    v.to_string(tokens)
                )
            }
            Exp::Unary(p, e) => {
                format!("({} {})", p.to_str().blue().bold(), e.to_string(tokens),)
            }
            Exp::Binary(p, l, r) => {
                format!(
                    "({} {} {})",
                    p.to_str().blue().bold(),
                    l.to_string(tokens),
                    r.to_string(tokens)
                )
            }
            Exp::If(c, t, e) => {
                format!(
                    "({} {} {} {})",
                    "if".purple(),
                    c.to_string(tokens),
                    t.to_string(tokens),
                    e.to_string(tokens)
                )
            }
            Exp::While(g, b) => {
                format!(
                    "({} {} {})",
                    "while".purple(),
                    g.to_string(tokens),
                    b.to_string(tokens),
                )
            }
            Exp::Begin(es) => {
                format!(
                    "({} {})",
                    "begin".purple(),
                    es.iter()
                        .map(|e| e.to_string(tokens))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            Exp::Apply(n, es) => {
                format!(
                    "({}{}{})",
                    match tokens.translate(n) {
                        x if x.starts_with('\'') => x.yellow(),
                        x => x.cyan(),
                    },
                    if es.is_empty() { "" } else { " " },
                    es.iter()
                        .map(|e| e.to_string(tokens))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            Exp::Match(p, cs, d) => {
                let arrow = "=>".blue().bold();
                let wildcard = "_".blue().bold();
                format!(
                    "({} {} {} ({} {} {}))",
                    "match".purple(),
                    p.to_string(tokens),
                    if cs.is_empty() {
                        "\x08".to_string()
                    } else {
                        cs.iter()
                            .map(|(e, t)| {
                                format!(
                                    "({} {} {})",
                                    e.to_string(tokens),
                                    arrow,
                                    t.to_string(tokens)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    },
                    wildcard,
                    arrow,
                    d.to_string(tokens)
                )
            }
        }
    }
}

fn translate_param<'a>(n: &Name, tokens: &Tokens<'a>) -> Vec<&'a str> {
    let p = tokens.translate(n);
    if p.ends_with('[') {
        vec![p, "\x08]"]
    } else {
        vec![p]
    }
}

// Defaults

impl Default for Exp {
    fn default() -> Self {
        Exp::Literal(i32::MIN)
    }
}

// Contains

impl Exp {
    fn children(&self) -> Vec<&Self> {
        match self {
            Exp::Literal(_) | Exp::Var(_, None) => vec![],
            Exp::Set(_, None, e) | Exp::Unary(_, e) | Exp::Var(_, Some(e)) => vec![e.deref()],
            Exp::Binary(_, e1, e2) | Exp::Set(_, Some(e1), e2) | Exp::While(e1, e2) => {
                vec![e1.deref(), e2.deref()]
            }
            Exp::If(e1, e2, e3) => vec![e1.deref(), e2.deref(), e3.deref()],
            Exp::Apply(_, es) | Exp::Begin(es) => es.iter().collect(),
            Exp::Match(e1, es, e2) => es
                .iter()
                .map(|(e1, e2)| [e1, e2])
                .chain([[e1.deref(), e2.deref()]])
                .flatten()
                .collect(),
        }
    }

    pub fn contains<F>(&self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(&Self) -> bool,
    {
        if f(self) {
            true
        } else {
            self.children().into_iter().any(f)
        }
    }
}
