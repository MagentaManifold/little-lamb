use std::time::Instant;

fn main() {
    let src = include_str!("factorial7.lil");

    println!("Running factorial 7 benchmark...\n");

    let start = Instant::now();

    use little_lamb::execute_string;
    let result = execute_string(src).expect("Execution failed");

    let duration = start.elapsed();

    println!("Result: {}", result);
    println!("\nBenchmark complete!");
    println!("Time: {:.2?}", duration);
}
