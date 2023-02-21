use clap::Parser as ArgParser;
use impcore_rs::jit;
use impcore_rs::parser::ImpcoreParser;
use impcore_rs::{print_ast, print_ir, rip};
use std::fs;

#[derive(ArgParser, Debug)]
struct Cli {
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long)]
    quiet: bool,
    #[arg(short, long)]
    filename: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let input_file = cli.filename.as_deref().unwrap_or("./imp/basic.imp");

    let contents = fs::read_to_string(input_file)
        .unwrap_or_else(|_| rip(format!("Failed to open file {}", input_file)));

    let ast = ImpcoreParser::generate_ast(&contents)
        .unwrap_or_else(|s| rip(s))
        .prepare();

    if cli.debug {
        print_ast(&ast);
    }

    let context = inkwell::context::Context::create();
    let mut compiler =
        jit::Compiler::new(&context, jit::ExecutionMode::Jit).expect("Failed to build compiler");

    compiler.quiet_mode = cli.quiet;

    let native_top_level_defs = ast
        .iter()
        .map(|e| e.defgen(&mut compiler))
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|e| rip(e));

    if cli.debug {
        print_ir(&compiler);
        eprintln!("\nEXECUTION OUTPUT\n--------------------------------------------------");
    }

    compiler.top_level_run_all(&native_top_level_defs);
}
