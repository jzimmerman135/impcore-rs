use crate::ast::{self, ImpcoreValue, Rule};
use crate::environment::Env;
use pest::iterators::{Pair, Pairs};

#[allow(dead_code)]
pub fn eval_file_use(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

#[allow(dead_code)]
pub fn eval_print(func_call: Pair<Rule>, env: &mut Env) -> ImpcoreValue {
    let mut print_func = func_call.into_inner();
    let print_operator = print_func.next().unwrap();
    let expr = print_func.next().unwrap();
    let v = ast::eval_exp(expr, env);
    match print_operator.as_str() {
        "println" => println!("{}", v),
        "printu" => print!("{}", v.abs()),
        "print" => print!("{}", v),
        _ => unreachable!(),
    }
    v
}
