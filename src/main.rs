use clap::Parser;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "little-lamb")]
#[command(about = "An untyped lambda calculus interpreter", long_about = None)]
struct Cli {
    /// Path to .lil file to execute. Reads from stdin if not provided
    file: Option<PathBuf>,

    /// Maximum depth for pretty printing output
    #[arg(short('d'), long("depth"), default_value_t = 20)]
    depth: usize,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = if let Some(file_path) = cli.file {
        little_lamb::execute_file(&file_path)?
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        // Trim null bytes at the end
        let input = input.trim_end_matches('\0');
        little_lamb::execute_string(input)?
    };

    println!("{}", result.to_readable_string_with_depth(cli.depth));
    Ok(())
}
