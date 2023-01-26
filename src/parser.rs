pub use pest::Parser;
#[derive(Parser)]
#[grammar = "grammar/csv.pest"]
pub struct ImpcoreParser;
