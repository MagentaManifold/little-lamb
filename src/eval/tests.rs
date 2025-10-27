use std::path::Path;

use super::common::{EvalError, de_bruijn};
use crate::import::Importer;
use crate::lexer::tokenize;
use crate::parser::parse;
use crate::syntax::{Term, desugar};

/// Helper function to parse and convert to de Bruijn indices
pub fn parse_and_de_bruijn(src: &str) -> Result<Term, EvalError> {
    let tokens = tokenize(src).unwrap();
    let ast = parse(&tokens).unwrap();
    let expr = desugar(ast, &mut Importer::new(), Some(Path::new(".")))?;
    de_bruijn(&expr, &mut Vec::new())
}

/// Macro to generate tests for all evaluation strategies
macro_rules! test_all_strategies {
    ($test_name:ident, $test_body:expr) => {
        paste::paste! {
            #[test]
            fn [<$test_name _substitution>]() {
                let eval_fn = |term| super::substitution::eval(term);
                $test_body(eval_fn);
            }

            #[test]
            fn [<$test_name _krivine>]() {
                let eval_fn = |term| super::krivine::eval(term);
                $test_body(eval_fn);
            }

            #[test]
            fn [<$test_name _kn>]() {
                let eval_fn = |term| super::kn::eval(term);
                $test_body(eval_fn);
            }
        }
    };
}

// Tests for de_bruijn conversion (not strategy-specific)
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
    // \x. x should become \x. x#0
    let src = r"\x. x";
    let result = parse_and_de_bruijn(src).unwrap();
    println!("Identity: {} -> {}", src, result);

    assert_eq!(result.to_string(), r"\x. x#0");
}

#[test]
fn test_de_bruijn_nested() {
    // \x. \y. x should become \x. \y. x#1
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

// Strategy-parameterized tests
test_all_strategies!(test_eval_identity, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"\x. x";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\x. x").unwrap();
    println!("Evaluated identity: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_application, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"let id = \x. x in let f = \y. y in (id f)";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\y. y").unwrap();
    println!("Evaluated application: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_church_numeral_zero, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"\f x. x";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\f x. x").unwrap();
    println!("Evaluated church 0: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_church_numeral_one, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"\f x. (f x)";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\f x. (f x)").unwrap();
    println!("Evaluated church 1: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_higher_order_function, |eval_fn: fn(
    Term,
)
    -> Result<
    Term,
    EvalError,
>| {
    let src = r"let twice = \f x. (f (f x)) in let id = \y. y in (twice id)";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\x. x").unwrap();
    println!("Evaluated higher-order: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_let_expression, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"let id = \x. x in let f = \y. y in (id f)";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\y. y").unwrap();
    println!("Evaluated let expression: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_k_combinator, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"let k = \x y. x in let a = \z. z in (k a)";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\y. \z. z").unwrap();
    println!("Evaluated K combinator: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_s_combinator_partial, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"\x y z. (x z) (y z)";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\x y z. ((x z) (y z))").unwrap();
    println!("Evaluated S combinator: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_currying_example, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"
        let const = \x y. x in
        let one = \f x. (f x) in
        const one
    ";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\y. \f x. (f x)").unwrap();
    println!("Evaluated currying: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_complex_composition, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"let comp = \f g x. (f (g x)) in comp";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\f g x. (f (g x))").unwrap();
    println!("Evaluated composition: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_eval_import_as_builtin, |eval_fn: fn(
    Term,
) -> Result<
    Term,
    EvalError,
>| {
    let src = r"import I as id in id";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"\x. x").unwrap();
    println!("Evaluated import: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(
    test_eval_import_as_builtin_boolean,
    |eval_fn: fn(Term) -> Result<Term, EvalError>| {
        let src = r"
        import true, false, and, or, not in
        and (or false true) (not true)
    ";
        let term = parse_and_de_bruijn(src).unwrap();
        let result = eval_fn(term).unwrap();
        let expected = parse_and_de_bruijn(r"\x y . y").unwrap();
        println!("Evaluated import: {} -> {}", src, result);
        assert_eq!(result, expected);
    }
);

test_all_strategies!(test_lib_pred, |eval_fn: fn(
    Term,
) -> Result<Term, EvalError>| {
    let src = r"import pred in pred 3";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"2").unwrap();
    println!("Evaluated: {} -> {}", src, result);
    assert_eq!(result, expected);
});

test_all_strategies!(test_lib_sub, |eval_fn: fn(
    Term,
) -> Result<Term, EvalError>| {
    let src = r"import sub in sub 5 3";
    let term = parse_and_de_bruijn(src).unwrap();
    let result = eval_fn(term).unwrap();
    let expected = parse_and_de_bruijn(r"2").unwrap();
    println!("Evaluated: {} -> {}", src, result);
    assert_eq!(result, expected);
});
