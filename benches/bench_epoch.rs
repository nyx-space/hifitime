extern crate criterion;
extern crate hifitime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hifitime::{Epoch, TimeUnit};

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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
