mod ast;
mod eval;
mod import;
mod lexer;
mod parser;

use std::path::Path;

use eval::{desugar, eval};
use lexer::tokenize;
use parser::parse;

use crate::{
    ast::{Expr, Term},
    import::Importer,
};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let usage = "Run `cargo run -- examples/example.lil`";
    let file_path = args.get(1).expect(usage);
    let file_dir = Path::new(file_path)
        .parent()
        .expect(&format!("Failed to get parent directory of {file_path}"));
    let src = std::fs::read_to_string(args.get(1).expect(usage)).expect(usage);
    let result = execute(&src, file_dir)?;

    println!("{}", result);
    Ok(())
}

fn execute(src: &str, file_dir: &Path) -> anyhow::Result<Expr> {
    let tokens = tokenize(src)?;
    let ast = parse(&tokens)?;
    let expr = desugar(ast, &mut Importer::new(), Some(file_dir))?;
    let term = Term::try_from(expr)?;
    let result = eval(term)?;
    let expr = Expr::from(result);
    Ok(expr)
}
