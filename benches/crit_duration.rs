use core::str::FromStr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hifitime::{Duration, Unit};

#[allow(unused_must_use)]
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "Convert f64 duration to hifitime duration and extract nanoseconds",
        |b| {
            b.iter(|| {
                let interval_length_s: f64 = 6311433599.999999;
                let interval_length: Duration = black_box(interval_length_s * Unit::Second);
                interval_length.to_parts().1;
            })
        },
    );
    c.bench_function("Parse easy duration", |b| {
        b.iter(|| Duration::from_str("15 d").unwrap())
    });
    c.bench_function("Parse complex duration", |b| {
        b.iter(|| Duration::from_str("1 d 15.5 hours 25 ns").unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
