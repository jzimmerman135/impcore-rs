use std::path::{Path, PathBuf};

use inkwell::{memory_buffer::MemoryBuffer, module::Linkage};

use crate::{
    compiler::{Compiler, NativeFunction},
    env::{
        Env, Tokenizer,
        Type::{self, *},
        Values,
    },
    IMPLIB_PATH,
};

// impcore function name, FFI name, argument types
// undefined behaviour if C function uses non-int values in arguments or return
pub type LibFn = (&'static str, &'static str, Vec<Type>);
pub type LibListing = (&'static str, Vec<LibFn>);

pub trait LibFunctions {
    fn functions(&self) -> Vec<&'static str>;
}

impl LibFunctions for LibListing {
    fn functions(&self) -> Vec<&'static str> {
        let (_filename, funcmappings) = self;
        funcmappings.iter().map(|&(n, _, _)| n).collect()
    }
}

pub fn libmapping(libname: &str) -> Result<LibListing, String> {
    Ok(match libname {
        "stdio" => (
            "stdin.bc",
            vec![
                ("getc", "__implib_getc", vec![]),
                ("printstr", "__implib_printstr", vec![Pointer]),
            ],
        ),
        "termio" => ("termio.bc", vec![("hmm", "__implib_getc", vec![])]),
        _ => return Err(format!("Unbound libname {}", libname)),
    })
}

impl<'ctx> Compiler<'ctx> {
    pub fn build_lib(
        &mut self,
        libname: &str,
        env: &Env,
        vals: &mut Values<'ctx>,
    ) -> Result<(), String> {
        let (pathname, function_mappings) = libmapping(libname)?;
        let mut path = PathBuf::from(IMPLIB_PATH);
        path.push(pathname);
        self.link_bitcode(path.as_path())?;
        for (callname, realname, _) in function_mappings {
            if let Some(f) = self.module.get_function(realname) {
                let fname = env.tokens.get(callname);
                vals.add_function(fname, f);
            } else {
                return Err(format!(
                    "Failed to link internal library {} unimplemented required function {}",
                    libname, realname
                ));
            }
        }
        Ok(())
    }

    fn link_bitcode(&mut self, path: &Path) -> Result<(), String> {
        let memory_buffer = MemoryBuffer::create_from_file(path).expect("Failed to open file");
        let other = self
            .context
            .create_module_from_ir(memory_buffer)
            .map_err(|_| "Failed to create module from buffer".to_string())?;
        self.module.link_in_module(other).map_err(|s| s.to_string())
    }

    pub fn build_basis(&mut self, env: &mut Env, vals: &mut Values<'ctx>) -> Result<(), String> {
        let context = &self.context;
        let module = &self.module;
        let builder = &self.builder;

        let int_type = self.itype;
        let ptr_type = int_type.ptr_type(self.addr_spc);
        let str_type = self.context.i8_type().ptr_type(self.addr_spc);

        let printf_type = int_type.fn_type(&[str_type.into()], true);
        let printf_fn = self
            .module
            .add_function("printf", printf_type, Some(Linkage::External));

        let impcore_printers = [
            ("println", "%i\n", "fmt_ln", Int),
            ("print", "%i", "fmt_i", Int),
            ("printu", "%u", "fmt_u", Int),
            ("printc", "%c", "fmt_c", Int),
        ];

        for (printer_name, fmt_str, fmt, arg_type) in impcore_printers {
            let unary_type = int_type.fn_type(
                &[match arg_type {
                    Pointer => ptr_type.into(),
                    Int => int_type.into(),
                }],
                false,
            );
            let print_fn = module.add_function(printer_name, unary_type, None);
            let block = context.append_basic_block(print_fn, "entry");
            builder.position_at_end(block);
            let fmt_ptr = builder
                .build_global_string_ptr(fmt_str, fmt)
                .as_pointer_value();
            let param = print_fn.get_first_param().unwrap();
            builder.build_call(printf_fn, &[fmt_ptr.into(), param.into()], "printfcall");
            let retval = match arg_type {
                Type::Int => param.into_int_value(),
                Type::Pointer => int_type.const_zero(),
            };
            builder.build_return(Some(&retval));
            self.optimizer.run_on(&print_fn);
            vals.add_function(env.tokens.tokenize(printer_name), print_fn);
        }

        Ok(())
    }

    pub fn build_cleanup(
        &mut self,
        env: &Env,
        vals: &Values<'ctx>,
    ) -> Result<NativeFunction<'ctx>, String> {
        let itype = self.itype;
        let fn_type = itype.fn_type(&[], false);
        let fn_value = self.module.add_function("cleanup", fn_type, None);
        let basic_block = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(basic_block);

        for (name, global_ptr) in vals.globals() {
            if *env.varty(name)? == Type::Int {
                continue;
            }
            let array = self.builder.build_load(
                itype.ptr_type(self.addr_spc),
                global_ptr.as_pointer_value(),
                "load",
            );
            self.builder.build_free(array.into_pointer_value());
        }

        let v = itype.const_zero();
        self.builder.build_return(Some(&v));

        if !fn_value.verify(true) {
            self.module.print_to_stderr();
            return Err("broken free".into());
        }

        Ok(NativeFunction::FreeAll(fn_value))
    }
}
