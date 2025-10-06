use crate::ast::{Expr, Term};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Undefined variable {name}")]
    UndefinedVariable { name: String },
    #[error("Maximum evaluation steps exceeded")]
    StepLimitExceeded,
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
        Expr::Lambda(lambda) => {
            env.push(lambda.param.clone());
            let body = de_bruijn(&lambda.body, env)?;
            env.pop();
            Ok(Term::lambda(lambda.param.clone(), body))
        }
        Expr::Apply(apply) => {
            let func = de_bruijn(&apply.func, env)?;
            let arg = de_bruijn(&apply.arg, env)?;
            Ok(Term::apply(func, arg))
        }
    }
}

fn shift(term: &Term, by: isize, cutoff: usize) -> Term {
    match term {
        Term::Var { name, index } => {
            if *index >= cutoff {
                let new_index = (*index as isize + by) as usize;
                Term::var(name.clone(), new_index)
            } else {
                term.clone()
            }
        }
        Term::Lambda { param, body } => {
            let new_body = shift(body, by, cutoff + 1);
            Term::lambda(param.clone(), new_body)
        }
        Term::Apply { func, arg } => {
            let new_func = shift(func, by, cutoff);
            let new_arg = shift(arg, by, cutoff);
            Term::apply(new_func, new_arg)
        }
    }
}

fn subst(term: &Term, index: usize, value: &Term) -> Term {
    match term {
        Term::Var {
            name: _,
            index: var_index,
        } => {
            if *var_index == index {
                shift(value, index as isize, 0)
            } else {
                term.clone()
            }
        }
        Term::Lambda { param, body } => {
            let new_body = subst(body, index + 1, value);
            Term::lambda(param.clone(), new_body)
        }
        Term::Apply { func, arg } => {
            let new_func = subst(func, index, value);
            let new_arg = subst(arg, index, value);
            Term::apply(new_func, new_arg)
        }
    }
}

fn beta(body: &Term, arg: &Term) -> Term {
    let shifted_arg = shift(arg, 1, 0);
    let substituted = subst(body, 0, &shifted_arg);
    shift(&substituted, -1, 0)
}

fn step_normal(term: &Term) -> Option<Term> {
    match term.clone() {
        Term::Apply { func, arg } => match *func {
            Term::Lambda { param: _, body } => Some(beta(&body, &arg)),
            _ => {
                // Try to step the function part
                if let Some(new_func) = step_normal(&func) {
                    Some(Term::apply(new_func, *arg.clone()))
                } else if let Some(new_arg) = step_normal(&arg) {
                    Some(Term::apply((*func).clone(), new_arg))
                } else {
                    None
                }
            }
        },
        Term::Lambda { param, body } => {
            if let Some(new_body) = step_normal(&body) {
                Some(Term::lambda(param.clone(), new_body))
            } else {
                None
            }
        }
        Term::Var { .. } => None,
    }
}

pub fn eval(ast: &Expr) -> Result<Expr, EvalError> {
    let mut env = Vec::new();
    let mut term = de_bruijn(ast, &mut env)?;
    for _ in 0..10000 {
        if let Some(next) = step_normal(&term) {
            term = next;
        } else {
            return Ok(term.into());
        }
    }
    Err(EvalError::StepLimitExceeded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_de_bruijn_identity() {
        // \x. x should become \x. x#1
        let src = r"\x. x";
        let identity = parse(src).unwrap();
        let mut env = Vec::new();
        let result = de_bruijn(&identity, &mut env).unwrap();
        println!("Identity: {} -> {}", identity, result);

        assert_eq!(result.to_string(), r"\x. x#0");
    }

    #[test]
    fn test_de_bruijn_nested() {
        // \x. \y. x should become \x. \y. x#2
        let src = r"\x. \y. x";
        let nested = parse(src).unwrap();
        let mut env = Vec::new();
        let result = de_bruijn(&nested, &mut env).unwrap();
        println!("Nested: {} -> {}", nested, result);

        assert_eq!(result.to_string(), r"\x. \y. x#1");
    }

    #[test]
    fn test_de_bruijn_simple_application() {
        let src = r"(\x. x y)";
        let app = parse(src).unwrap();
        let mut env = Vec::new();
        let result = de_bruijn(&app, &mut env);

        // This should fail because y is a free variable
        assert!(result.is_err());
        if let Err(EvalError::UndefinedVariable { name }) = result {
            assert_eq!(name, "y");
        }
    }

    #[test]
    fn test_de_bruijn_with_let() {
        let src = r"let id = \x. x in id";
        let expr = parse(src).unwrap();
        let mut env = Vec::new();
        let result = de_bruijn(&expr, &mut env).unwrap();
        println!("Let expression: {} -> {}", expr, result);

        assert_eq!(result.to_string(), r"(\id. id#0 \x. x#0)");
    }

    #[test]
    fn test_de_bruijn_complex_nesting() {
        let src = r"\x. \x. x";
        let expr = parse(src).unwrap();
        let mut env = Vec::new();
        let result = de_bruijn(&expr, &mut env).unwrap();
        println!("Complex nesting: {} -> {}", expr, result);

        assert_eq!(result.to_string(), r"\x. \x. x#0");
    }

    #[test]
    fn test_subst_top_simple() {
        // Test case: (\x. \y. x) applied to (\z. z)
        // The body of the first lambda is: \y. x#1 (where x#1 refers to the outer x)
        // When we apply the lambda to (\z. z), we substitute x#1 with (\z. z)

        // First, let's create the body: \y. x where x refers to the parameter of the outer lambda
        // In de Bruijn terms: \y. x#1 (x refers 1 level up)
        let body = Term::lambda("y", Term::var("x", 1));

        // Create the argument: \z. z
        let arg_expr = parse(r"\z. z").unwrap();
        let arg = de_bruijn(&arg_expr, &mut Vec::new()).unwrap();

        let result = beta(&body, &arg);
        println!("Subst top: {} with {} -> {}", body, arg, result);

        // After substitution, x#1 should be replaced with \z. z, but shifted appropriately
        // The result should be: \y. \z. z
        assert_eq!(result.to_string(), r"\y. \z. z#0");
    }

    #[test]
    fn test_subst_top_beta_reduction() {
        // Test a more realistic beta reduction scenario
        // (\x. x) applied to (\y. y)
        // This should substitute the body (x#0) with the argument (\y. y)

        // Create a term that represents just the parameter: x#0
        let param_ref = Term::var("x", 0);

        // Create the argument: \y. y
        let arg_expr = parse(r"\y. y").unwrap();
        let arg = de_bruijn(&arg_expr, &mut Vec::new()).unwrap();

        let result = beta(&param_ref, &arg);
        println!("Beta reduction: {} with {} -> {}", param_ref, arg, result);

        // The parameter reference should be replaced with the argument
        assert_eq!(result.to_string(), r"\y. y#0");
    }

    #[test]
    fn test_eval_identity() {
        let src = r"\x. x";
        let identity = parse(src).unwrap();
        let result = eval(&identity).unwrap();
        println!("Evaluated identity: {} -> {}", identity, result);
        assert_eq!(result.to_string(), r"\x . x");
    }

    #[test]
    fn test_eval_application() {
        // Test let id = \x. x in let f = \y. y in (id f)
        // This should reduce to \y. y
        let src = r"let id = \x. x in let f = \y. y in (id f)";
        let app = parse(src).unwrap();
        let result = eval(&app).unwrap();
        println!("Evaluated application: {} -> {}", app, result);
        assert_eq!(result.to_string(), r"\y . y");
    }

    #[test]
    fn test_eval_church_numeral_zero() {
        // Church numeral 0: \f. \x. x
        let src = r"\f. \x. x";
        let zero = parse(src).unwrap();
        let result = eval(&zero).unwrap();
        println!("Evaluated church 0: {} -> {}", zero, result);
        assert_eq!(result.to_string(), r"\f . \x . x");
    }

    #[test]
    fn test_eval_church_numeral_one() {
        // Church numeral 1: \f. \x. (f x)
        let src = r"\f. \x. (f x)";
        let one = parse(src).unwrap();
        let result = eval(&one).unwrap();
        println!("Evaluated church 1: {} -> {}", one, result);
        assert_eq!(result.to_string(), r"\f . \x . (f x)");
    }

    #[test]
    fn test_eval_higher_order_function() {
        // Test composition: let twice = \f. \x. (f (f x)) in let id = \y. y in (twice id)
        let src = r"let twice = \f. \x. (f (f x)) in let id = \y. y in (twice id)";
        let hof = parse(src).unwrap();
        let result = eval(&hof).unwrap();
        println!("Evaluated higher-order: {} -> {}", hof, result);
        assert_eq!(result.to_string(), r"\x . x");
    }

    #[test]
    fn test_eval_let_expression() {
        // Test let id = \x. x in (id (\y. y)) but expressed differently
        let src = r"let id = \x. x in let f = \y. y in (id f)";
        let let_expr = parse(src).unwrap();
        let result = eval(&let_expr).unwrap();
        println!("Evaluated let expression: {} -> {}", let_expr, result);
        assert_eq!(result.to_string(), r"\y . y");
    }

    #[test]
    fn test_eval_k_combinator() {
        // Test K combinator: \x. \y. x applied to something
        let src = r"let k = \x. \y. x in let a = \z. z in (k a)";
        let k_app = parse(src).unwrap();
        let result = eval(&k_app).unwrap();
        println!("Evaluated K combinator: {} -> {}", k_app, result);
        assert_eq!(result.to_string(), r"\y . \z . z");
    }

    #[test]
    fn test_eval_s_combinator_partial() {
        // Test S combinator (partial application): \x. \y. \z. ((x z) (y z))
        let src = r"\x. \y. \z. ((x z) (y z))";
        let s_comb = parse(src).unwrap();
        let result = eval(&s_comb).unwrap();
        println!("Evaluated S combinator: {} -> {}", s_comb, result);
        assert_eq!(result.to_string(), r"\x . \y . \z . ((x z) (y z))");
    }

    #[test]
    fn test_eval_currying_example() {
        // Test currying: let add = \x. \y. \f. \z. (f ((x f) ((y f) z))) in (add (\f. \x. (f x)))
        // This is a more complex example showing how curried functions work
        let src = r"let const = \x. \y. x in let one = \f. \x. (f x) in (const one)";
        let curry = parse(src).unwrap();
        let result = eval(&curry).unwrap();
        println!("Evaluated currying: {} -> {}", curry, result);
        assert_eq!(result.to_string(), r"\y . \f . \x . (f x)");
    }

    #[test]
    fn test_eval_complex_composition() {
        // Test function composition: let comp = \f. \g. \x. (f (g x)) in comp
        let src = r"let comp = \f. \g. \x. (f (g x)) in comp";
        let composition = parse(src).unwrap();
        let result = eval(&composition).unwrap();
        println!("Evaluated composition: {} -> {}", composition, result);
        assert_eq!(result.to_string(), r"\f . \g . \x . (f (g x))");
    }
}
