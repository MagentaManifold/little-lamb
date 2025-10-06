mod ast;
mod eval;
mod parser;

use eval::eval;
use parser::parse;

fn main() {
    let usage = "Run `cargo run -- examples/example.lil`";
    let src = std::fs::read_to_string(std::env::args().nth(1).expect(usage)).expect(usage);

    match parse(&src) {
        Ok(ast) => match eval(&ast) {
            Ok(output) => println!("{output}"),
            Err(eval_err) => {
                println!("Evaluation error: {}", eval_err);
            }
        },
        Err(parse_err) => {
            println!("Parse error: {}", parse_err);
        }
    };
}
