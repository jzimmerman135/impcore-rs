pub mod def_parse;
pub mod expr_parse;
pub mod macro_parse;

use crate::ast::{Ast, AstDef, AstExpr};
pub use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar/macros.pest"]
pub struct ImpcoreParser;

impl ImpcoreParser {
    pub fn generate_ast(code: &str) -> Result<Ast, String> {
        let mut tests = vec![];
        let mut defs = ImpcoreParser::parse(Rule::impcore, code)
            .map_err(|e| format!("Parsing Failed: {}", e))?
            .next()
            .unwrap()
            .into_inner()
            .filter_map(|tldef| match tldef.as_rule() {
                Rule::EOI => None,
                Rule::check_assert | Rule::check_error | Rule::check_expect => {
                    tests.push(AstDef::parse(tldef));
                    None
                }
                _ => Some(AstDef::parse(tldef)),
            })
            .collect::<Vec<AstDef>>();

        defs.append(&mut tests);
        Ok(Ast(defs))
    }
}
