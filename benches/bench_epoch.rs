extern crate criterion;
extern crate hifitime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hifitime::{Duration, Epoch, TimeUnit};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("TBD seconds and JDE ET", |b| {
        b.iter(|| {
            let e = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
            e.as_tdb_seconds();
            e.as_jde_et_days();

            let f: Epoch = e + black_box(50) * TimeUnit::Second;
            f.as_tdb_seconds();
            f.as_jde_et_days();
        })
    });

    c.bench_function("Duration to f64 seconds", |b| {
        b.iter(|| {
            let d: Duration = TimeUnit::Second * black_box(3.0);
            d.in_seconds();
        })
    });

    c.bench_function("Duration add and assert day hour", |b| {
        b.iter(|| {
            assert_eq!(
                TimeUnit::Day * black_box(10.0),
                TimeUnit::Day * black_box(10)
            );
            assert_eq!(
                TimeUnit::Hour * black_box(-7.0),
                TimeUnit::Hour * black_box(-7)
            );
        })
    });

    c.bench_function("Duration add and assert minute second", |b| {
        b.iter(|| {
            assert_eq!(
                TimeUnit::Minute * black_box(-2.0),
                TimeUnit::Minute * black_box(-2)
            );
            assert_eq!(
                TimeUnit::Second * black_box(3.0),
                TimeUnit::Second * black_box(3)
            );
        })
    });

    c.bench_function("Duration add and assert subsecons", |b| {
        b.iter(|| {
            assert_eq!(
                TimeUnit::Millisecond * black_box(4.0),
                TimeUnit::Millisecond * black_box(4)
            );
            assert_eq!(
                TimeUnit::Nanosecond * black_box(5.0),
                TimeUnit::Nanosecond * black_box(5)
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
