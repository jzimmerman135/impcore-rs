use std::collections::HashMap;

use crate::{
    ast::{Def, Exp},
    env::{
        self, Env, Name,
        Type::{self, Int, Pointer},
    },
    implib, EXTERNAL_PARAM,
};
use colored::Colorize;
use itertools::Itertools;

impl<'a> Env<'a> {
    pub fn bind_defty(&mut self, def: &Def) -> Result<Vec<(Name, Option<Type>)>, String> {
        let oldtys = match def {
            Def::Val(n, _) => {
                self.globaltys.insert(*n, self.type_from_name_syntax(n));
                vec![]
            }
            Def::Define(n, xs, _) => {
                let argtys = xs
                    .iter()
                    .map(|x| (*x, self.type_from_name_syntax(x)))
                    .collect();
                self.funtys.insert(*n, argtys);
                let newtys = xs
                    .iter()
                    .map(|x| (*x, self.type_from_name_syntax(x)))
                    .collect::<HashMap<_, _>>();
                env::map_swap(&mut self.globaltys, newtys)
            }
            Def::Exp(Exp::Apply(n, args)) => {
                self.welltyped(n, args)?;
                vec![]
            }
            Def::Import(libname) => {
                let (_, function_mappings) = implib::libmapping(libname)?;
                for (callname, _, types) in function_mappings {
                    let fname = self.tokens.get(callname);
                    self.funtys
                        .insert(fname, types.iter().map(|t| (EXTERNAL_PARAM, *t)).collect());
                }
                vec![]
            }
            _ => {
                vec![]
            }
        };
        Ok(oldtys)
    }

    pub fn varty(&self, name: &Name) -> Result<&Type, String> {
        self.globaltys.get(name).ok_or(format!(
            "Could not find type of variable {}",
            self.tokens.translate(name)
        ))
    }

    pub fn funty(&self, name: &Name) -> Result<&Vec<(Name, Type)>, String> {
        self.funtys.get(name).ok_or(format!(
            "Could not find type of function ({} ...)",
            self.tokens.translate(name)
        ))
    }

    pub fn welltyped(&self, fname: &Name, args: &[Exp]) -> Result<bool, String> {
        let funty = self.funty(fname)?;
        let got = args
            .iter()
            .map(|arg| match arg {
                Exp::Var(n, None) => self.varty(n).copied(),
                _ => Ok(Int),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if funty.len() == args.len()
            && funty
                .iter()
                .map(|(_, t)| t)
                .zip(&got)
                .all(|(t1, t2)| t1 == t2)
        {
            Ok(true)
        } else {
            let tokens = &self.tokens;
            let fnamestr = tokens.translate(fname).cyan().bold();
            Err(format!(
                "In call {}\nContract ({} {}) called with bad types ({} {})",
                Exp::Apply(*fname, args.to_owned()).to_string(tokens),
                fnamestr,
                funty
                    .iter()
                    .map(|(n, ty)| if *ty != Pointer && *n != EXTERNAL_PARAM {
                        tokens.translate(n).to_owned()
                    } else if *n == EXTERNAL_PARAM {
                        ty.to_string().to_owned()
                    } else {
                        tokens.translate(n).to_owned() + "]"
                    })
                    .join(" ")
                    .blue()
                    .bold(),
                fnamestr,
                got.iter()
                    .enumerate()
                    .map(|(i, &gotty)| if i < funty.len() && gotty == funty[i].1 {
                        gotty.to_string().green()
                    } else {
                        gotty.to_string().red()
                    })
                    .join(" "),
            ))
        }
    }

    /// eek this seems so dangerous, i guess types always come from syntax though
    fn type_from_name_syntax(&self, name: &Name) -> Type {
        if self.tokens.translate(name).ends_with('[') {
            Pointer
        } else {
            Int
        }
    }
}

impl Type {
    pub fn to_string(&self) -> &str {
        match self {
            Pointer => "ptr",
            Int => "int",
        }
    }
}
