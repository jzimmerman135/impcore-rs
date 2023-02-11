use impcore_rs::parser::ImpcoreParser;
use impcore_rs::{ast::AstDef, jit};
use std::{fs, process};

#[allow(unused)]
fn print_ast(ast: &[AstDef]) {
    println!("\nPRINTING AST\n------------");
    for node in ast.iter() {
        println!("{:?}", node);
    }
}

#[allow(unused)]
fn print_ir(compiler: &jit::Compiler) {
    println!("\nLLVM IR\n--------------------------------------------------");
    compiler.module.print_to_stderr();
}

fn rip(e: String) -> ! {
    eprintln!("error: {}", e);
    process::exit(1)
}

fn main() {
    let filename = "./imp/hw1.imp";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| rip(format!("dailed to open file {}", filename)));

    let top_level_nodes =
        ImpcoreParser::generate_top_level_exprs(&contents).unwrap_or_else(|s| rip(s));

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
