use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use little_lamb::{eval, execute_with_strategy};

fn substitution_benchmark(c: &mut Criterion) {
    let fib7_src = include_str!("fib7.lil");
    let factorial7_src = include_str!("factorial7.lil");

    c.bench_function("substitution/fib 7", |b| {
        b.iter(|| {
            execute_with_strategy(black_box(fib7_src), eval::substitution_eval)
                .expect("Execution failed")
        })
    });

    c.bench_function("substitution/factorial 7", |b| {
        b.iter(|| {
            execute_with_strategy(black_box(factorial7_src), eval::substitution_eval)
                .expect("Execution failed")
        })
    });
}

criterion_group!(benches, substitution_benchmark);
criterion_main!(benches);
