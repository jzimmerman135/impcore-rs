use super::*;
use crate::ast::AstExpr;
use inkwell::IntPredicate;

pub fn codegen_literal<'a>(
    value: i32,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    Ok(compiler.context.i32_type().const_int(value as u64, true))
}

pub fn codegen_variable<'a>(
    name: &str,
    maybe_indexer: Option<&AstExpr<'a>>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let mut addr = get_address(name, compiler)?;

    if let Some(indexer) = maybe_indexer {
        let index = indexer.codegen(compiler)?;
        addr = unsafe { compiler.builder.build_gep(addr, &[index], "index") };
    }

    Ok(compiler.builder.build_load(addr, "load").into_int_value())
}

pub fn codegen_assign<'a>(
    name: &str,
    maybe_indexer: Option<&AstExpr<'a>>,
    body: &AstExpr<'a>,
    compiler: &mut Compiler<'a>,
) -> Result<IntValue<'a>, String> {
    let mut addr = get_address(name, compiler)?;

    if let Some(indexer) = maybe_indexer {
        let index = indexer.codegen(compiler)?;
        addr = unsafe { compiler.builder.build_gep(addr, &[index], "index") };
    }

    let value = body.codegen(compiler)?;
    compiler.builder.build_store(addr, value);

    Ok(value)
}

fn get_address<'a>(name: &str, compiler: &Compiler<'a>) -> Result<PointerValue<'a>, String> {
    let local_variable = compiler.param_table.get(name);
    match local_variable {
        Some(&e) => match compiler.is_pointer(name) {
            true => Some(compiler.builder.build_load(e, "load").into_pointer_value()),
            false => Some(e),
        },
        None => compiler.global_table.get(name).map(|e| {
            compiler
                .builder
                .build_load(e.as_pointer_value(), "load")
                .into_pointer_value()
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
            .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
        "!=" => compiler
            .builder
            .build_int_compare(IntPredicate::NE, lhs, rhs, "ne"),
        _ => unimplemented!("Haven't built the {} binary operator yet", operator),
    };
    let itype = compiler.context.i32_type();
    let value = compiler.builder.build_int_cast(value, itype, "cast");
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

    let value = match operator {
        "++" => compiler.builder.build_int_add(body, one, "incr"),
        "--" => compiler.builder.build_int_sub(body, one, "decr"),
        "!" | "not" => compiler
            .builder
            .build_int_compare(IntPredicate::EQ, body, zero, "not"),
        _ => unimplemented!("Haven't built the {} unary operator yet", operator),
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
    let i32_type = compiler.context.i32_type();
    let parent_fn = compiler
        .curr_function
        .ok_or("No curr function in the if block")?;

    let zero = compiler.context.i32_type().const_zero();
    let cond_expr = condition_expr.codegen(compiler)?;
    let comparison =
        compiler
            .builder
            .build_int_compare(IntPredicate::NE, cond_expr, zero, "ifcond");

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

    let phi = compiler.builder.build_phi(i32_type, "iftmp");
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

    let _body_value = body_expr.codegen(compiler)?;
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
