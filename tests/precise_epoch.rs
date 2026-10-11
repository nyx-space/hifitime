use hifitime::prelude::*;

#[cfg(feature = "std")]
use core::str::FromStr;

/// Time scales for which the conversion to and from TAI only involves integer offsets (and leap seconds).
const EXACT_TS: [TimeScale; 8] = [
    TimeScale::TAI,
    TimeScale::TT,
    TimeScale::UTC,
    TimeScale::GPST,
    TimeScale::GST,
    TimeScale::BDT,
    TimeScale::QZSST,
    TimeScale::ET,
];

/// Time scales whose conversion involves a relativistic rate (applied with 128 bit integer arithmetic).
const RATE_TS: [TimeScale; 3] = [TimeScale::TCG, TimeScale::TCL, TimeScale::TL];

/// Time scales whose inverse conversion relies on the fixed-point iteration of the TDB model.
const TDB_TS: [TimeScale; 2] = [TimeScale::TDB, TimeScale::TCB];

fn zs(value: i128) -> PreciseDuration {
    PreciseDuration::from_total_zeptoseconds(value)
}

fn test_epochs() -> Vec<PreciseEpoch> {
    [
        Epoch::from_gregorian_utc(1972, 1, 1, 0, 0, 0, 0),
        Epoch::from_gregorian_utc(1999, 12, 31, 23, 59, 59, 999_999_999),
        Epoch::from_gregorian_utc(2026, 10, 11, 12, 34, 56, 123_456_789),
        Epoch::from_gregorian_tai(2050, 6, 15, 6, 0, 0, 1),
    ]
    .into_iter()
    .map(|e| PreciseEpoch::from(e) + zs(987_654_321_098))
    .collect()
}

#[test]
fn exact_roundtrips() {
    for p in test_epochs() {
        for ts in EXACT_TS {
            let rt = p.to_time_scale(ts).to_time_scale(TimeScale::TAI);
            assert_eq!(
                (rt - p).total_zeptoseconds(),
                0,
                "{ts} round trip of {p} is not exact"
            );
        }
    }
}

#[test]
fn relativistic_rate_roundtrips() {
    for p in test_epochs() {
        for ts in RATE_TS {
            let err = (p.to_time_scale(ts).to_time_scale(TimeScale::TAI) - p).total_zeptoseconds();
            // Below one femtosecond (1e6 zs): only limited by the binary representation of the rates.
            assert!(err.abs() < 1_000_000, "{ts} round trip error of {err} zs");
        }
        for ts in TDB_TS {
            let err = (p.to_time_scale(ts).to_time_scale(TimeScale::TAI) - p).total_zeptoseconds();
            // Limited by the TDB model inversion, cf. the inner_g iteration in `to_time_scale`.
            assert!(
                err.abs() < 10_000_000_000,
                "{ts} round trip error of {err} zs"
            );
        }
    }
}

#[test]
fn agrees_with_epoch() {
    for p in test_epochs() {
        let e = p.to_epoch();
        for ts in EXACT_TS.iter().chain(RATE_TS.iter()).chain(TDB_TS.iter()) {
            let from_precise = p.to_time_scale(*ts).to_epoch();
            let from_epoch = e.to_time_scale(*ts);
            let delta = (from_precise - from_epoch).abs();
            assert!(
                delta <= 2.nanoseconds(),
                "{ts}: precise {from_precise} vs nanosecond {from_epoch}"
            );
        }
    }
}

#[test]
fn epoch_conversions() {
    let e = Epoch::from_gregorian_utc(2026, 10, 11, 12, 34, 56, 123_456_789);
    let p = PreciseEpoch::from(e);
    assert_eq!(p.time_scale, TimeScale::UTC);
    assert_eq!(p.subnanoseconds(), 0);
    assert_eq!(Epoch::from(p), e);

    let p = p + zs(999_999_999_999);
    assert_eq!(p.subnanoseconds(), 999_999_999_999);
    // Floor vs. rounding to the nanosecond
    assert_eq!(p.to_epoch(), e);
    assert_eq!(p.to_epoch_rounded(), e + 1.nanoseconds());
    // Generic precision change
    let back: PreciseEpoch = p.to_epoch().to_precision();
    assert_eq!(back, PreciseEpoch::from(e));
    assert_eq!(p.to_precision::<PreciseDuration>(), p);
}

#[test]
fn subnanosecond_arithmetic() {
    let p = PreciseEpoch::from_gregorian_tai_at_midnight(2026, 10, 11);
    let q = p + zs(1);
    assert_ne!(p, q);
    assert!(q > p);
    assert_eq!(q - p, zs(1));
    // Mixed precision: a Duration is losslessly promoted
    assert_eq!((q + 1.seconds()) - q, PreciseDuration::from(1.seconds()));
    let mut r = q;
    r -= 1.nanoseconds();
    r += zs(1_000_000_000_000);
    assert_eq!(r, q);
    // Floating point seconds are exact with respect to the binary value of the f64
    let half_as = p + 0.5e-18;
    assert_eq!((half_as - p).total_zeptoseconds(), 500);
}

#[test]
fn equality_across_time_scales() {
    for p in test_epochs() {
        for ts in EXACT_TS {
            let other = p.to_time_scale(ts);
            assert_eq!(p, other, "{ts}");
            assert_eq!(p.cmp(&other), core::cmp::Ordering::Equal);
            // One zeptosecond is enough to break equality
            assert_ne!(p, other + zs(1), "{ts}");
            assert!(p < other + zs(1), "{ts}");
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn hash_consistent_with_eq() {
    use std::collections::HashSet;
    let p = test_epochs()[2];
    let mut set = HashSet::new();
    set.insert(p);
    assert!(set.contains(&p.to_time_scale(TimeScale::GPST)));
    assert!(!set.contains(&(p + zs(1))));
}

#[test]
fn initializers() {
    // Gregorian initializers match the nanosecond ones
    assert_eq!(
        PreciseEpoch::from_gregorian_utc(2026, 10, 11, 12, 34, 56, 789),
        PreciseEpoch::from(Epoch::from_gregorian_utc(2026, 10, 11, 12, 34, 56, 789))
    );
    // Floating point initializers are exact with respect to the f64
    let p = PreciseEpoch::from_tai_seconds(0.1);
    assert_eq!(p.duration.total_zeptoseconds(), 100_000_000_000_000_005_551);
    let p = PreciseEpoch::from_gpst_nanoseconds(1);
    assert_eq!(p.to_gpst_duration(), PreciseDuration::from(1.nanoseconds()));
    assert_eq!(
        PreciseEpoch::from_total_zeptoseconds(42, TimeScale::GST)
            .to_zeptoseconds_in_time_scale(TimeScale::GST),
        42
    );
    // MJD / JDE consistency
    let p = PreciseEpoch::from_mjd_tai(60_000.25);
    assert_eq!(p, PreciseEpoch::from(Epoch::from_mjd_tai(60_000.25)));
}

#[cfg(feature = "std")]
#[test]
fn display_and_parsing() {
    let e = Epoch::from_gregorian_utc(2026, 10, 11, 12, 34, 56, 123_456_789);
    let p = PreciseEpoch::from(e);
    // Without sub-nanoseconds, the display is identical to Epoch
    assert_eq!(format!("{p}"), format!("{e}"));
    assert_eq!(format!("{p:?}"), format!("{e:?}"));

    let p = p + zs(987_654_321_000);
    assert_eq!(format!("{p}"), "2026-10-11T12:34:56.123456789987654321 UTC");
    assert_eq!(
        p.to_gregorian_str(TimeScale::UTC),
        "2026-10-11T12:34:56.123456789987654321 UTC"
    );
    assert_eq!(PreciseEpoch::from_str(&format!("{p}")).unwrap(), p);
    assert_eq!(
        PreciseEpoch::from_gregorian_str("2026-10-11T12:34:56.123456789987654321 UTC").unwrap(),
        p
    );
    // Up to 21 digits
    let q = PreciseEpoch::from_str("2026-10-11T12:34:56.000000000000000000001 TAI").unwrap();
    assert_eq!(q.subnanoseconds(), 1);
    assert!(PreciseEpoch::from_str("2026-10-11T12:34:56.0000000000000000000001 TAI").is_err());
    // Time zone offsets still work
    assert_eq!(
        PreciseEpoch::from_str("2026-10-11T14:34:56.123456789987654321+02:00").unwrap(),
        p
    );
    // The Epoch parser is unchanged
    assert!(Epoch::from_str("2026-10-11T12:34:56.123456789987 UTC").is_err());
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip() {
    let p = test_epochs()[2];
    let s = serde_json::to_string(&p).unwrap();
    let back: PreciseEpoch = serde_json::from_str(&s).unwrap();
    assert_eq!(back, p);
}

#[test]
fn rounding_and_with_funcs() {
    let p = PreciseEpoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43) + zs(7);
    assert_eq!(
        p.floor(PreciseDuration::from(1.hours())),
        PreciseEpoch::from_gregorian_tai_hms(2022, 5, 20, 17, 0, 0)
    );
    assert_eq!(
        p.ceil(PreciseDuration::from(1.hours())),
        PreciseEpoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    );
    // with_hms keeps the sub-second part, including the zeptoseconds
    assert_eq!(p.with_hms(1, 2, 3).subnanoseconds(), 7);
    assert_eq!(p.with_hms_strict(1, 2, 3).subnanoseconds(), 0);
    assert_eq!(p.hours(), 17);
    assert_eq!(p.weekday(), Weekday::Friday);
}
