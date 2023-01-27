use crate::ast::Rule;
use crate::environment::Env;
use crate::io::eval_print;
use pest::iterators::Pairs;
pub struct Function {}
pub struct Formal {}

pub fn eval_define(_pairs: Pairs<Rule>, _env: &mut Env) {
    todo!()
}

pub fn eval_function(pairs: Pairs<Rule>, env: &mut Env) -> i32 {
    todo!()
    // match pairs.as_rule() {
    //     Rule::binary => eval_binary(pair.into_inner(), env),
    //     Rule::unary => eval_unary(pair.into_inner(), env),
    //     Rule::user => eval_user(pair.into_inner(), env),
    //     Rule::print => eval_print(pair.into_inner(), env),
    //     _ => unreachable!(),
    // }
}

fn eval_binary(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
fn eval_unary(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
fn eval_user(_pairs: Pairs<Rule>, _env: &mut Env) -> i32 {
    todo!()
}
