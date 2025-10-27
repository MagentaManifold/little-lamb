use std::rc::Rc;

use crate::syntax::{Term, TermInner};

use super::common::EvalError;

#[derive(Clone)]
enum Closure {
    Term { term: Term, env: Env },
    Dummy { name: Rc<str>, depth: usize },
    Neutral { term: Term, depth: usize },
}

impl Closure {
    fn term(term: Term, env: Env) -> Self {
        Closure::Term { term, env }
    }
    fn dummy(name: Rc<str>, depth: usize) -> Self {
        Closure::Dummy { name, depth }
    }
    fn neutral(term: Term, depth: usize) -> Self {
        Closure::Neutral { term, depth }
    }
}

#[derive(Clone)]
enum Env {
    Nil,
    Cons { next: Rc<Env>, closure: Rc<Closure> },
}

impl Env {
    fn push(self, clo: Closure) -> Env {
        Env::Cons {
            next: Rc::new(self),
            closure: Rc::new(clo),
        }
    }

    fn get(&self, mut index: usize) -> Option<Rc<Closure>> {
        let mut curr = self;
        while let Env::Cons { next, closure } = curr {
            if index == 0 {
                return Some(closure.clone());
            }
            curr = next;
            index -= 1;
        }
        None
    }
}

enum Continuation {
    ApplyArg(Closure),
    BuildLambda(Rc<str>),
}

type Stack = Vec<Continuation>;

pub fn eval(term: Term) -> Result<Term, EvalError> {
    let mut clo = Closure::term(term, Env::Nil);
    let mut step: usize = 0;
    let mut stack: Stack = Vec::new();
    let mut depth: usize = 0;
    while step < 1_000_000 {
        match clo {
            Closure::Term { ref term, ref env } => match term.inner() {
                TermInner::Apply { func, arg } => {
                    stack.push(Continuation::ApplyArg(Closure::term(
                        arg.clone(),
                        env.clone(),
                    )));
                    clo = Closure::term(func.clone(), env.clone());
                }
                TermInner::Lambda { param, body } => {
                    if matches!(stack.last(), Some(Continuation::ApplyArg(_))) {
                        if let Some(Continuation::ApplyArg(arg_clo)) = stack.pop() {
                            clo = Closure::term(body.clone(), env.clone().push(arg_clo));
                        }
                    } else {
                        depth += 1;
                        stack.push(Continuation::BuildLambda(param.clone()));
                        clo = Closure::term(
                            body.clone(),
                            env.clone().push(Closure::dummy(param.clone(), depth)),
                        );
                    }
                }
                TermInner::Var { name, index } => {
                    if let Some(var_clo) = env.get(*index) {
                        clo = (*var_clo).clone();
                    } else {
                        return Err(EvalError::UndefinedVariable {
                            name: name.to_string(),
                        });
                    }
                }
            },
            Closure::Dummy {
                name,
                depth: dummy_depth,
            } => {
                clo = Closure::neutral(Term::var(name.clone(), depth - dummy_depth), depth);
            }
            Closure::Neutral {
                ref term,
                depth: neutral_depth,
            } => {
                if let Some(cont) = stack.pop() {
                    match cont {
                        Continuation::ApplyArg(arg_clo) => match arg_clo {
                            Closure::Term { .. } => {
                                stack.push(Continuation::ApplyArg(clo.clone()));
                                clo = arg_clo;
                                depth = neutral_depth;
                            }
                            Closure::Neutral {
                                term: other_term,
                                depth: other_depth,
                            } => {
                                clo = Closure::neutral(
                                    Term::apply(other_term.clone(), term.clone()),
                                    other_depth,
                                );
                            }
                            Closure::Dummy { .. } => unreachable!(),
                        },
                        Continuation::BuildLambda(param) => {
                            let new_term = Term::lambda(param.clone(), term.clone());
                            clo = Closure::neutral(new_term, neutral_depth);
                        }
                    }
                } else {
                    return Ok(term.clone());
                }
            }
        }
        step += 1;
    }
    Err(EvalError::StepLimitExceeded)
}
