use core::str::FromStr;
use criterion::{criterion_group, criterion_main, Criterion};
use hifitime::{Duration, Unit};
use std::hint::black_box;

#[allow(unused_must_use)]
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "Convert f64 duration to hifitime duration and extract nanoseconds",
        |b| {
            b.iter(|| {
                let interval_length_s: f64 = 6311433599.999999;
                let interval_length: Duration = black_box(interval_length_s * Unit::Second);
                interval_length.to_parts();
            })
        },
    );
    c.bench_function("Parse easy duration", |b| {
        b.iter(|| Duration::from_str("15 d").unwrap())
    });
    c.bench_function("Parse complex duration", |b| {
        b.iter(|| Duration::from_str("1 d 15.5 hours 25 ns").unwrap())
    });

    c.bench_function("Duration::from_seconds", |b| {
        b.iter(|| Duration::from_seconds(black_box(12345.6789)))
    });

    c.bench_function("Duration * f64 (scaling)", |b| {
        let d = Duration::from_seconds(12345.6789);
        b.iter(|| black_box(d) * black_box(2.5))
    });

    c.bench_function("Duration / f64 (scaling)", |b| {
        let d = Duration::from_seconds(12345.6789);
        b.iter(|| black_box(d) / black_box(2.5))
    });

    c.bench_function("f64 -> Duration -> f64 round trip", |b| {
        b.iter(|| Duration::from_seconds(black_box(12345.6789)).to_seconds())
    });

    c.bench_function("Duration * f64 (power of two)", |b| {
        let d = Duration::from_seconds(12345.6789);
        b.iter(|| black_box(d) * black_box(2.0))
    });

    c.bench_function("Duration / f64 (power of two)", |b| {
        let d = Duration::from_seconds(12345.6789);
        b.iter(|| black_box(d) / black_box(2.0))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
