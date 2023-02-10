use super::*;
use inkwell::IntPredicate;

impl<'ctx> ast::AstNode<'ctx> {
    fn codegen(&self, compiler: &mut Compiler<'ctx>) -> Result<IntValue<'ctx>, String> {
        todo!();
    }
}

impl<'a> ast::Binary<'a> {
    fn codegen(&'a self, compiler: &'a mut Compiler<'a>) -> Result<IntValue<'a>, String> {
        let operator = self.0;
        let lhs = self.1.codegen(compiler)?;
        let rhs = self.2.codegen(compiler)?;
        let value = match operator {
            "*" => compiler.builder.build_int_mul(lhs, rhs, "mul"),
            "/" => compiler.builder.build_int_signed_div(lhs, rhs, "div"),
            "+" => compiler.builder.build_int_add(lhs, rhs, "mul"),
            "-" => compiler.builder.build_int_sub(lhs, rhs, "sub"),
            "%" | "mod" => compiler.builder.build_int_signed_rem(lhs, rhs, "mod"),
            ">" => compiler
                .builder
                .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            ">=" => compiler
                .builder
                .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            "<" => compiler
                .builder
                .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            "<=" => compiler
                .builder
                .build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            "=" => compiler
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, "le"),
            "!=" => compiler
                .builder
                .build_int_compare(IntPredicate::NE, lhs, rhs, "le"),
            _ => unimplemented!("Haven't built the {} binary operator yet", operator),
        };
        Ok(value)
    }
}

fn codegen_binary_no_OO<'ctx>(
    compiler: &'ctx mut Compiler<'ctx>,
    binary: &'ctx ast::Binary,
) -> Result<IntValue<'ctx>, String> {
    let operator = binary.0;
    let lhs = compiler.codegen_expr(&binary.1)?;
    let rhs = compiler.codegen_expr(&binary.2)?;
    let value = match operator {
        "*" => compiler.builder.build_int_mul(lhs, rhs, "mul"),
        "/" => compiler.builder.build_int_signed_div(lhs, rhs, "div"),
        "+" => compiler.builder.build_int_add(lhs, rhs, "mul"),
        "-" => compiler.builder.build_int_sub(lhs, rhs, "sub"),
        "%" | "mod" => compiler.builder.build_int_signed_rem(lhs, rhs, "mod"),
        ">" => compiler
            .builder
            .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
        ">=" => compiler
            .builder
            .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
        "<" => compiler
            .builder
            .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
        "<=" => compiler
            .builder
            .build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
        "=" => compiler
            .builder
            .build_int_compare(IntPredicate::EQ, lhs, rhs, "le"),
        "!=" => compiler
            .builder
            .build_int_compare(IntPredicate::NE, lhs, rhs, "le"),
        _ => unimplemented!("Haven't built the {} binary operator yet", operator),
    };
    let itype = compiler.context.i32_type();
    let value = compiler.builder.build_int_cast(value, itype, "cast");
    Ok(value)
}

impl<'ctx> Compiler<'ctx> {
    pub fn codegen_expr(&mut self, expr: &'ctx AstNode) -> Result<IntValue<'ctx>, String> {
        match expr {
            AstNode::Literal(inner) => self.codegen_literal(inner),
            AstNode::Variable(inner) => self.codegen_variable(inner),
            AstNode::Binary(inner) => self.codegen_binary(inner),
            AstNode::Unary(inner) => self.codegen_unary(inner),
            AstNode::Call(inner) => self.codegen_call(inner),
            AstNode::If(inner) => self.codegen_if(inner),
            AstNode::Begin(inner) => self.codegen_begin(inner),
            AstNode::While(inner) => self.codegen_while(inner),
            AstNode::Error(..) => Ok(self.context.i32_type().const_int(0, true)),
            _ => unimplemented!("Haven't implemented codegen for {:?}", expr),
            // AstNode::Assign(inner) => inner.codegen(compiler),
            // AstNode::NewGlobal(inner) => inner.codegen(compiler),
        }
    }

    fn codegen_literal(&mut self, literal: &'ctx ast::Literal) -> Result<IntValue<'ctx>, String> {
        Ok(self.context.i32_type().const_int(literal.0 as u64, false))
    }

    fn codegen_variable(
        &mut self,
        variable: &'ctx ast::Variable,
    ) -> Result<IntValue<'ctx>, String> {
        match self.formal_table.get(variable.0) {
            Some(&val) => Ok(val),
            None => match self.global_table.get(variable.0) {
                Some(&val) => Ok(val),
                None => Err(format!("variable {} not found", variable.0)),
            },
        }
    }

    fn codegen_binary(&mut self, binary: &'ctx ast::Binary) -> Result<IntValue<'ctx>, String> {
        let operator = binary.0;
        let lhs = self.codegen_expr(&binary.1)?;
        let rhs = self.codegen_expr(&binary.2)?;
        let value = match operator {
            "*" => self.builder.build_int_mul(lhs, rhs, "mul"),
            "/" => self.builder.build_int_signed_div(lhs, rhs, "div"),
            "+" => self.builder.build_int_add(lhs, rhs, "mul"),
            "-" => self.builder.build_int_sub(lhs, rhs, "sub"),
            "%" | "mod" => self.builder.build_int_signed_rem(lhs, rhs, "mod"),
            ">" => self
                .builder
                .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            ">=" => self
                .builder
                .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            "<" => self
                .builder
                .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            "<=" => self
                .builder
                .build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            "=" => self
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, "le"),
            "!=" => self
                .builder
                .build_int_compare(IntPredicate::NE, lhs, rhs, "le"),
            _ => unimplemented!("Haven't built the {} binary operator yet", operator),
        };
        let itype = self.context.i32_type();
        let value = self.builder.build_int_cast(value, itype, "cast");
        Ok(value)
    }

    fn codegen_unary(&mut self, unary: &'ctx ast::Unary) -> Result<IntValue<'ctx>, String> {
        let operator = unary.0;
        let arg = self.codegen_expr(&unary.1)?;
        let itype = self.context.i32_type();
        let one = itype.const_int(1, true);
        let zero = itype.const_int(0, true);
        let value = match operator {
            "++" => self.builder.build_int_add(arg, one, "incr"),
            "--" => self.builder.build_int_sub(arg, one, "decr"),
            "!" | "not" => self
                .builder
                .build_int_compare(IntPredicate::EQ, arg, zero, "not"),
            _ => unimplemented!("Haven't built the {} unary operator yet", operator),
        };
        let itype = self.context.i32_type();
        let value = self.builder.build_int_cast(value, itype, "cast");
        Ok(value)
    }

    fn codegen_call(&mut self, call: &'ctx ast::Call) -> Result<IntValue<'ctx>, String> {
        let function_name = call.0;
        let arg_nodes = call.1.iter();

        let function = match self.module.get_function(function_name) {
            Some(f) => f,
            None => return Err(format!("Unbound function {}", function_name)),
        };

        let args = arg_nodes
            .map(|e| match self.codegen_expr(e) {
                Ok(val) => Ok(BasicMetadataValueEnum::IntValue(val)),
                Err(str) => Err(str),
            })
            .collect::<Result<Vec<_>, String>>()?;

        self.builder
            .build_call(function, &args, "userfn")
            .try_as_basic_value()
            .left()
            .map(|e| Ok(e.into_int_value()))
            .unwrap()
    }

    fn codegen_if(&mut self, ifx: &'ctx ast::If) -> Result<IntValue<'ctx>, String> {
        let i32_type = self.context.i32_type();
        let parent_fn = self
            .curr_function
            .ok_or_else(|| "No curr function in the if block".to_string())?;

        let zero = self.context.i32_type().const_zero();
        let cond_expr = self.codegen_expr(&ifx.0)?;
        let comparison =
            self.builder
                .build_int_compare(IntPredicate::NE, cond_expr, zero, "ifcond");

        let then_block = self.context.append_basic_block(parent_fn, "then");
        let else_block = self.context.append_basic_block(parent_fn, "else");
        let merge_block = self.context.append_basic_block(parent_fn, "ifcont");

        self.builder
            .build_conditional_branch(comparison, then_block, else_block);

        self.builder.position_at_end(then_block);
        let then_val = self.codegen_expr(&ifx.1)?;
        self.builder.build_unconditional_branch(merge_block);
        let then_block = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_block);
        let else_val = self.codegen_expr(&ifx.2)?;
        self.builder.build_unconditional_branch(merge_block);
        let else_block = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_block);

        let phi = self.builder.build_phi(i32_type, "iftmp");
        phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
        Ok(phi.as_basic_value().into_int_value())
    }

    fn codegen_begin(&mut self, beginx: &'ctx ast::Begin) -> Result<IntValue<'ctx>, String> {
        let exprs = &beginx.0;
        let mut v = self.context.i32_type().const_int(0, true);
        for expr in exprs {
            v = self.codegen_expr(expr)?;
        }
        Ok(v)
    }

    fn codegen_while(&mut self, whilex: &'ctx ast::While) -> Result<IntValue<'ctx>, String> {
        let parent_fn = self
            .curr_function
            .ok_or_else(|| "No curr function in the if block".to_string())?;

        let end_cond_expr = &*whilex.0;
        let body_expr = &*whilex.1;

        let loop_block = self.context.append_basic_block(parent_fn, "loop");
        self.builder.build_unconditional_branch(loop_block);
        self.builder.position_at_end(loop_block);

        let _body_value = self.codegen_expr(body_expr)?;
        let end_cond = self.codegen_expr(end_cond_expr)?;
        let zero = self.context.i32_type().const_int(0, false);

        let end_cond =
            self.builder
                .build_int_compare(IntPredicate::NE, end_cond, zero, "whilecond");

        let after_block = self.context.append_basic_block(parent_fn, "afterwhile");
        self.builder
            .build_conditional_branch(end_cond, loop_block, after_block);

        self.builder.position_at_end(after_block);
        Ok(self.context.i32_type().const_int(0, false))
    }
}
