use impcore_rs::jit;
use impcore_rs::parser::ImpcoreParser;
use impcore_rs::{print_ast, print_ir, rip};
use std::{fs, process};

fn main() {
    let filename = "./imp/basic.imp";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| rip(format!("Failed to open file {}", filename)));

    let top_level_nodes = ImpcoreParser::generate_ast(&contents)
        .unwrap_or_else(|s| rip(s))
        .prepare();

    print_ast(&top_level_nodes);

    let context = inkwell::context::Context::create();
    let mut compiler =
        jit::Compiler::new(&context, jit::ExecutionMode::Jit).expect("Failed to build compiler");

    let tles = top_level_nodes
        .iter()
        .map(|e| e.defgen(&mut compiler))
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            process::exit(1)
        });

    print_ir(&compiler);

    println!("\nEXECUTION OUTPUT\n--------------------------------------------------");
    compiler.top_level_run_all(&tles);
}
