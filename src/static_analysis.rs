use crate::ast::*;
use std::collections::HashSet;

pub fn rebuild(mut ast: Ast) -> Result<Ast, String> {
    squash_globals(&mut ast);
    build_scopes(&mut ast)?;
    ast.0.push(AstDef::FreeAll);
    Ok(ast)
}

fn get_globals<'a>(ast: &Ast<'a>) -> HashSet<&'a str> {
    ast.0
        .iter()
        .filter_map(|e| match e {
            AstDef::Global(name, ..) => Some(&**name),
            _ => None,
        })
        .collect()
}

/// leaves unbounded variables scoped as `AstScope::Unknown`
///
/// `AstScope::Local` means stack allocated for a function call,
/// `AstScope::Param` params means SSA,
/// `AstScope::Global` means constant pointer to heap variable,
fn build_scopes(ast: &mut Ast) -> Result<(), String> {
    let globals = get_globals(ast);
    for def in ast.iter_mut() {
        match def {
            AstDef::Function(_, params, locals, body) => {
                locals.clear();

                // so we know where to store it
                body.apply_mut(&mut |expr| {
                    match expr {
                        AstExpr::Assign(name, value, AstScope::Unknown)
                            if params.contains(name) =>
                        {
                            locals.insert(name);
                            *expr = AstExpr::Assign(name, value.to_owned(), AstScope::Local);
                        }
                        AstExpr::Assign(name, value, AstScope::Unknown)
                            if globals.contains(name) =>
                        {
                            *expr = AstExpr::Assign(name, value.to_owned(), AstScope::Global);
                        }
                        _ => (),
                    };
                    Ok(())
                })?;

                // so we know where to look it up
                body.apply_mut(&mut |expr| {
                    match expr {
                        AstExpr::Variable(name, AstScope::Unknown) if locals.contains(name) => {
                            *expr = AstExpr::Variable(name, AstScope::Local);
                        }
                        AstExpr::Variable(name, AstScope::Unknown) if params.contains(name) => {
                            *expr = AstExpr::Variable(name, AstScope::Param);
                        }
                        AstExpr::Variable(name, AstScope::Unknown) if globals.contains(name) => {
                            *expr = AstExpr::Variable(name, AstScope::Global);
                        }
                        _ => (),
                    };
                    Ok(())
                })?;
            }
            _ => def.apply_to_children(&mut |expr| {
                // we now know where to store these
                match expr {
                    AstExpr::Assign(name, value, AstScope::Unknown) if globals.contains(name) => {
                        *expr = AstExpr::Assign(name, value.to_owned(), AstScope::Global);
                    }
                    AstExpr::Variable(name, AstScope::Unknown) if globals.contains(name) => {
                        *expr = AstExpr::Variable(name, AstScope::Global);
                    }
                    _ => (),
                };
                Ok(())
            })?,
        }
    }
    Ok(())
}

fn squash_globals(ast: &mut Ast) {
    use std::mem;
    use AstExpr::Assign;
    use AstScope::Global as GlobalScope;

    let mut global_names = HashSet::new();
    let mut declarations = vec![];
    let mut defs = mem::take(&mut ast.0)
        .into_iter()
        .map(|e| match &e {
            AstDef::Global(n, v) => {
                if global_names.insert(&**n) {
                    declarations.push(AstDef::Global(n, AstExpr::Literal(0)));
                }
                AstDef::TopLevelExpr(Assign(n, Box::new(v.to_owned()), GlobalScope))
            }
            _ => e,
        })
        .collect();

    declarations.append(&mut defs);
    *ast = Ast(declarations)
}
