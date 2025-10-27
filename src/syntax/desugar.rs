use std::path::Path;

use super::{Ast, Expr};
use crate::eval::EvalError;
use crate::import::Importer;

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
