use std::collections::HashMap;

use cranelift::prelude::{settings::Value, types, FunctionBuilder, Variable};
use cranelift_jit::JITModule;

use crate::ast::AstNode;

#[allow(unused)]
pub struct FunctionTranslator<'a> {
    pub return_type: types::Type,
    pub builder: FunctionBuilder<'a>,
    pub variables: HashMap<String, Variable>,
    pub module: &'a mut JITModule,
}

impl<'a> FunctionTranslator<'a> {
    #[allow(unused)]
    fn translate_expr(&mut self, expr: AstNode) -> Value {
        todo!()
    }
}
