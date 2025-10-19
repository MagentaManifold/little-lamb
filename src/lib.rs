pub mod ast;
pub mod eval;
pub mod import;
pub mod lexer;
pub mod parser;

use std::path::Path;

use anyhow::Error;
use eval::{desugar, eval};
use lexer::tokenize;
use parser::parse;

use crate::{
    ast::{Expr, Term},
    import::Importer,
};

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
    let result = eval(term)?;
    let expr = Expr::from(result);
    Ok(expr)
}

pub fn execute_string(src: &str) -> anyhow::Result<Expr> {
    let tokens = tokenize(src)?;
    let ast = parse(&tokens)?;
    let expr = desugar(ast, &mut Importer::new(), None)?;
    let term = Term::try_from(expr)?;
    let result = eval(term)?;
    let expr = Expr::from(result);
    Ok(expr)
}
