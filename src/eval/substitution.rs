use crate::syntax::{Term, TermInner};

use super::common::{EvalError, beta};

/// Evaluate the term to normal form
pub fn eval(term: Term) -> Result<Term, EvalError> {
    eval_inner(term, 0, 0).map(|(term, _, _)| term)
}

/// Evaluate the term until a lambda is at the outermost position, returning the term, step count, and non-reducing count
fn eval_head_normal(
    term: Term,
    step: usize,
    non_reducing_count: usize,
) -> Result<(Term, usize, usize), EvalError> {
    let mut term = term;
    let mut step = step;
    let mut non_reducing_count = non_reducing_count;

    loop {
        if step >= 1_000_000 {
            return Err(EvalError::StepLimitExceeded);
        }
        if non_reducing_count > 1000 {
            return Err(EvalError::Divergence);
        }
        match term.inner() {
            TermInner::Apply { func, arg } => {
                let (func, next_step, next_non_reducing_count) =
                    eval_head_normal(func.clone(), step, non_reducing_count)?;
                match func.inner() {
                    TermInner::Lambda { body, .. } => {
                        let (beta_result, non_reducing) = beta(body, arg);
                        term = beta_result;
                        step = next_step + 1;
                        non_reducing_count = if non_reducing {
                            next_non_reducing_count + 1
                        } else {
                            0
                        };
                    }
                    _ => {
                        let (reduced_arg, next_step, next_non_reducing_count) =
                            eval_inner(arg.clone(), next_step, next_non_reducing_count)?;
                        return Ok((
                            Term::apply(func, reduced_arg),
                            next_step,
                            next_non_reducing_count,
                        ));
                    }
                }
            }
            TermInner::Lambda { .. } => {
                return Ok((term, step, non_reducing_count));
            }
            TermInner::Var { .. } => return Ok((term, step, non_reducing_count)),
        }
    }
}

/// Evaluate the term to normal form, returning the final term, step count, and non-reducing count
fn eval_inner(
    term: Term,
    step: usize,
    non_reducing_count: usize,
) -> Result<(Term, usize, usize), EvalError> {
    let mut term = term;
    let mut step = step;
    let mut non_reducing_count = non_reducing_count;

    loop {
        if step >= 1_000_000 {
            return Err(EvalError::StepLimitExceeded);
        }
        if non_reducing_count > 1000 {
            return Err(EvalError::Divergence);
        }
        match term.inner() {
            TermInner::Apply { func, arg } => {
                let (func, next_step, next_non_reducing_count) =
                    eval_head_normal(func.clone(), step, non_reducing_count)?;
                match func.inner() {
                    TermInner::Lambda { body, .. } => {
                        let (beta_result, non_reducing) = beta(body, arg);
                        term = beta_result;
                        step = next_step + 1;
                        non_reducing_count = if non_reducing {
                            next_non_reducing_count + 1
                        } else {
                            0
                        };
                    }
                    _ => {
                        let (reduced_arg, next_step, next_non_reducing_count) =
                            eval_inner(arg.clone(), next_step, next_non_reducing_count)?;
                        return Ok((
                            Term::apply(func, reduced_arg),
                            next_step,
                            next_non_reducing_count,
                        ));
                    }
                }
            }
            TermInner::Lambda { param, body } => {
                let (new_body, next_step, next_non_reducing_count) =
                    eval_inner(body.clone(), step, non_reducing_count)?;
                return Ok((
                    Term::lambda(param.clone(), new_body),
                    next_step,
                    next_non_reducing_count,
                ));
            }
            TermInner::Var { .. } => return Ok((term, step, non_reducing_count)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::common::beta;

    #[test]
    fn test_subst_top_simple() {
        use crate::eval::tests::parse_and_de_bruijn;

        // Test case: (\x. \y. x) applied to (\z. z)
        // The body of the first lambda is: \y. x#1 (where x#1 refers to the outer x)
        // When we apply the lambda to (\z. z), we substitute x#1 with (\z. z)

        // First, let's create the body: \y. x where x refers to the parameter of the outer lambda
        // In de Bruijn terms: \y. x#1 (x refers 1 level up)
        let body = Term::lambda("y", Term::var("x", 1));

        // Create the argument: \z. z
        let arg = parse_and_de_bruijn(r"\z. z").unwrap();

        let (result, non_reducing) = beta(&body, &arg);
        println!("Subst top: {} with {} -> {}", body, arg, result);

        // After substitution, x#1 should be replaced with \z. z, but shifted appropriately
        // The result should be: \y. \z. z
        assert_eq!(result.to_string(), r"\y. \z. z#0");
        assert!(!non_reducing);
    }

    #[test]
    fn test_subst_top_beta_reduction() {
        use crate::eval::tests::parse_and_de_bruijn;

        // Test a more realistic beta reduction scenario
        // (\x. x) applied to (\y. y)
        // This should substitute the body (x#0) with the argument (\y. y)

        // Create a term that represents just the parameter: x#0
        let param_ref = Term::var("x", 0);

        // Create the argument: \y. y
        let arg = parse_and_de_bruijn(r"\y. y").unwrap();

        let (result, non_reducing) = beta(&param_ref, &arg);
        println!("Beta reduction: {} with {} -> {}", param_ref, arg, result);

        // The parameter reference should be replaced with the argument
        assert_eq!(result.to_string(), r"\y. y#0");
        assert!(!non_reducing);
    }
}
