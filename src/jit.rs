use crate::ast::{Expr, Rule};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, Linkage, Module};
use std::{collections::HashMap, slice};

#[allow(unused)]
pub struct JIT {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    data_ctx: DataContext,
    module: JITModule,
}

impl Default for JIT {
    fn default() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
        }
    }
}

impl JIT {
    pub fn compile(&mut self, expressions: Vec<Expr>) -> Result<*const u8, String> {
        // self.translate(params, the_return, stmts)?;
        todo!()
    }

    pub fn create_data(&mut self, name: &str, contents: Vec<u8>) -> Result<&[u8], String> {
        self.data_ctx.define(contents.into_boxed_slice());
        let id = self
            .module
            .declare_data(name, Linkage::Export, true, false)
            .map_err(|e| e.to_string())?;

        self.module
            .define_data(id, &self.data_ctx)
            .map_err(|e| e.to_string())?;

        self.data_ctx.clear();
        self.module.finalize_definitions();
        let (data, len) = self.module.get_finalized_data(id);
        Ok(unsafe { slice::from_raw_parts(data, len) })
    }

    pub fn translate(
        &mut self,
        params: Vec<String>,
        the_return: String,
        expressions: Vec<Expr>,
    ) -> Result<(), String> {
        let int_type = self.module.target_config().pointer_type();
        for _ in &params {
            self.ctx.func.signature.params.push(AbiParam::new(int_type));
        }

        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(int_type));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_block = builder.create_block();

        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let variables = declare_variables(
            int_type,
            &mut builder,
            &params,
            &the_return,
            &expressions,
            entry_block,
        );

        let mut trans = FunctionTranslator {
            return_type: int_type,
            builder,
            variables,
            module: &mut self.module,
        };

        Err("".to_string())
    }
}

fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    the_return: &str,
    stmts: &[Expr],
    entry_block: Block,
) -> HashMap<String, Variable> {
    todo!()
}

struct FunctionTranslator<'a> {
    return_type: types::Type,
    builder: FunctionBuilder<'a>,
    variables: HashMap<String, Variable>,
    module: &'a mut JITModule,
}

impl<'a> FunctionTranslator<'a> {
    fn translate_expr(&mut self, expr: Expr) -> Value {
        todo!()
    }
}
