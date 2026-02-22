use criterion::{criterion_group, criterion_main, Criterion};
use rust_102::game_of_life::{step_parallel, step_serial};
use rust_102::seed::seed;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let mut curr_buffer = vec![false; 800 * 600];
    let mut next_buffer = vec![false; 800 * 600];
    seed(&mut curr_buffer, 800, 600);

    c.bench_function("step_serial", |b| {
        b.iter_batched(
            || {},
            |_| step_serial(black_box(&curr_buffer), black_box(&mut next_buffer), 800, 600),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("step_parallel", |b| {
        b.iter_batched(
            || {},
            |_| step_parallel(black_box(&curr_buffer), black_box(&mut next_buffer), &mut 0, 800, 600, 32),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
