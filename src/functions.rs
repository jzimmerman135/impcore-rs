use crate::ast::{self, eval_exp, ImpcoreValue, Rule};
use crate::environment::Env;
use crate::io::{self};
use pest::iterators::{Pair, Pairs};
pub struct FunctionEntry {}
pub struct Functions {}
impl Functions {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn lookup_function(&self, _name: &str) -> FunctionEntry {
        todo!()
    }

    #[allow(dead_code)]
    pub fn add_function(&self, _name: &str, _params: Vec<Formals>) {
        todo!()
    }
}
pub struct FormalEntry {}
pub struct Formals {}
impl Formals {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn push_formal(&self, _name: &str) {
        todo!()
    }

    #[allow(dead_code)]
    pub fn lookup_formal(&self, _name: &str) -> FormalEntry {
        todo!()
    }

    #[allow(dead_code)]
    pub fn pop_formal(&self, _name: &str, _params: Vec<Formals>) {
        todo!()
    }
}

#[allow(dead_code)]
pub fn eval_define(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

pub fn eval_function(expr: Pair<Rule>, env: &mut Env) -> ImpcoreValue {
    let func_call = expr.into_inner().next().unwrap();
    match func_call.as_rule() {
        Rule::unary => eval_unary(func_call, env),
        Rule::binary => eval_binary(func_call, env),
        Rule::print => io::eval_print(func_call, env),
        Rule::user => eval_user(func_call, env),
        _ => unreachable!(),
    }
}

fn eval_binary(func_call: Pair<Rule>, env: &mut Env) -> ImpcoreValue {
    let mut binary_func = func_call.into_inner();
    let binary_operator = binary_func.next().unwrap();
    let expr1 = binary_func.next().unwrap();
    let expr2 = binary_func.next().unwrap();
    let u = ast::eval_exp(expr1, env);
    let v = ast::eval_exp(expr2, env);
    match binary_operator.as_str() {
        "*" => u * v,
        "/" => u / v,
        "+" => u + v,
        "-" => u - v,
        "%" => u % v,
        ">>" => u >> v,
        "<<" => u << v,
        "&" => u & v,
        "|" => u | v,
        "^" => u ^ v,
        "=" => (u == v) as ImpcoreValue,
        ">" => (u > v) as ImpcoreValue,
        "<" => (u < v) as ImpcoreValue,
        ">=" => (u >= v) as ImpcoreValue,
        "<=" => (u <= v) as ImpcoreValue,
        "&&" => todo!(),
        "||" => todo!(),
        _ => unreachable!(),
    }
}

fn eval_unary(func_call: Pair<Rule>, env: &mut Env) -> ImpcoreValue {
    let mut unary_func = func_call.into_inner();
    let unary_operator = unary_func.next().unwrap();
    let expr = unary_func.next().unwrap();
    let v = ast::eval_exp(expr, env);
    match unary_operator.as_str() {
        "++" => v + 1,
        "--" => v - 1,
        "!" | "not" => (v == 0) as ImpcoreValue,
        _ => unreachable!(),
    }
}

fn eval_user(func_call: Pair<Rule>, env: &mut Env) -> i32 {
    let mut user_func = func_call.into_inner();
    let _func_name = user_func.next().unwrap();
    let (_n_args, _) = user_func.size_hint();
    let _args: Vec<ImpcoreValue> = user_func.map(|expr| eval_exp(expr, env)).collect();
    todo!()
}
