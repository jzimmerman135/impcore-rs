use std::{fmt::Display, process::exit};

use colored::Colorize;
use env::Name;

extern crate pest;

#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod codegen;
pub mod compiler;
pub mod env;
pub mod implib;
pub mod lazygraph;
pub mod parse;
pub mod preprocessor;
pub mod types;

pub const IMPLIB_PATH: &str = "linklibs/";
pub const EXTERNAL_PARAM: Name = -1;

pub fn usage() -> ! {
    eprintln!(
        "{}: ./impcore {} {}{}{}
                            {}
                            {}
                            {}
                            {}",
        "Usage".bold().cyan(),
        "<filename>".blue(),
        "[-I ".yellow(),
        "<include_dir>".blue(),
        "]".yellow(),
        "[-e/--early]".yellow(),
        "[-a/--ast]".yellow(),
        "[-d/--debug]".yellow(),
        "[-q/--quiet]".yellow(),
    );
    exit(1)
}

pub trait Rip<T> {
    fn rip(self) -> T;
}

impl<T, E> Rip<T> for Result<T, E>
where
    E: Display,
{
    fn rip(self) -> T {
        match self {
            Ok(x) => x,
            Err(msg) => {
                eprintln!("{}: {}", "Impcore error".red().bold(), msg);
                exit(1)
            }
        }
    }
}
