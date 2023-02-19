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

    let tles = ast
        .iter()
        .map(|e| e.defgen(&mut compiler))
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|e| rip(e));

    if cli.debug {
        print_ir(&compiler);
        eprintln!("\nEXECUTION OUTPUT\n--------------------------------------------------");
    }

    compiler.top_level_run_all(&tles);
}

#[cfg(test)]
mod tests {
    use inkwell::*;

    fn run_memset_on<'ctx>(
        context: &'ctx context::Context,
        module: &module::Module<'ctx>,
        alignment: u32,
    ) -> Result<(), &'static str> {
        let i8_type = context.i8_type();
        let i32_type = context.i32_type();
        let i64_type = context.i64_type();
        let array_len = 4;
        let fn_type = i32_type
            .ptr_type(AddressSpace::default())
            .fn_type(&[], false);
        let fn_value = module.add_function("test_fn", fn_type, None);
        let builder = context.create_builder();
        let entry = context.append_basic_block(fn_value, "entry");

        builder.position_at_end(entry);

        let len_value = i64_type.const_int(array_len as u64, false);
        let element_type = i32_type;
        let _array_type = element_type.array_type(array_len as u32);
        let array_ptr = builder
            .build_array_malloc(i32_type, len_value, "array_ptr")
            .unwrap();

        let elems_to_copy = 2;
        let bytes_to_copy = elems_to_copy * std::mem::size_of::<i32>();
        let size_val = i64_type.const_int(bytes_to_copy as u64, false);
        // Memset the first half of the array as 0
        let val = i8_type.const_zero();
        builder.build_memset(array_ptr, alignment, val, size_val)?;
        // Memset the second half of the array as -1
        let val = i8_type.const_all_ones();
        let index = i32_type.const_int(2, false);
        #[cfg(not(feature = "llvm15-0"))]
        let part_2 = unsafe { builder.build_in_bounds_gep(array_ptr, &[index], "index") };
        #[cfg(feature = "llvm15-0")]
        let part_2 =
            unsafe { builder.build_in_bounds_gep(element_type, array_ptr, &[index], "index") };
        builder.build_memset(part_2, alignment, val, size_val)?;
        builder.build_return(Some(&array_ptr));

        Ok(())
    }

    #[test]
    fn test_memset() {
        // 1. Allocate an array with a few elements.
        // 2. Memmove from the first half of the array to the second half.
        // 3. Run the code in an execution engine and verify the array's contents.
        let context = context::Context::create();
        let module = context.create_module("av");

        assert!(run_memset_on(&context, &module, 8).is_ok());

        module.print_to_stderr();
        // Verify the module
        if let Err(errors) = module.verify() {
            panic!("Errors defining module: {:?}", errors);
        }

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        unsafe {
            let func = execution_engine
                .get_function::<unsafe extern "C" fn() -> *const i32>("test_fn")
                .unwrap();
            let actual = std::slice::from_raw_parts(func.call(), 4);

            assert_eq!(&[0, 0, -1, -1], actual);
        }
    }
}
