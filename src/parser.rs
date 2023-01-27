use crate::Parser;

#[derive(Parser)]
#[grammar = "grammar/csv.pest"]
pub struct ImpcoreParser;

#[derive(Parser)]
#[grammar = "grammar/csv.pest"]
pub struct CSVParser;
