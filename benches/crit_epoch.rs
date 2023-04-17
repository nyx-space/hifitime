use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hifitime::{Duration, Epoch, Unit};

#[allow(unused_must_use)]
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("TBD seconds and JDE ET", |b| {
        b.iter(|| {
            let e = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
            black_box(Epoch::from_tdb_seconds(e.to_tdb_seconds()));
            black_box(Epoch::from_jde_et(e.to_jde_et_days()));

            let f: Epoch = e + black_box(50) * Unit::Second;
            black_box(f.to_tdb_seconds());
            black_box(f.to_jde_et_days());
        })
    });

    c.bench_function("TT", |b| {
        b.iter(|| {
            let e = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
            e.to_tt_seconds();

            let f: Epoch = e + black_box(50) * Unit::Second;
            f.to_tt_seconds();
        })
    });

    c.bench_function("Duration to f64 seconds", |b| {
        b.iter(|| {
            let d: Duration = Unit::Second * black_box(3.0);
            d.to_seconds();
        })
    });

    c.bench_function("Duration add and assert day hour", |b| {
        b.iter(|| {
            assert_eq!(Unit::Day * black_box(10.0), Unit::Day * black_box(10));
            assert_eq!(Unit::Hour * black_box(-7.0), Unit::Hour * black_box(-7));
        })
    });

    c.bench_function("Duration add and assert minute second", |b| {
        b.iter(|| {
            assert_eq!(Unit::Minute * black_box(-2.0), Unit::Minute * black_box(-2));
            assert_eq!(Unit::Second * black_box(3.0), Unit::Second * black_box(3));
        })
    });

    c.bench_function("Duration add and assert subseconds", |b| {
        b.iter(|| {
            assert_eq!(
                Unit::Millisecond * black_box(4.0),
                Unit::Millisecond * black_box(4)
            );
            assert_eq!(
                Unit::Nanosecond * black_box(5.0),
                Unit::Nanosecond * black_box(5)
            );
        })
    });

    #[cfg(feature = "std")]
    {
        c.bench_function("RFC3339 with seconds", |b| {
            b.iter(|| Epoch::from_gregorian_str("2018-02-13T23:08:32Z").unwrap());
        });

        c.bench_function("RFC3339 with milliseconds", |b| {
            b.iter(|| Epoch::from_gregorian_str("2018-02-13T23:08:32.123Z").unwrap());
        });

        c.bench_function("RFC3339 with nanoseconds", |b| {
            b.iter(|| Epoch::from_gregorian_str("2018-02-13T23:08:32.123456983Z").unwrap());
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
