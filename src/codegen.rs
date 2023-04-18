use inkwell::{
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, FunctionValue, IntValue, PointerValue},
    IntPredicate,
};

use crate::{
    ast::{
        Def, Exp,
        Primitive::{self, *},
    },
    compiler::{Compiler, NativeFunction},
    env::{Env, Name, Type, Values},
};

impl<'ctx> Compiler<'ctx> {
    pub fn codegen(
        &mut self,
        def: Def,
        env: &Env,
        vals: &mut Values<'ctx>,
    ) -> Result<NativeFunction<'ctx>, String> {
        vals.clear_formals();
        let native = match &def {
            Def::Define(n, xs, e) => {
                let function = defgen_define(n, xs, e, self, env, vals)?;
                vals.add_function(*n, function);
                Ok(NativeFunction::DeclareFunction(*n))
            }
            Def::Val(n, e) => {
                let anonfn = defgen_val(n, e, self, env, vals)?;
                Ok(NativeFunction::TopLevelExp(anonfn))
            }
            Def::Exp(e) => {
                let anonfn = defgen_anon(e, self, env, vals)?;
                Ok(NativeFunction::TopLevelExp(anonfn))
            }
            Def::CheckExpect(l, r) => {
                let quiet_mode = self.quiet_mode;
                self.quiet_mode = true;
                let lhs = defgen_anon(l, self, env, vals)?;
                let rhs = defgen_anon(r, self, env, vals)?;
                self.quiet_mode = quiet_mode;
                Ok(NativeFunction::CheckExpect(lhs, rhs, def))
            }
            Def::CheckAssert(e) => {
                let quiet_mode = self.quiet_mode;
                self.quiet_mode = true;
                let cond = defgen_anon(e, self, env, vals)?;
                self.quiet_mode = quiet_mode;
                Ok(NativeFunction::CheckAssert(cond, def))
            }
            Def::Import(n) => {
                self.build_lib(n, env, vals)?;
                Ok(NativeFunction::NoOp)
            }
            _ => unreachable!("trying to codegen from a macro"),
        };
        native
    }

    pub fn evalgen(
        &mut self,
        exp: &Exp,
        env: &Env,
        vals: &Values<'ctx>,
    ) -> Result<IntValue<'ctx>, String> {
        if vals.curr_function.is_none() {
            panic!("No funcion to work inside");
        }

        match &exp {
            Exp::Literal(i) => Ok(self.itype.const_int(*i as u64, true)),
            Exp::Var(n, i) => {
                let i = match i {
                    Some(e) => Some(self.evalgen(e, env, vals)?),
                    None => None,
                };
                let addr = index_var(n, i, self, env, vals)?;
                let value = self.builder.build_load(self.itype, addr, "load");
                Ok(value.into_int_value())
            }
            Exp::Set(n, i, v) => {
                let i = match i {
                    Some(e) => Some(self.evalgen(e, env, vals)?),
                    None => None,
                };
                let value = self.evalgen(v, env, vals)?;
                let addr = index_var(n, i, self, env, vals)?;
                self.builder.build_store(addr, value);
                Ok(value)
            }
            Exp::Binary(p, l, r) => {
                let lhs = self.evalgen(l, env, vals)?;
                let rhs = self.evalgen(r, env, vals)?;
                gen_binary(*p, lhs, rhs, self)
            }
            Exp::Unary(p, e) => {
                let v = self.evalgen(e, env, vals)?;
                gen_unary(*p, v, self, env, vals)
            }
            Exp::Apply(n, args) => gen_call(n, args, self, env, vals),
            Exp::If(c, t, e) => gen_if(c, t, e, self, env, vals),
            Exp::While(g, b) => gen_while(g, b, self, env, vals),
            Exp::Begin(es) => {
                let vals = es
                    .iter()
                    .map(|e| self.evalgen(e, env, vals))
                    .collect::<Result<Vec<_>, _>>()?;
                let (last, _) = vals.split_last().unwrap();
                Ok(*last)
            }
            Exp::Match(p, cs, d) => gen_match(p, cs, d, self, env, vals),
        }
    }

    pub fn printres(&self, itval: &IntValue, env: &Env, vals: &Values) {
        if self.quiet_mode {
            return;
        }
        let println = vals.function(&env.tokens.get("println"), env).unwrap();
        self.builder
            .build_call(println, &[(*itval).into()], "printres");
    }
}

// definitions

fn defgen_anon<'ctx>(
    e: &Exp,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &mut Values<'ctx>,
) -> Result<FunctionValue<'ctx>, String> {
    let fn_type = compiler.itype.fn_type(&[], false);
    let fn_value = compiler.module.add_function("#anon", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    vals.curr_function = Some(fn_value);
    let v = compiler.evalgen(e, env, vals)?;
    compiler.printres(&v, env, vals);
    compiler.builder.build_return(Some(&v));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe { fn_value.delete() };
        return Err(format!(
            "Could not verify anonymous expression \n{:?}\n{:?}",
            e.to_string(&env.tokens),
            fn_value
        ));
    }

    Ok(fn_value)
}

fn defgen_val<'ctx>(
    n: &Name,
    e: &Exp,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &mut Values<'ctx>,
) -> Result<FunctionValue<'ctx>, String> {
    let fn_type = compiler.itype.fn_type(&[], false);
    let fn_value = compiler.module.add_function("#anon", fn_type, None);
    let basic_block = compiler.context.append_basic_block(fn_value, "entry");
    compiler.builder.position_at_end(basic_block);
    vals.curr_function = Some(fn_value);

    // declare or retrieve global
    let is_array = *env.varty(n)? == Type::Pointer;
    let itype = compiler.itype;
    let ptr_type = compiler.itype.ptr_type(compiler.addr_spc);
    let global_ptr = match vals.var(n, env, compiler) {
        Ok(global_ptr) => global_ptr,
        Err(_) => {
            let global = if !is_array {
                let global = compiler.module.add_global(
                    itype,
                    Some(compiler.addr_spc),
                    env.tokens.translate(n),
                );
                global.set_initializer(&itype.const_zero());
                global
            } else {
                let global_ptr = compiler.module.add_global(
                    ptr_type,
                    Some(compiler.addr_spc),
                    env.tokens.translate(n),
                );
                global_ptr.set_initializer(&ptr_type.const_null());
                global_ptr
            };
            vals.add_var(*n, global);
            global.as_pointer_value()
        }
    };

    // set global
    let body_value = compiler.evalgen(e, env, vals)?;

    let builder = &compiler.builder;
    let retval = if is_array {
        let old_array = builder
            .build_load(ptr_type, global_ptr, "load")
            .into_pointer_value();
        builder.build_free(old_array);
        let size = body_value;
        let sizeof_int = builder.build_int_cast(size.get_type().size_of(), itype, "cast");
        let n_bytes = builder.build_int_mul(size, sizeof_int, "bytes");
        let new_array = builder.build_array_malloc(itype, size, "array")?;
        let i8zero = compiler.context.i8_type().const_zero();
        builder.build_memset(new_array, 4, i8zero, n_bytes)?;
        builder.build_store(global_ptr, new_array);
        size
    } else {
        builder.build_store(global_ptr, body_value);
        body_value
    };

    compiler.printres(&retval, env, vals);
    builder.build_return(Some(&retval));

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe { fn_value.delete() };
        return Err(format!(
            "Could not verify val expression \n{:?}\n{:?}",
            e.to_string(&env.tokens),
            fn_value
        ));
    }

    Ok(fn_value)
}

fn defgen_define<'ctx>(
    name: &Name,
    params: &[Name],
    body: &Exp,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &mut Values<'ctx>,
) -> Result<FunctionValue<'ctx>, String> {
    let int_type = compiler.itype;
    let ptr_type = int_type.ptr_type(compiler.addr_spc);
    let fn_name = env.tokens.translate(name);

    // get fn_value
    let fn_value = if let Some(prev_function) = compiler.module.get_function(fn_name) {
        for bb in prev_function.get_basic_blocks() {
            bb.remove_from_function().unwrap();
        }
        prev_function
    } else {
        let mut args_types: Vec<BasicMetadataTypeEnum> = vec![];
        for param in params {
            match env
                .varty(param)
                .expect("Forgot to bind_defty before compiling")
            {
                Type::Int => args_types.push(int_type.into()),
                Type::Pointer => args_types.push(ptr_type.into()),
            }
        }
        let fn_type = compiler.context.i32_type().fn_type(&args_types, false);
        let fn_val = compiler.module.add_function(fn_name, fn_type, None);
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            let param_name = env.tokens.translate(&params[i]);
            match env
                .varty(&params[i])
                .expect("Forgot to bind_defty before compiling")
            {
                Type::Int => arg.into_int_value().set_name(param_name),
                Type::Pointer => arg.into_pointer_value().set_name(param_name),
            }
        }
        fn_val
    };

    // codegen body
    let entry = compiler.context.append_basic_block(fn_value, fn_name);
    compiler.builder.position_at_end(entry);
    vals.curr_function = Some(fn_value);

    for (&param, param_value) in params.iter().zip(fn_value.get_param_iter()) {
        let alloca = match env
            .varty(&param)
            .expect("Forgot to bind_defty before compiling")
        {
            Type::Int => {
                let param_int = param_value.into_int_value();
                let alloca = compiler.builder.build_alloca(int_type, "alloca");
                compiler.builder.build_store(alloca, param_int);
                alloca
            }
            Type::Pointer => {
                let param_ptr = param_value.into_pointer_value();
                let alloca = compiler.builder.build_alloca(ptr_type, "alloca");
                compiler.builder.build_store(alloca, param_ptr);
                alloca
            }
        };
        vals.add_param(param, alloca);
    }

    let body = compiler.evalgen(body, env, vals)?;
    compiler.builder.build_return(Some(&body));
    compiler.optimizer.run_on(&fn_value);

    if !fn_value.verify(true) {
        compiler.module.print_to_stderr();
        unsafe { fn_value.delete() };
        return Err(format!("Could not verify function {}", name));
    }

    Ok(fn_value)
}

// expressions
fn gen_match<'ctx>(
    pred_exp: &Exp,
    cases: &[(Exp, Exp)],
    default_exp: &Exp,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &Values<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let parent_fn = vals.curr_function.unwrap();
    let int_type = compiler.context.i32_type();

    let scrut = compiler.evalgen(pred_exp, env, vals)?;
    let default_block = compiler.context.append_basic_block(parent_fn, "default");
    let case_blocks = cases
        .iter()
        .map(|(lhs, _)| {
            let case_block = compiler.context.append_basic_block(parent_fn, "case");
            Ok((compiler.evalgen(lhs, env, vals)?, case_block))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let res_alloca = compiler.builder.build_alloca(int_type, "alloca");
    compiler
        .builder
        .build_switch(scrut, default_block, case_blocks.as_slice());
    let merge_block = compiler.context.append_basic_block(parent_fn, "merge");

    compiler.builder.position_at_end(default_block);
    let default = compiler.evalgen(default_exp, env, vals)?;
    compiler.builder.build_store(res_alloca, default);
    compiler.builder.build_unconditional_branch(merge_block);

    for ((_, block), (_, rhs)) in case_blocks.iter().zip(cases) {
        compiler.builder.position_at_end(*block);
        let armval = compiler.evalgen(rhs, env, vals)?;
        compiler.builder.build_store(res_alloca, armval);
        compiler.builder.build_unconditional_branch(merge_block);
    }

    compiler.builder.position_at_end(merge_block);
    let itype = compiler.context.i32_type();
    let v = compiler
        .builder
        .build_load(itype, res_alloca, "load")
        .into_int_value();

    Ok(v)
}

fn gen_while<'ctx>(
    guard_exp: &Exp,
    body_exp: &Exp,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &Values<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let parent_fn = vals.curr_function.unwrap();
    let loop_block = compiler.context.append_basic_block(parent_fn, "loop");
    compiler.builder.build_unconditional_branch(loop_block);
    compiler.builder.position_at_end(loop_block);

    compiler.evalgen(body_exp, env, vals)?;
    let end_cond = compiler.evalgen(guard_exp, env, vals)?;
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

fn gen_if<'ctx>(
    cond_exp: &Exp,
    then_exp: &Exp,
    else_exp: &Exp,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &Values<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let int_type = compiler.context.i32_type();

    let parent_fn = vals.curr_function.unwrap();

    let cond_expr = compiler.evalgen(cond_exp, env, vals)?;
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
    let then_val = compiler.evalgen(then_exp, env, vals)?;
    compiler.builder.build_unconditional_branch(merge_block);
    let then_block = compiler.builder.get_insert_block().unwrap();

    compiler.builder.position_at_end(else_block);
    let else_val = compiler.evalgen(else_exp, env, vals)?;
    compiler.builder.build_unconditional_branch(merge_block);
    let else_block = compiler.builder.get_insert_block().unwrap();

    compiler.builder.position_at_end(merge_block);

    let phi = compiler.builder.build_phi(int_type, "iftmp");
    phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
    Ok(phi.as_basic_value().into_int_value())
}

fn gen_call<'ctx>(
    n: &i32,
    args: &[Exp],
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &Values<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let function = vals.function(n, env)?;
    let argvals = args
        .iter()
        .map(|e| match e {
            Exp::Var(n, None) if *env.varty(n)? == Type::Pointer => {
                index_var(n, None, compiler, env, vals).map(BasicMetadataValueEnum::PointerValue)
            }
            _ => compiler
                .evalgen(e, env, vals)
                .map(BasicMetadataValueEnum::IntValue),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(compiler
        .builder
        .build_call(function, &argvals, "apply")
        .try_as_basic_value()
        .left()
        .ok_or(format!("weirdness in call {}", function))?
        .into_int_value())
}

fn gen_unary<'ctx>(
    prim: Primitive,
    body: IntValue<'ctx>,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &Values<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let one = compiler.itype.const_int(1, true);
    let zero = compiler.itype.const_zero();
    let value = {
        let builder = &compiler.builder;
        match prim {
            Not => builder.build_int_compare(IntPredicate::EQ, body, zero, "not"),
            Incr => builder.build_int_add(body, one, "incr"),
            Decr => builder.build_int_sub(body, one, "decr"),
            Neg => builder.build_int_nsw_neg(body, "neg"),
            Println | Printc | Print => {
                let prim_fn_name = env.tokens.get(prim.to_str());
                builder
                    .build_call(
                        vals.function(&prim_fn_name, env)
                            .map_err(|_| "Compiler bug missing print functions".to_string())?,
                        &[body.into()],
                        "print",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
                    .into_int_value()
            }
            _ => unreachable!(),
        }
    };
    let itype = compiler.context.i32_type();
    let value = compiler.builder.build_int_z_extend(value, itype, "zext");
    Ok(value)
}

fn gen_binary<'ctx>(
    prim: Primitive,
    lhs: IntValue<'ctx>,
    rhs: IntValue<'ctx>,
    compiler: &mut Compiler<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let value = {
        let builder = &compiler.builder;
        match prim {
            Mul => builder.build_int_mul(lhs, rhs, "mul"),
            Div => builder.build_int_signed_div(lhs, rhs, "div"),
            UDiv => builder.build_int_unsigned_div(lhs, rhs, "div"),
            Add => builder.build_int_add(lhs, rhs, "mul"),
            Sub => builder.build_int_sub(lhs, rhs, "sub"),
            Mod => builder.build_int_signed_rem(lhs, rhs, "mod"),
            Gt => builder.build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            Gte => builder.build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            Lt => builder.build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            Lte => builder.build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            Eq => builder.build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
            Neq => builder.build_int_compare(IntPredicate::NE, lhs, rhs, "ne"),
            BitXor => builder.build_xor(lhs, rhs, "xor"),
            BitAnd => builder.build_and(lhs, rhs, "bitand"),
            BitOr => builder.build_or(lhs, rhs, "bitor"),
            LShift => builder.build_left_shift(lhs, rhs, "shiftl"),
            RShift => builder.build_right_shift(lhs, rhs, true, "shiftr"),
            URShift => builder.build_right_shift(lhs, rhs, false, "ushiftr"),
            And => {
                let zero = compiler.context.i32_type().const_zero();
                let bool_false = compiler.context.bool_type().const_zero();
                let is_lhs = builder.build_int_compare(IntPredicate::NE, lhs, zero, "icmp");
                let is_rhs = builder.build_int_compare(IntPredicate::NE, rhs, zero, "icmp");
                builder
                    .build_select(is_lhs, is_rhs, bool_false, "select")
                    .into_int_value()
            }
            Or => {
                let zero = compiler.context.i32_type().const_zero();
                let bool_true = compiler.context.bool_type().const_zero();
                let is_lhs = builder.build_int_compare(IntPredicate::NE, lhs, zero, "icmp");
                let is_rhs = builder.build_int_compare(IntPredicate::NE, rhs, zero, "icmp");
                builder
                    .build_select(is_lhs, bool_true, is_rhs, "select")
                    .into_int_value()
            }
            _ => unreachable!(),
        }
    };
    let itype = compiler.context.i32_type();
    let value = compiler.builder.build_int_z_extend(value, itype, "zext");
    Ok(value)
}

fn index_var<'ctx>(
    n: &i32,
    i: Option<IntValue<'ctx>>,
    compiler: &mut Compiler<'ctx>,
    env: &Env,
    vals: &Values<'ctx>,
) -> Result<PointerValue<'ctx>, String> {
    let addr = vals.var(n, env, compiler)?;
    match i {
        Some(i) => Ok(unsafe {
            compiler
                .builder
                .build_gep(compiler.itype, addr, &[i], "index")
        }),
        None => Ok(addr),
    }
}
