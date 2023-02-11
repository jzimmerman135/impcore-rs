pub mod def_parse;
pub mod expr_parse;

use crate::ast::{AstDef, AstExpr};
pub use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar/impcore.pest"]
pub struct ImpcoreParser;

impl ImpcoreParser {
    pub fn generate_top_level_exprs(code: &str) -> Result<Vec<AstDef>, String> {
        let mut tests = vec![];
        let mut defs = ImpcoreParser::parse(Rule::impcore, code)
            .map_err(|e| format!("Parsing Failed: {}", e))?
            .next()
            .unwrap()
            .into_inner()
            .filter_map(|tle| match tle.as_rule() {
                Rule::EOI => None,
                Rule::check_assert | Rule::check_error | Rule::check_expect => {
                    tests.push(AstDef::parse(tle));
                    None
                }
                _ => Some(AstDef::parse(tle)),
            })
            .collect::<Vec<AstDef>>();

        defs.append(&mut tests);
        Ok(defs)
    }
}
