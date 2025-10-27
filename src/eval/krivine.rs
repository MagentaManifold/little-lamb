use std::rc::Rc;

use crate::syntax::{Term, TermInner};

use super::common::EvalError;

/// A closure pairs a term with its environment for lazy evaluation.
/// In the Krivine machine, closures represent suspended computations.
#[derive(Clone)]
struct Closure {
    term: Term,
    env: Env,
}

/// Environment: maps de Bruijn indices to closures (call-by-name thunks)
type Env = Rc<Vec<Closure>>;

impl Closure {
    fn new(term: Term, env: Env) -> Self {
        Self { term, env }
    }

    /// Creates a closure by extending an environment with a new binding
    fn with_extended_env(term: Term, binding: Closure, parent_env: &Env) -> Self {
        let mut extended = Vec::with_capacity(parent_env.len() + 1);
        extended.push(binding);
        extended.extend(parent_env.iter().cloned());
        Self {
            term,
            env: Rc::new(extended),
        }
    }
}

/// Work item for the normalization stack
enum Work {
    /// Normalize a closure and push result
    Normalize(Closure, usize),
    /// Build a lambda after body is normalized
    BuildLambda(String),
    /// Normalize and apply stacked arguments  
    NormalizeStackedArgs(Vec<Closure>, usize),
    /// Normalize the final argument
    NormalizeFinalArg(Closure, usize, usize), // arg_clo, depth, num_stacked_args
    /// Build the final application
    BuildFinalApp(usize), // num_stacked_args
}

/// Evaluate a term to normal form using the Krivine machine.
/// Uses an explicit work stack to avoid deep recursion (works on WASM).
pub fn eval(term: Term) -> Result<Term, EvalError> {
    let initial_clo = Closure::new(term, Rc::new(vec![]));
    normalize(initial_clo, 0)
}

/// Normalize a closure to normal form using iterative trampolining.
/// This avoids stack overflow by using an explicit work stack instead of recursion.
fn normalize(initial_clo: Closure, initial_depth: usize) -> Result<Term, EvalError> {
    let mut work_stack: Vec<Work> = vec![Work::Normalize(initial_clo, initial_depth)];
    let mut result_stack: Vec<Term> = Vec::new();
    let mut step = 0;

    while let Some(work) = work_stack.pop() {
        if step >= 1_000_000 {
            return Err(EvalError::StepLimitExceeded);
        }
        step += 1;

        match work {
            Work::Normalize(clo, depth) => match clo.term.inner() {
                TermInner::Var { name, index } => {
                    if *index < clo.env.len() {
                        // Follow environment binding
                        work_stack.push(Work::Normalize(clo.env[*index].clone(), depth));
                    } else if clo.env.is_empty() && *index < depth {
                        // Lambda-bound variable marker
                        result_stack.push(Term::var(name.to_string(), depth - *index - 1));
                    } else {
                        // Free variable
                        result_stack.push(Term::var(name.to_string(), *index - clo.env.len()));
                    }
                }

                TermInner::Lambda { param, body } => {
                    // Schedule: build lambda after normalizing body
                    work_stack.push(Work::BuildLambda(param.to_string()));
                    let bound_var =
                        Closure::new(Term::var(param.to_string(), depth), Rc::new(vec![]));
                    let child = Closure::with_extended_env(body.clone(), bound_var, &clo.env);
                    work_stack.push(Work::Normalize(child, depth + 1));
                }

                TermInner::Apply { func, arg } => {
                    // Evaluate function to WHNF
                    let func_clo = Closure::new(func.clone(), clo.env.clone());
                    let (head_clo, stack) = eval_to_whnf(func_clo, step)?;

                    match head_clo.term.inner() {
                        TermInner::Lambda { body, .. } => {
                            // Beta-reduction
                            let arg_clo = Closure::new(arg.clone(), clo.env.clone());
                            let reduced =
                                Closure::with_extended_env(body.clone(), arg_clo, &head_clo.env);

                            if stack.is_empty() {
                                work_stack.push(Work::Normalize(reduced, depth));
                            } else {
                                // Reapply stacked arguments
                                let result_term =
                                    stack.iter().rev().fold(reduced.term, |acc, stacked_arg| {
                                        Term::apply(acc, stacked_arg.term.clone())
                                    });
                                let final_clo = Closure::new(result_term, reduced.env);
                                work_stack.push(Work::Normalize(final_clo, depth));
                            }
                        }

                        _ => {
                            // Head is not a lambda - normalize all parts
                            let arg_clo = Closure::new(arg.clone(), clo.env.clone());
                            let num_stacked = stack.len();

                            // Schedule work: normalize func, then stacked args, then final arg, then combine
                            work_stack.push(Work::BuildFinalApp(num_stacked));
                            work_stack.push(Work::NormalizeFinalArg(arg_clo, depth, num_stacked));
                            work_stack.push(Work::NormalizeStackedArgs(stack, depth));
                            work_stack.push(Work::Normalize(head_clo, depth));
                        }
                    }
                }
            },

            Work::BuildLambda(param) => {
                let body = result_stack.pop().expect("Missing body for lambda");
                result_stack.push(Term::lambda(param, body));
            }

            Work::NormalizeStackedArgs(stack, depth) => {
                // Normalize each stacked argument in order
                for stacked_arg in stack.into_iter().rev() {
                    work_stack.push(Work::Normalize(stacked_arg, depth));
                }
            }

            Work::NormalizeFinalArg(arg_clo, depth, _num_stacked) => {
                work_stack.push(Work::Normalize(arg_clo, depth));
            }

            Work::BuildFinalApp(num_stacked) => {
                // Pop: final_arg, then stacked_args (in reverse), then func
                let final_arg = result_stack.pop().expect("Missing final argument");

                let mut stacked_args = Vec::with_capacity(num_stacked);
                for _ in 0..num_stacked {
                    stacked_args.push(result_stack.pop().expect("Missing stacked argument"));
                }

                let func = result_stack.pop().expect("Missing function");

                // Build application: func applied to stacked_args (reversed), then final_arg
                let mut result = func;
                for stacked_arg in stacked_args.into_iter().rev() {
                    result = Term::apply(result, stacked_arg);
                }
                result = Term::apply(result, final_arg);

                result_stack.push(result);
            }
        }
    }

    result_stack.pop().ok_or(EvalError::StepLimitExceeded)
}

/// Evaluate a closure to weak head normal form (WHNF).
///
/// Returns the closure at WHNF and any unapplied arguments on the stack.
/// WHNF means we stop at a lambda or free variable at the head position.
fn eval_to_whnf(mut clo: Closure, mut step: usize) -> Result<(Closure, Vec<Closure>), EvalError> {
    let mut stack: Vec<Closure> = Vec::new();

    loop {
        if step >= 1_000_000 {
            return Err(EvalError::StepLimitExceeded);
        }

        match clo.term.inner() {
            TermInner::Apply { func, arg } => {
                // Push argument onto stack and continue with function
                let arg_closure = Closure::new(arg.clone(), clo.env.clone());
                stack.push(arg_closure);
                clo = Closure::new(func.clone(), clo.env.clone());
                step += 1;
            }

            TermInner::Var { index, .. } => {
                if *index < clo.env.len() {
                    // Dereference variable in environment
                    clo = clo.env[*index].clone();
                    step += 1;
                } else {
                    // Free variable - we're at WHNF
                    return Ok((clo, stack));
                }
            }

            TermInner::Lambda { body, .. } => {
                if let Some(arg) = stack.pop() {
                    // Beta-reduction: apply lambda to argument
                    clo = Closure::with_extended_env(body.clone(), arg, &clo.env);
                    step += 1;
                } else {
                    // Lambda with no argument - we're at WHNF
                    return Ok((clo, stack));
                }
            }
        }
    }
}
