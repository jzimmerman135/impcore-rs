use crate::{
    ast::{Ast, Def, Exp, Primitive},
    env::{Tokenizer, Tokens},
    implib::{libmapping, LibFunctions},
};

use colored::Colorize;
use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser as PestParser;

#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct Parser;

impl Parser {
    pub fn build_ast(code: &str) -> Result<(Ast, Tokens), String> {
        let tokens = Tokens::new();
        Parser::generate_ast(code, tokens)
    }

    fn generate_ast<'a>(
        code: &'a str,
        mut tokens: Tokens<'a>,
    ) -> Result<(Ast, Tokens<'a>), String> {
        let pairs = Parser::parse(Rule::impcore, code)
            .map_err(|e| format!("Parsing Failed: {}", e))?
            .next()
            .unwrap()
            .into_inner();
        let mut ast = vec![];
        for p in pairs {
            if p.as_rule() == Rule::EOI {
                continue;
            } else {
                let def = Def::parse(p, &mut tokens)?;
                ast.push(def);
            }
        }
        Ok((ast, tokens))
    }
}

impl Def {
    fn parse<'a>(def: Pair<'a, Rule>, tokens: &mut Tokens<'a>) -> Result<Def, String> {
        let def = def.into_inner().next().unwrap();
        let rule = def.as_rule();
        let errstr = def.as_str();

        if Rule::exp == rule {
            return Ok(Def::Exp(Exp::parse(def, tokens)?));
        }

        let mut inner = def.into_inner();
        let astdef = match rule {
            Rule::define | Rule::inline => {
                let name = tokens.tokenize(inner.next().unwrap().as_str().trim());
                let mut body = Exp::default();
                let mut params = vec![];
                for p in inner {
                    match p.as_rule() {
                        Rule::exp => {
                            body = Exp::parse(p, tokens)?;
                            break;
                        }
                        _ => params.push(tokens.tokenize(p.into_inner().next().unwrap().as_str())),
                    }
                }
                if rule == Rule::define {
                    Def::Define(name, params, body)
                } else {
                    Def::Inline(name, params, body)
                }
            }
            Rule::val | Rule::macrox => {
                let name =
                    tokens.tokenize(inner.next().unwrap().into_inner().next().unwrap().as_str());
                let value = Exp::parse(inner.next().unwrap(), tokens)?;
                Def::Val(name, value)
            }
            Rule::expect => {
                let lhs = Exp::parse(inner.next().unwrap(), tokens)?;
                let rhs = Exp::parse(inner.next().unwrap(), tokens)?;
                Def::CheckExpect(lhs, rhs)
            }
            Rule::assert => {
                let cond = Exp::parse(inner.next().unwrap(), tokens)?;
                Def::CheckAssert(cond)
            }
            Rule::alias => {
                let name = tokens.tokenize(inner.next().unwrap().as_str());
                let expression = Exp::parse(inner.next().unwrap(), tokens)?;
                Def::Alias(name, expression)
            }
            Rule::undef => {
                let name = tokens.tokenize(inner.next().unwrap().as_str());
                Def::Undef(name)
            }
            Rule::import => {
                let libname = inner.next().unwrap().as_str().trim();
                let internal_functions = libmapping(libname)?.functions();
                for f in internal_functions {
                    tokens.tokenize(f);
                }
                Def::Import(libname.to_string())
            }
            _ => unreachable!("Got unreachable def rule::{:?} in {}", rule, errstr),
        };
        Ok(astdef)
    }
}

fn str2lit(numstr: &str) -> i32 {
    let (sign, numstr) = match numstr.strip_prefix('-') {
        Some(negstr) => (-1, negstr),
        None => (1, numstr),
    };
    let number: u32 = if let Some(hexadecimal_str) = numstr.strip_prefix("0x") {
        u32::from_str_radix(hexadecimal_str, 16).unwrap()
    } else if let Some(binary_str) = numstr.strip_prefix("0b") {
        u32::from_str_radix(binary_str, 2).unwrap()
    } else {
        numstr.parse().unwrap()
    };
    number as i32 * sign
}

impl Exp {
    fn parse<'a>(exp: Pair<'a, Rule>, tokens: &mut Tokens<'a>) -> Result<Exp, String> {
        let errstr = exp.as_str();
        let exp = exp.into_inner().next().unwrap();
        let rule = exp.as_rule();
        let astexp = match rule {
            Rule::lit => Exp::Literal(str2lit(exp.as_str())),
            Rule::macrox => {
                let name = tokens.tokenize(exp.as_str());
                Exp::Var(name, None)
            }
            Rule::var => {
                let mut inner = exp.into_inner();
                let name = tokens.tokenize(inner.next().unwrap().as_str());
                let mut indexer = None;
                if let Some(index) = inner.next() {
                    indexer = Some(Box::new(Exp::parse(index, tokens)?));
                }
                Exp::Var(name, indexer)
            }
            Rule::set => {
                let mut inner = exp.into_inner();
                let mut var = inner.next().unwrap().into_inner();
                let name = tokens.tokenize(var.next().unwrap().as_str());
                let index = if let Some(index) = var.next() {
                    Some(Box::new(Exp::parse(index, tokens)?))
                } else {
                    None
                };
                let value = Exp::parse(inner.next().unwrap(), tokens)?;
                Exp::Set(name, index, Box::new(value))
            }
            Rule::ifx => {
                let mut inner = exp.into_inner();
                let cond = Exp::parse(inner.next().unwrap(), tokens)?;
                let lhs = Exp::parse(inner.next().unwrap(), tokens)?;
                let rhs = Exp::parse(inner.next().unwrap(), tokens)?;
                Exp::If(Box::new(cond), Box::new(lhs), Box::new(rhs))
            }
            Rule::whilex => {
                let mut inner = exp.into_inner();
                let guard = Exp::parse(inner.next().unwrap(), tokens)?;
                let body = Exp::parse(inner.next().unwrap(), tokens)?;
                Exp::While(Box::new(guard), Box::new(body))
            }
            Rule::begin => {
                let inner = exp.into_inner();
                let exprs = inner
                    .map(|p| Exp::parse(p, tokens))
                    .collect::<Result<_, _>>()?;
                Exp::Begin(exprs)
            }
            Rule::apply => {
                let mut inner = exp.into_inner();
                let f = inner.next().unwrap();
                let prim = Primitive::from(f.as_str());
                match f.as_rule() {
                    Rule::binary => {
                        let lhs = Exp::parse(inner.next().unwrap(), tokens)?;
                        let rhs = Exp::parse(inner.next().unwrap(), tokens)?;
                        Exp::Binary(prim.unwrap(), Box::new(lhs), Box::new(rhs))
                    }
                    Rule::unary => {
                        let body = Exp::parse(inner.next().unwrap(), tokens)?;
                        Exp::Unary(prim.unwrap(), Box::new(body))
                    }
                    _ => {
                        let fn_name = tokens.tokenize(f.as_str().trim());
                        let args = inner
                            .map(|p| match p.as_rule() {
                                Rule::exp => Exp::parse(p, tokens),
                                Rule::param => Ok(Exp::Var(
                                    tokens.tokenize(p.into_inner().next().unwrap().as_str()),
                                    None,
                                )),
                                _ => unreachable!(),
                            })
                            .collect::<Result<_, _>>()?;
                        Exp::Apply(fn_name, args)
                    }
                }
            }
            Rule::matchx => {
                let mut inner = exp.into_inner();
                let pred = Exp::parse(inner.next().unwrap(), tokens)?;
                let mut cases = vec![];
                let mut ill_formed_lhs_variables = vec![];
                for p in inner {
                    match p.as_rule() {
                        Rule::case => {
                            let mut arm = p.into_inner();
                            let when = arm.next().unwrap();
                            let whenstr = when.as_str();
                            let case = Exp::parse(when, tokens)?;
                            if case.contains(|e| {
                                matches!(e, Exp::Apply(..) | Exp::Set(..) | Exp::Var(..))
                            }) {
                                ill_formed_lhs_variables.push(whenstr)
                            }
                            let then = Exp::parse(arm.next().unwrap(), tokens)?;
                            cases.push((case, then))
                        }
                        Rule::exp => {
                            let default = Exp::parse(p, tokens)?;
                            let match_expr = Exp::Match(Box::new(pred), cases, Box::new(default));
                            if !ill_formed_lhs_variables.is_empty() {
                                return Err(format!(
                                    "Syntax error in {}\nNon-static members [{}] included in left side of match cases",
                                    match_expr.to_string(tokens),
                                    ill_formed_lhs_variables
                                        .iter()
                                        .map(|s| s.trim())
                                        .join(", ")
                                        .red(),
                                ));
                            }
                            return Ok(match_expr);
                        }
                        _ => break,
                    }
                }
                unreachable!()
            }
            _ => unreachable!("Got unreachable exp rule::{:?} in {}", rule, errstr),
        };
        Ok(astexp)
    }
}
