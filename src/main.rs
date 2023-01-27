extern crate pest;
#[macro_use]
extern crate pest_derive;

use parser::CSVParser;
pub use std::{error::Error, fs};

use pest::Parser;

mod parser;

fn csv_parse() -> Result<(), Box<dyn Error>> {
    let unparsed_file =
        fs::read_to_string("numbers.csv").expect("failed to read file \'numbers.csv\'");
    let file = CSVParser::parse(Rule::file, &unparsed_file)?
        .next()
        .unwrap();

    let mut record_count = 0;
    let mut field_sum = 0.0f64;

    for pair in file.into_inner() {
        match pair.as_rule() {
            Rule::record => {
                record_count += 1;
                for field in pair.into_inner() {
                    field_sum += field.as_str().parse::<f64>().unwrap();
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("sum of fields {}", field_sum);
    println!("number of fields {}", record_count);
    Ok(())
}

fn main() {
    csv_parse().unwrap();
}
