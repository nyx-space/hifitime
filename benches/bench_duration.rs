extern crate criterion;
extern crate hifitime;
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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
