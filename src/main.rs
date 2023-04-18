use std::env::{self, Args};

use inkwell::context::Context;
use newimpcore::{
    ast::TokenString,
    compiler::{Compiler, ExecutionMode},
    env::{map_restore, Env, Values},
    lazygraph::LazyGraph,
    parse::Parser,
    preprocessor, Rip,
};

fn parse_args(mut args: Args) -> Flags {
    args.next();

    let mut flags = Flags::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-e" | "--early" => flags.print_raw_ast = true,
            "-a" | "--ast" => flags.print_preprocessed_ast = true,
            "-d" | "--debug" => flags.debug = true,
            "-l" | "--llvm" => flags.llvm = true,
            "-q" | "--quiet" => flags.quiet = true,
            "-I" => flags.dirs.push(args.next().expect("Missing include dir")),
            _ if flags.entryfile.is_empty() => flags.entryfile = arg,
            _ => continue,
        }
    }

    if flags.entryfile.is_empty() {
        newimpcore::usage();
    }

    flags
}

#[derive(Default)]
struct Flags {
    print_raw_ast: bool,
    print_preprocessed_ast: bool,
    debug: bool,
    quiet: bool,
    llvm: bool,
    dirs: Vec<String>,
    entryfile: String,
}

fn run() -> Result<(), String> {
    let flags = parse_args(env::args());

    let entryfile = &flags.entryfile;
    let mut dirs = flags.dirs.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    dirs.push("stdlib");

    let code = preprocessor::collect_code(entryfile, &dirs).rip();
    let (ast, tokens) = Parser::build_ast(&code).rip();

    let mut env = Env::new(tokens);

    if flags.print_raw_ast {
        eprint!("{}", ast.to_string(&env.tokens));
        return Ok(());
    }

    let ast = preprocessor::preprocessor(ast, &env.tokens)?;

    // dependency graph for lazy compilation
    let mut lazygraph = LazyGraph::new();

    // storage of SSA values
    let mut values = Values::default();

    let context = Context::create();
    let mut compiler = Compiler::new(&context, ExecutionMode::Jit)?;
    compiler.quiet_mode = flags.quiet;

    let mut native_top_level_exprs = vec![];
    compiler.build_basis(&mut env, &mut values)?;

    let mut ast_string = String::new();
    if flags.print_preprocessed_ast {
        ast_string = ast.to_string(&env.tokens);
    }

    for def in ast {
        let ready = lazygraph
            .bump(def, &env.tokens)
            .map_err(|e| lazygraph.explain(&e, &env.tokens))
            .rip();
        for def in ready {
            let oldbinds = env.bind_defty(&def).rip();
            if !flags.print_preprocessed_ast {
                let native = compiler.codegen(def, &env, &mut values)?;
                native_top_level_exprs.push(native);
            }
            map_restore(&mut env.globaltys, oldbinds);
        }
    }

    if flags.print_preprocessed_ast {
        eprint!("{}", ast_string);
    }

    if flags.llvm | flags.debug {
        compiler.module.print_to_stderr();
        return Ok(());
    }

    native_top_level_exprs.push(compiler.build_cleanup(&env, &values)?);
    compiler.native_run_all(&native_top_level_exprs, &env.tokens);
    Ok(())
}

fn main() {
    run().rip();
}
