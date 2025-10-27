pub mod eval;
pub mod import;
pub mod lexer;
pub mod parser;
pub mod syntax;

use std::path::Path;

use anyhow::Error;
use import::Importer;
use lexer::tokenize;
use parser::parse;
use syntax::desugar;

pub use syntax::{Expr, Term};

use crate::eval::common::EvalStrategy;

pub fn execute_file(file_path: &Path) -> anyhow::Result<Expr> {
    let file_dir = file_path.parent().ok_or_else(|| {
        Error::msg(format!(
            "Failed to get parent directory of {}",
            file_path.display()
        ))
    })?;
    let src = std::fs::read_to_string(file_path)?;
    let tokens = tokenize(&src)?;
    let ast = parse(&tokens)?;
    let expr = desugar(ast, &mut Importer::new(), Some(file_dir))?;
    let term = Term::try_from(expr)?;
    let result = eval::eval(term)?;
    let expr = Expr::from(result);
    Ok(expr)
}

pub fn execute_string(src: &str) -> anyhow::Result<Expr> {
    let tokens = tokenize(src)?;
    let ast = parse(&tokens)?;
    let expr = desugar(ast, &mut Importer::new(), None)?;
    let term = Term::try_from(expr)?;
    let result = eval::eval(term)?;
    let expr = Expr::from(result);
    Ok(expr)
}

pub fn execute_with_strategy(src: &str, eval_fn: EvalStrategy) -> anyhow::Result<Expr> {
    let tokens = tokenize(src)?;
    let ast = parse(&tokens)?;
    let expr = desugar(ast, &mut Importer::new(), None)?;
    let term = Term::try_from(expr)?;
    let result = eval_fn(term)?;
    let expr = Expr::from(result);
    Ok(expr)
}
