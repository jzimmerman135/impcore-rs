use super::*;

impl<'ctx> Compiler<'ctx> {
    pub fn defgen_function(
        &mut self,
        function: &'ctx ast::Function<'ctx>,
    ) -> Result<FunctionValue<'ctx>, String> {
        let function_name = function.0;
        let params = function.1.iter().collect::<Vec<_>>();
        let function_value = self.defgen_prototype(function_name, &params);

        self.formal_table.clear();
        for (param, param_value) in params.into_iter().zip(function_value.get_param_iter()) {
            self.formal_table
                .insert(*param, param_value.into_int_value());
        }

        let entry = self
            .context
            .append_basic_block(function_value, function_name);

        self.builder.position_at_end(entry);
        self.curr_function = Some(function_value);
        let body = self.codegen_expr(&function.2)?;
        self.curr_function = None;
        self.builder.build_return(Some(&body));

        if !function_value.verify(true) {
            self.module.print_to_stderr();
            unsafe {
                function_value.delete();
            }
            return Err(format!("Could not verify function {}", function_name));
        }

        // self.fpm.run_on(&function_value);

        Ok(function_value)
    }

    fn defgen_prototype<'a>(&self, name: &'a str, params: &[&&'a str]) -> FunctionValue<'ctx> {
        let ret_type = self.context.i32_type();
        let args_types = std::iter::repeat(ret_type)
            .take(params.len())
            .map(|f| f.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let fn_type = self.context.i32_type().fn_type(&args_types, false);
        let fn_val = self.module.add_function(name, fn_type, None);
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_int_value().set_name(params[i]);
        }
        fn_val
    }

    #[allow(unused)]
    fn defgen_val(&mut self, val: &'ctx ast::NewGlobal) -> Result<IntValue, String> {
        let name = val.0;
        let value = self.codegen_expr(&val.1)?;
        self.global_table.insert(name, value);
        Ok(value)
    }

    pub fn defgen_anonymous(&mut self, node: &'ctx AstNode) -> Result<FunctionValue<'ctx>, String> {
        let fn_type = self.context.i32_type().fn_type(&[], false);
        let fn_value = self.module.add_function("#anon", fn_type, None);
        let basic_block = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(basic_block);
        self.curr_function = Some(fn_value);
        let v = self.codegen_expr(node)?;
        self.builder.build_return(Some(&v));
        self.curr_function = None;

        if !fn_value.verify(true) {
            self.module.print_to_stderr();
            return Err(format!(
                "Could not verify anonymous expression \n{:?}\n{:?}",
                node, fn_value
            ));
        }

        Ok(fn_value)
    }

    pub fn defgen_check_assert(
        &mut self,
        node: &'ctx ast::CheckAssert,
    ) -> Result<FunctionValue<'ctx>, String> {
        self.defgen_anonymous(&*node.0)
    }

    pub fn defgen_check_expect(
        &mut self,
        node: &'ctx ast::CheckExpect,
    ) -> Result<(FunctionValue<'ctx>, FunctionValue<'ctx>), String> {
        Ok((
            self.defgen_anonymous(&*node.0)?,
            self.defgen_anonymous(&*node.1)?,
        ))
    }
}
