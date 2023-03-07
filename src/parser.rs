pub mod def_parse;
pub mod expr_parse;
pub mod macro_parse;

use crate::ast::{Ast, AstDef, AstExpr};
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar/match.pest"]
pub struct ImpcoreParser;

impl ImpcoreParser {
    pub fn generate_ast(code: &str) -> Result<Ast, String> {
        let mut parser_output = Ast { defs: vec![] };
        let mut tests = vec![];
        let mut defs: Vec<AstDef> = ImpcoreParser::parse(Rule::impcore, code)
            .map_err(|e| format!("Parsing Failed: {}", e))?
            .next()
            .unwrap()
            .into_inner()
            .filter_map(|p| {
                if let Rule::EOI = p.as_rule() {
                    None
                } else {
                    Some(AstDef::parse(p))
                }
            })
            .collect();
        defs.append(&mut tests);
        parser_output.defs = defs;
        Ok(parser_output)
    }
}
