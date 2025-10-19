use std::path::Path;

use crate::{
    ast::{Ast, Expr, Term, TermInner},
    import::ImportError,
    import::Importer,
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

/// Desugar an AST into an Expr by converting Let bindings to lambda applications
pub fn desugar(
    ast: Ast,
    importer: &mut Importer,
    file_dir: Option<&Path>,
) -> Result<Expr, EvalError> {
    match ast {
        Ast::Var(name) => Ok(Expr::var(name)),
        Ast::Nat(num) => Ok(desugar_nat(num)),
        Ast::Lambda { param, body } => Ok(Expr::lambda(param, desugar(*body, importer, file_dir)?)),
        Ast::Apply { func, arg } => Ok(Expr::apply(
            desugar(*func, importer, file_dir)?,
            desugar(*arg, importer, file_dir)?,
        )),
        Ast::Let { name, value, body } => Ok(desugar_let(
            name,
            desugar(*value, importer, file_dir)?,
            desugar(*body, importer, file_dir)?,
        )),
        Ast::Import { module, name, body } => {
            let imported_ast = importer.import(&module, file_dir)?;
            Ok(desugar_let(
                name,
                imported_ast,
                desugar(*body, importer, file_dir)?,
            ))
        }
    }
}

fn desugar_nat(num: usize) -> Expr {
    Expr::lambda(
        "f",
        Expr::lambda(
            "x",
            (0..num).fold(Expr::var("x"), |acc, _| Expr::apply(Expr::var("f"), acc)),
        ),
    )
}

fn desugar_let(name: String, value: Expr, body: Expr) -> Expr {
    Expr::apply(Expr::lambda(name, body), value)
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

fn shift(term: &Term, by: isize, cutoff: usize) -> Term {
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
            Term::lambda(param.to_string(), new_body)
        }
        TermInner::Apply { func, arg } => {
            let new_func = shift(func, by, cutoff);
            let new_arg = shift(arg, by, cutoff);
            Term::apply(new_func, new_arg)
        }
    }
}

fn subst(term: &Term, index: usize, value: &Term) -> Term {
    match term.inner() {
        TermInner::Var {
            name: _,
            index: var_index,
        } => {
            if *var_index == index {
                shift(value, index as isize, 0)
            } else {
                term.clone()
            }
        }
        TermInner::Lambda { param, body } => {
            let new_body = subst(body, index + 1, value);
            Term::lambda(param.to_string(), new_body)
        }
        TermInner::Apply { func, arg } => {
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
    match term.inner() {
        TermInner::Apply { func, arg } => match func.inner() {
            TermInner::Lambda { param: _, body } => Some(beta(body, arg)),
            _ => {
                if let Some(new_func) = step_normal(func) {
                    Some(Term::apply(new_func, arg.clone()))
                } else if let Some(new_arg) = step_normal(arg) {
                    Some(Term::apply(func.clone(), new_arg))
                } else {
                    None
                }
            }
        },
        TermInner::Lambda { param, body } => {
            if let Some(new_body) = step_normal(body) {
                Some(Term::lambda(param.to_string(), new_body))
            } else {
                None
            }
        }
        TermInner::Var { .. } => None,
    }
}

pub fn eval(mut term: Term) -> Result<Term, EvalError> {
    let mut prev_size = term_size(&term);
    let mut non_reducing_count: usize = 0;

    for _ in 0..1000000 {
        if let Some(next) = step_normal(&term) {
            term = next;

            let current_size = term_size(&term);
            if current_size >= prev_size {
                non_reducing_count += 1;
                if non_reducing_count > 1000 {
                    return Err(EvalError::Divergence);
                }
            } else {
                non_reducing_count = 0;
            }
            prev_size = current_size;
        } else {
            return Ok(term);
        }
    }
    Err(EvalError::StepLimitExceeded)
}

/// Helper function to estimate term size
fn term_size(term: &Term) -> usize {
    match term.inner() {
        TermInner::Var { .. } => 1,
        TermInner::Lambda { body, .. } => 1 + term_size(body),
        TermInner::Apply { func, arg } => 1 + term_size(func) + term_size(arg),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    /// Helper function to parse and convert to de Bruijn indices
    fn parse_and_de_bruijn(src: &str) -> Result<Term, EvalError> {
        let tokens = tokenize(src).unwrap();
        let ast = parse(&tokens).unwrap();
        let expr = desugar(ast, &mut Importer::new(), Some(Path::new(".")))?;
        de_bruijn(&expr, &mut Vec::new())
    }

    /// Helper function to parse and evaluate an expression
    fn src_eval(src: &str) -> Result<Term, EvalError> {
        let tokens = tokenize(src).unwrap();
        let ast = parse(&tokens).unwrap();
        let expr = desugar(ast, &mut Importer::new(), Some(Path::new(".")))?;
        let term = de_bruijn(&expr, &mut Vec::new())?;
        eval(term)
    }

    #[test]
    fn test_desugar_zero() {
        let src = r"0";
        let result = parse_and_de_bruijn(src).unwrap();
        println!("Desugared 0: {} -> {}", src, result);
        let expected = parse_and_de_bruijn(r"\f x. x").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_desugar_two() {
        let src = r"2";
        let result = parse_and_de_bruijn(src).unwrap();
        println!("Desugared 2: {} -> {}", src, result);
        let expected = parse_and_de_bruijn(r"\f x. (f (f x))").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_de_bruijn_identity() {
        // \x. x should become \x. x#1
        let src = r"\x. x";
        let result = parse_and_de_bruijn(src).unwrap();
        println!("Identity: {} -> {}", src, result);

        assert_eq!(result.to_string(), r"\x. x#0");
    }

    #[test]
    fn test_de_bruijn_nested() {
        // \x. \y. x should become \x. \y. x#2
        let src = r"\x. \y. x";
        let result = parse_and_de_bruijn(src).unwrap();
        println!("Nested: {} -> {}", src, result);

        assert_eq!(result.to_string(), r"\x. \y. x#1");
    }

    #[test]
    fn test_de_bruijn_simple_application() {
        let src = r"(\x. x y)";
        let result = parse_and_de_bruijn(src);

        assert!(result.is_err());
        if let Err(EvalError::UndefinedVariable { name }) = result {
            assert_eq!(name, "y");
        }
    }

    #[test]
    fn test_de_bruijn_with_let() {
        let src = r"let id = \x. x in id";
        let result = parse_and_de_bruijn(src).unwrap();
        println!("Let expression: {} -> {}", src, result);

        assert_eq!(result.to_string(), r"(\id. id#0) (\x. x#0)");
    }

    #[test]
    fn test_de_bruijn_complex_nesting() {
        let src = r"\x. \x. x";
        let result = parse_and_de_bruijn(src).unwrap();
        println!("Complex nesting: {} -> {}", src, result);

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
        let arg = parse_and_de_bruijn(r"\z. z").unwrap();

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
        let arg = parse_and_de_bruijn(r"\y. y").unwrap();

        let result = beta(&param_ref, &arg);
        println!("Beta reduction: {} with {} -> {}", param_ref, arg, result);

        // The parameter reference should be replaced with the argument
        assert_eq!(result.to_string(), r"\y. y#0");
    }

    #[test]
    fn test_eval_identity() {
        let src = r"\x. x";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\x. x").unwrap();
        println!("Evaluated identity: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_application() {
        let src = r"let id = \x. x in let f = \y. y in (id f)";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\y. y").unwrap();
        println!("Evaluated application: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_church_numeral_zero() {
        let src = r"\f x. x";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\f x. x").unwrap();
        println!("Evaluated church 0: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_church_numeral_one() {
        let src = r"\f x. (f x)";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\f x. (f x)").unwrap();
        println!("Evaluated church 1: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_higher_order_function() {
        let src = r"let twice = \f x. (f (f x)) in let id = \y. y in (twice id)";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\x. x").unwrap();
        println!("Evaluated higher-order: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_let_expression() {
        let src = r"let id = \x. x in let f = \y. y in (id f)";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\y. y").unwrap();
        println!("Evaluated let expression: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_k_combinator() {
        let src = r"let k = \x y. x in let a = \z. z in (k a)";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\y. \z. z").unwrap();
        println!("Evaluated K combinator: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_s_combinator_partial() {
        let src = r"\x y z. (x z) (y z)";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\x y z. ((x z) (y z))").unwrap();
        println!("Evaluated S combinator: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_currying_example() {
        let src = r"
            let const = \x y. x in
            let one = \f x. (f x) in
            const one
        ";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\y. \f x. (f x)").unwrap();
        println!("Evaluated currying: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_complex_composition() {
        let src = r"let comp = \f g x. (f (g x)) in comp";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\f g x. (f (g x))").unwrap();
        println!("Evaluated composition: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_import_as_builtin() {
        let src = r"import I as id in id";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\x. x").unwrap();
        println!("Evaluated import: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_import_as_builtin_boolean() {
        let src = r"
            import true, false, and, or, not in
            and (or false true) (not true)
        ";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"\x y . y").unwrap();
        println!("Evaluated import: {} -> {}", src, result);
        assert_eq!(result, expected);
    }
    #[test]
    fn test_lib_pred() {
        let src = r"import pred in pred 3";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"2").unwrap();
        println!("Evaluated: {} -> {}", src, result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lib_sub() {
        let src = r"import sub in sub 5 3";
        let result = src_eval(src).unwrap();
        let expected = parse_and_de_bruijn(r"2").unwrap();
        println!("Evaluated: {} -> {}", src, result);
        assert_eq!(result, expected);
    }
}
