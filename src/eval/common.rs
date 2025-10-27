use crate::{
    import::ImportError,
    syntax::{Expr, Term, TermInner},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Undefined variable {name}")]
    UndefinedVariable { name: String },
    #[error("Maximum evaluation steps exceeded")]
    StepLimitExceeded,
    #[error("Evaluation likely diverged")]
    Divergence,
    #[error(transparent)]
    Import(#[from] ImportError),
}

pub fn de_bruijn(ast: &Expr, env: &mut Vec<String>) -> Result<Term, EvalError> {
    match ast {
        Expr::Var(name) => {
            if let Some(pos) = env.iter().rev().position(|n| n == name) {
                Ok(Term::var(name.clone(), pos))
            } else {
                Err(EvalError::UndefinedVariable { name: name.clone() })
            }
        }
        Expr::Lambda { param, body } => {
            env.push(param.clone());
            let body_term = de_bruijn(body, env)?;
            env.pop();
            Ok(Term::lambda(param.clone(), body_term))
        }
        Expr::Apply { func, arg } => {
            let func_term = de_bruijn(func, env)?;
            let arg_term = de_bruijn(arg, env)?;
            Ok(Term::apply(func_term, arg_term))
        }
    }
}

pub(super) fn shift(term: &Term, by: isize, cutoff: usize) -> Term {
    match term.inner() {
        TermInner::Var { name, index } => {
            if *index >= cutoff {
                let new_index = (*index as isize + by) as usize;
                Term::var(name.to_string(), new_index)
            } else {
                term.clone()
            }
        }
        TermInner::Lambda { param, body } => {
            let new_body = shift(body, by, cutoff + 1);
            Term::lambda(param.clone(), new_body)
        }
        TermInner::Apply { func, arg } => {
            let new_func = shift(func, by, cutoff);
            let new_arg = shift(arg, by, cutoff);
            Term::apply(new_func, new_arg)
        }
    }
}

pub(super) fn subst(term: &Term, index: usize, value: &Term) -> (Term, usize) {
    match term.inner() {
        TermInner::Var {
            name: _,
            index: var_index,
        } => {
            if *var_index == index {
                (shift(value, index as isize, 0), 1)
            } else {
                (term.clone(), 0)
            }
        }
        TermInner::Lambda { param, body } => {
            let (new_body, sub_count) = subst(body, index + 1, value);
            (Term::lambda(param.clone(), new_body), sub_count)
        }
        TermInner::Apply { func, arg } => {
            let (new_func, func_sub_count) = subst(func, index, value);
            let (new_arg, arg_sub_count) = subst(arg, index, value);
            (
                Term::apply(new_func, new_arg),
                func_sub_count + arg_sub_count,
            )
        }
    }
}

pub(super) fn beta(body: &Term, arg: &Term) -> (Term, bool) {
    let shifted_arg = shift(arg, 1, 0);
    let (substituted, sub_count) = subst(body, 0, &shifted_arg);
    (shift(&substituted, -1, 0), sub_count > 1)
}
