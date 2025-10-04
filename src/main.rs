mod ast;
mod eval;
mod parser;

use chumsky::Parser;
use eval::eval;
use parser::parser;

fn main() {
    let usage = "Run `cargo run -- examples/example.lil`";
    let src = std::fs::read_to_string(std::env::args().nth(1).expect(usage)).expect(usage);

    match parser().parse(&src).into_result() {
        Ok(ast) => match eval(&ast) {
            Ok(output) => println!("{output}"),
            Err(eval_err) => {
                println!("Evaluation error: {}", eval_err);
            }
        },
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|err| println!("Parse error: {err}")),
    };
}
