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

#[derive(ArgParser)]
struct Cli {
    #[arg(short, long)]
    filename: Option<String>,
    #[arg(short, long)]
    quiet: bool,
    #[arg(short, long)]
    interpreter: bool,
    #[arg(short, long)]
    debug: bool,
    #[arg(long)]
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
    let exec_mode = match cli.interpreter {
        true => jit::ExecutionMode::Interpreter,
        false => jit::ExecutionMode::Jit,
    };

    let mut compiler = jit::Compiler::new(&context, exec_mode).expect("Failed to build compiler");
    compiler.quiet_mode = cli.quiet;

    if cli.interpreter {
        compiler.interpret(&ast).unwrap_or_else(|e| rip(e));
        return;
    }

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
