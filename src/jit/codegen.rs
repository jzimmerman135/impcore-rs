use super::*;
use crate::ast::{AstExpr, AstType};
use inkwell::{values::BasicValue, IntPredicate};

impl<'a> AstExpr<'a> {
    pub fn codegen(&self, compiler: &mut Compiler<'a>) -> Result<IntValue<'a>, String> {
        match self {
            Self::Binary(op, lhs, rhs) => codegen::codegen_binary(op, lhs, rhs, compiler),
            Self::Unary(op, body) => codegen::codegen_unary(op, body, compiler),
            Self::If(cond, t, f) => codegen::codegen_if(cond, t, f, compiler),
            Self::While(cond, body) => codegen::codegen_while(cond, body, compiler),
            Self::Call(name, args) => codegen::codegen_call(name, args, compiler),
            Self::Literal(value) => codegen::codegen_literal(*value, compiler),
            Self::Variable(name, index) => {
                codegen::codegen_variable(name, index.as_deref(), compiler)
            }
            Self::Assign(name, body, index) => {
                codegen::codegen_assign(name, index.as_deref(), body, compiler)
            }
            Self::Error => codegen::codegen_literal(-1, compiler),
            Self::Match(scrut, arms) => codegen::codegen_match(scrut, arms.as_slice(), compiler),
            Self::Begin(exprs) => codegen::codegen_begin(exprs, compiler),
            _ => unreachable!("cannot codegen from {:?}", self),
        }
    }
}

pub fn codegen_literal<'a>(
    value: i32,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    Ok(compiler.context.i32_type().const_int(value as u64, true))
}

pub fn codegen_variable<'a>(
    name: &str,
    maybe_index: Option<&AstExpr<'a>>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let addr = index_address(get_address(name, compiler)?, maybe_index, compiler)?;
    let value = compiler.builder.build_load(addr, "load");
    Ok(value.into_int_value())
}

pub fn codegen_assign<'a>(
    name: &str,
    maybe_index: Option<&AstExpr<'a>>,
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let addr = index_address(get_address(name, compiler)?, maybe_index, compiler)?;
    let value = body.codegen(compiler)?;
    compiler.builder.build_store(addr, value);
    Ok(value)
}

fn index_address<'a>(
    addr: PointerValue<'a>,
    index: Option<&AstExpr<'a>>,
    compiler: &mut Compiler<'a>,
) -> Result<PointerValue<'a>, String> {
    if let Some(index_expr) = index {
        let index_value = index_expr.codegen(compiler)?;
        Ok(unsafe { compiler.builder.build_gep(addr, &[index_value], "index") })
    } else {
        Ok(addr)
    }
}

fn get_address<'a>(name: &str, compiler: &Compiler<'a>) -> Result<PointerValue<'a>, String> {
    let local_variable = compiler.param_table.get(name);
    match local_variable {
        Some(&local) => match compiler.is_pointer(name) {
            true => Some(
                compiler
                    .builder
                    .build_load(local, "load")
                    .into_pointer_value(),
            ),
            false => Some(local),
        },
        None => compiler
            .global_table
            .get(name)
            .map(|g| match compiler.is_pointer(name) {
                true => compiler
                    .builder
                    .build_load(g.as_basic_value_enum().into_pointer_value(), "load")
                    .into_pointer_value(),
                false => g.as_pointer_value(),
            }),
    }
    .ok_or(format!("Unbound variable {}", name))
}

pub fn codegen_binary<'a>(
    operator: &str,
    lhs_expr: &AstExpr<'a>,
    rhs_expr: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let lhs = lhs_expr.codegen(compiler)?;
    let rhs = rhs_expr.codegen(compiler)?;
    let value = {
        let builder = &compiler.builder;
        match operator {
            "*" => builder.build_int_mul(lhs, rhs, "mul"),
            "/" => builder.build_int_signed_div(lhs, rhs, "div"),
            "+" => builder.build_int_add(lhs, rhs, "mul"),
            "-" => builder.build_int_sub(lhs, rhs, "sub"),
            "%" | "mod" => builder.build_int_signed_rem(lhs, rhs, "mod"),
            ">" => builder.build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            ">=" => builder.build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            "<" => builder.build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            "<=" => builder.build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            "=" => builder.build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
            "!=" => builder.build_int_compare(IntPredicate::NE, lhs, rhs, "ne"),
            "^" => builder.build_xor(lhs, rhs, "xor"),
            "&" => builder.build_and(lhs, rhs, "bitand"),
            "|" => builder.build_or(lhs, rhs, "bitor"),
            "<<" => builder.build_left_shift(lhs, rhs, "shiftl"),
            ">>" => builder.build_right_shift(lhs, rhs, true, "shiftr"),
            ">>>" => builder.build_right_shift(lhs, rhs, false, "ushiftr"),
            "&&" | "and" => {
                let zero = compiler.context.i32_type().const_zero();
                let bool_false = compiler.context.bool_type().const_zero();
                let is_lhs = builder.build_int_compare(IntPredicate::NE, lhs, zero, "icmp");
                let is_rhs = builder.build_int_compare(IntPredicate::NE, rhs, zero, "icmp");
                builder
                    .build_select(is_lhs, is_rhs, bool_false, "select")
                    .into_int_value()
            }
            "||" | "or" => {
                let zero = compiler.context.i32_type().const_zero();
                let bool_true = compiler.context.bool_type().const_zero();
                let is_lhs = builder.build_int_compare(IntPredicate::NE, lhs, zero, "icmp");
                let is_rhs = builder.build_int_compare(IntPredicate::NE, rhs, zero, "icmp");
                builder
                    .build_select(is_lhs, bool_true, is_rhs, "select")
                    .into_int_value()
            }
            _ => unimplemented!("Haven't built the {} binary operator yet", operator),
        }
    };
    let itype = compiler.context.i32_type();
    let value = compiler.builder.build_int_z_extend(value, itype, "zext");
    Ok(value)
}

pub fn codegen_unary<'a>(
    operator: &str,
    body_expr: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let body = body_expr.codegen(compiler)?;
    let itype = compiler.context.i32_type();
    let one = itype.const_int(1, true);
    let zero = itype.const_zero();

    let value = {
        let builder = &compiler.builder;
        match operator {
            "++" => builder.build_int_add(body, one, "incr"),
            "--" => builder.build_int_sub(body, one, "decr"),
            "!" | "not" => builder.build_int_compare(IntPredicate::EQ, body, zero, "not"),
            "println" | "printu" | "print" => builder
                .build_call(
                    *compiler.lib.get(operator).unwrap(),
                    &[body.into()],
                    "print",
                )
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value(),
            _ => unimplemented!("Haven't built the {} unary operator yet", operator),
        }
    };

    let value = compiler.builder.build_int_cast(value, itype, "cast");
    Ok(value)
}

pub fn codegen_call<'a>(
    name: &str,
    args: &[AstExpr<'a>],
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let function = compiler
        .module
        .get_function(name)
        .ok_or(format!("Unbound function {}", name))?;

    let expected_argcount = function.get_params().len();
    let received_argcount = args.len();

    if expected_argcount != received_argcount {
        return Err(format!(
            "Function ({} ...) called with {} args instead of expected {}",
            name, received_argcount, expected_argcount,
        ));
    }

    let args = args
        .iter()
        .map(|e| match e {
            AstExpr::Pointer(name) => {
                get_address(name, compiler).map(BasicMetadataValueEnum::PointerValue)
            }
            _ => e.codegen(compiler).map(BasicMetadataValueEnum::IntValue),
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(compiler
        .builder
        .build_call(function, &args, "userfn")
        .try_as_basic_value()
        .left()
        .ok_or(format!("weirdness in call {}", function))?
        .into_int_value())
}

pub fn codegen_if<'a>(
    condition_expr: &AstExpr<'a>,
    true_expr: &AstExpr<'a>,
    false_expr: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let int_type = compiler.context.i32_type();
    let parent_fn = compiler
        .curr_function
        .ok_or("No curr function in the if block")?;

    let cond_expr = condition_expr.codegen(compiler)?;
    let comparison = compiler.builder.build_int_compare(
        IntPredicate::NE,
        cond_expr,
        int_type.const_zero(),
        "ifcond",
    );

    let then_block = compiler.context.append_basic_block(parent_fn, "then");
    let else_block = compiler.context.append_basic_block(parent_fn, "else");
    let merge_block = compiler.context.append_basic_block(parent_fn, "ifcont");

    compiler
        .builder
        .build_conditional_branch(comparison, then_block, else_block);

    compiler.builder.position_at_end(then_block);
    let then_val = true_expr.codegen(compiler)?;
    compiler.builder.build_unconditional_branch(merge_block);
    let then_block = compiler.builder.get_insert_block().unwrap();

    compiler.builder.position_at_end(else_block);
    let else_val = false_expr.codegen(compiler)?;
    compiler.builder.build_unconditional_branch(merge_block);
    let else_block = compiler.builder.get_insert_block().unwrap();

    compiler.builder.position_at_end(merge_block);

    let phi = compiler.builder.build_phi(int_type, "iftmp");
    phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
    Ok(phi.as_basic_value().into_int_value())
}

pub fn codegen_while<'a>(
    condition_expr: &AstExpr<'a>,
    body_expr: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let parent_fn = compiler
        .curr_function
        .ok_or_else(|| "No curr function in the if block".to_string())?;

    let loop_block = compiler.context.append_basic_block(parent_fn, "loop");
    compiler.builder.build_unconditional_branch(loop_block);
    compiler.builder.position_at_end(loop_block);

    body_expr.codegen(compiler)?;
    let end_cond = condition_expr.codegen(compiler)?;
    let zero = compiler.context.i32_type().const_int(0, false);

    let end_cond =
        compiler
            .builder
            .build_int_compare(IntPredicate::NE, end_cond, zero, "whilecond");

    let after_block = compiler.context.append_basic_block(parent_fn, "afterwhile");
    compiler
        .builder
        .build_conditional_branch(end_cond, loop_block, after_block);

    compiler.builder.position_at_end(after_block);
    Ok(compiler.context.i32_type().const_int(0, false))
}

pub fn codegen_begin<'a>(
    exprs: &[AstExpr<'a>],
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let mut v = compiler.context.i32_type().const_int(0, true);
    for expr in exprs {
        v = expr.codegen(compiler)?;
    }
    Ok(v)
}

fn codegen_match<'a>(
    scrut_expr: &AstExpr<'a>,
    arms_exprs: &[(AstExpr<'a>, AstExpr<'a>)],
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let parent_fn = compiler
        .curr_function
        .ok_or_else(|| "No curr function in the match block".to_string())?;
    let int_type = compiler.context.i32_type();

    let scrut = scrut_expr.codegen(compiler)?;
    let default_block = compiler.context.append_basic_block(parent_fn, "default");
    let case_blocks = arms_exprs
        .iter()
        .map(|(lhs, _)| {
            if lhs.contains(&mut |e| matches!(e, AstExpr::Variable(..) | AstExpr::Pointer(..))) {
                return Err(
                    "Left side of match cannot contain references to variables or pointers"
                        .to_string(),
                );
            }
            let case_block = compiler.context.append_basic_block(parent_fn, "case");
            Ok((lhs.codegen(compiler)?, case_block))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let res_alloca = compiler.builder.build_alloca(int_type, "alloca");
    compiler
        .builder
        .build_switch(scrut, default_block, case_blocks.as_slice());
    let merge_block = compiler.context.append_basic_block(parent_fn, "merge");

    compiler.builder.position_at_end(default_block);
    compiler.builder.build_store(res_alloca, scrut);
    compiler.builder.build_unconditional_branch(merge_block);

    for ((_, block), (_, rhs)) in case_blocks.iter().zip(arms_exprs) {
        compiler.builder.position_at_end(*block);
        let armval = rhs.codegen(compiler)?;
        compiler.builder.build_store(res_alloca, armval);
        compiler.builder.build_unconditional_branch(merge_block);
    }

    compiler.builder.position_at_end(merge_block);
    let v = compiler
        .builder
        .build_load(res_alloca, "load")
        .into_int_value();

    Ok(v)
}
