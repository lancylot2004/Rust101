use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use image::ImageReader;
use picture_lib::invert::{invert_basic, invert_concurrently};

fn criterion_benchmark(c: &mut Criterion) {
    // Load test image
    let img = ImageReader::open("test.jpg")
        .expect("Failed to open test.jpg")
        .decode()
        .expect("Failed to decode image")
        .to_rgb8();

    c.bench_function("invert_basic", |b| {
        b.iter_batched(
            || img.clone(),
            |mut img_copy| invert_basic(black_box(&mut img_copy)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("invert_concurrent", |b| {
        b.iter_batched(
            || img.clone(),
            |mut img_copy| invert_concurrently(black_box(&mut img_copy)),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
