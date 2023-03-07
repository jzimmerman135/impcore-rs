/**
 * IMPCORE RUNTIME
 * CodeBase collect, opens all translation units
 * CodeBase build ast, assembles all code into an ast
 * Compiler codegen, converts and ast into NativeTopLevels
 * Compiler run all, runs the top level expressions
 * */
use clap::Parser as ArgParser;
use impcore_rs::jit;
use impcore_rs::preprocessor;
use impcore_rs::{print_ast, print_ir, rip};
use std::path::PathBuf;

#[derive(ArgParser, Debug)]
struct Cli {
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long)]
    quiet: bool,
    #[arg(short, long)]
    filename: Option<String>,
    #[arg(short, long)]
    emit_llvm: bool,
}

fn main() {
    let cli = Cli::parse();

    let entry_filepath = PathBuf::from(cli.filename.as_deref().unwrap_or("./imp/basic.imp"));
    let codebase = preprocessor::CodeBase::collect(&entry_filepath).unwrap_or_else(|s| rip(s));
    let ast = codebase
        .build_ast(&entry_filepath)
        .unwrap_or_else(|s| rip(s));

    if cli.debug {
        print_ast(&ast);
    }

    let context = inkwell::context::Context::create();
    let mut compiler =
        jit::Compiler::new(&context, jit::ExecutionMode::Jit).expect("Failed to build compiler");

    compiler.quiet_mode = cli.quiet;

    let native_top_level_functions = compiler.compile(&ast).unwrap_or_else(|e| rip(e));

    if cli.emit_llvm {
        compiler.module.print_to_file("error.ll").unwrap();
    }

    if cli.debug {
        print_ir(&compiler);
        eprintln!("\nEXECUTION OUTPUT\n--------------------------------------------------");
    }

    compiler.native_run_all(&native_top_level_functions);
}
