use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use little_lamb::{eval, execute_with_strategy};

fn kn_benchmark(c: &mut Criterion) {
    let fib7_src = include_str!("fib7.lil");
    let fib9_src = include_str!("fib9.lil");
    let factorial7_src = include_str!("factorial7.lil");
    let factorial8_src = include_str!("factorial8.lil");

    c.bench_function("kn/fib 7", |b| {
        b.iter(|| {
            execute_with_strategy(black_box(fib7_src), eval::kn_eval).expect("Execution failed")
        })
    });

    c.bench_function("kn/fib 9", |b| {
        b.iter(|| {
            execute_with_strategy(black_box(fib9_src), eval::kn_eval).expect("Execution failed")
        })
    });

    c.bench_function("kn/factorial 7", |b| {
        b.iter(|| {
            execute_with_strategy(black_box(factorial7_src), eval::kn_eval)
                .expect("Execution failed")
        })
    });

    c.bench_function("kn/factorial 8", |b| {
        b.iter(|| {
            execute_with_strategy(black_box(factorial8_src), eval::kn_eval)
                .expect("Execution failed")
        })
    });
}

criterion_group!(benches, kn_benchmark);
criterion_main!(benches);
