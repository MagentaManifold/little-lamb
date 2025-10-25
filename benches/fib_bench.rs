use little_lamb::execute_string;
use std::time::Instant;

fn main() {
    let src = include_str!("fib7.lil");

    println!("Running fib 7 benchmark...");

    let start = Instant::now();

    let result = execute_string(src).expect("Execution failed");

    let duration = start.elapsed();

    println!("Result: {}", result);
    println!("Benchmark complete!");
    println!("Time: {:.2?}\n", duration);

    let src = include_str!("fib9.lil");

    println!("Running fib 9 benchmark...");

    let start = Instant::now();

    let result = execute_string(src).expect("Execution failed");

    let duration = start.elapsed();

    println!("Result: {}", result);
    println!("Benchmark complete!");
    println!("Time: {:.2?}", duration);
}
