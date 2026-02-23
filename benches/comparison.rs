use criterion::{criterion_group, criterion_main, AxisScale, BenchmarkGroup, BenchmarkId, Criterion, PlotConfiguration};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::hint::black_box;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use criterion::measurement::Measurement;
use rust_102::implementations::parallel::step_parallel;
use rust_102::implementations::pool::{initialise_pool, step_pool};
use rust_102::implementations::serial::step_serial;
use rust_102::implementations::workers::step_workers;

fn make_seeded(width: usize, height: usize) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(0xDEADBEEF);
    let mut buf = vec![0u8; width * height];
    for v in &mut buf {
        *v = rng.random::<u8>() & 1;
    }
    buf
}

fn concurrent_group<'a, M: Measurement>(criterion: &'a mut Criterion<M>, name: &str) -> BenchmarkGroup<'a, M> {
    let mut group = criterion.benchmark_group(name);
    group.measurement_time(Duration::from_secs(10));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.warm_up_time(Duration::from_secs(1));
    group
}

fn criterion_benchmark(c: &mut Criterion) {
    let width = 800usize;
    let height = 600usize;
    let total = width * height;

    let mut group = c.benchmark_group("step_serial");

    group.bench_function("step_serial", |b| {
        b.iter_batched(
            || {
                let curr_buffer = make_seeded(width, height);
                let next_buffer = vec![0u8; total];
                (curr_buffer, next_buffer)
            },
            |(curr_buffer, mut next_buffer)| {
                step_serial(
                    black_box(&curr_buffer),
                    black_box(&mut next_buffer),
                    width,
                    height,
                )
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
    group = concurrent_group(c, "parallel");

    for num_threads in (1u32..=7).map(|k| 1usize << k) {
        group.bench_with_input(
            BenchmarkId::new("num_threads", num_threads),
            &num_threads,
            |bencher, num_threads| {
                bencher.iter_batched(
                    || {
                        let curr_buffer = make_seeded(width, height);
                        let next_buffer = vec![0u8; total];
                        (curr_buffer, next_buffer)
                    },
                    |(curr_buffer, mut next_buffer)| {
                        step_parallel(
                            black_box(&curr_buffer),
                            black_box(&mut next_buffer),
                            *num_threads,
                            width,
                            height,
                        )
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
    group = concurrent_group(c, "workers");

    for chunk_size in (1u32..=12).map(|k| 1usize << k) {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            &chunk_size,
            |bencher, chunk_size| {
                bencher.iter_batched(
                    || {
                        let curr_buffer = make_seeded(width, height);
                        let next_buffer = vec![0u8; total];
                        (curr_buffer, next_buffer)
                    },
                    |(curr_buffer, mut next_buffer)| {
                        step_workers(
                            black_box(&curr_buffer),
                            black_box(&mut next_buffer),
                            8,
                            *chunk_size,
                            width,
                            height,
                        )
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
    let mut group = concurrent_group(c, "pool");

    for chunk_size in (1u32..=12).map(|k| 1usize << k) {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            &chunk_size,
            |bencher, &chunk_size| {
                let curr_buffer = Arc::new(RwLock::new(make_seeded(width, height)));
                let next_buffer = Arc::new(Mutex::new(vec![0u8; total]));
                let pool = initialise_pool(
                    Arc::clone(&curr_buffer),
                    Arc::clone(&next_buffer),
                    8,
                    chunk_size,
                    width,
                    height,
                );

                bencher.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        step_pool(
                            black_box(&pool),
                            black_box(&curr_buffer),
                            black_box(&next_buffer),
                        );
                    }
                    start.elapsed()
                });

                drop(pool);
            },
        );
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
