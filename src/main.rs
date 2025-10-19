use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let usage = "Run `cargo run -- examples/example.lil`";
    let file_path = Path::new(args.get(1).expect(usage));

    let result = little_lamb::execute_file(file_path)?;

    println!("{}", result);
    Ok(())
}
