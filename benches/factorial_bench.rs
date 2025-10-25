use little_lamb::execute_string;
use std::time::Instant;

fn main() {
    let src = include_str!("factorial7.lil");

    println!("Running factorial 7 benchmark...");

    let start = Instant::now();

    let _result = execute_string(src).expect("Execution failed");

    let duration = start.elapsed();

    // println!(
    //     "Result: {}...",
    //     result.to_string()[..200.min(result.to_string().len())].to_string()
    // );
    println!("Benchmark complete!");
    println!("Time: {:.2?}\n", duration);

    let src = include_str!("factorial8.lil");

    // lci> let fact = Y (\f. \x. (IsZero x) 1 (Mult (f(Pred x)) x)) in fact 8
    // 40320
    // (176628 reductions, 14.05s CPU)
    // little-lamb is faster than lci! (?)
    println!("Running factorial 8 benchmark...");

    let start = Instant::now();

    let _result = execute_string(src).expect("Execution failed");

    let duration = start.elapsed();

    // println!(
    //     "Result: {}...",
    //     result.to_string()[..200.min(result.to_string().len())].to_string()
    // );
    println!("Benchmark complete!");
    println!("Time: {:.2?}", duration);
}
