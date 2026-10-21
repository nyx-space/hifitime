//! Tests that run inside a WebAssembly host: `wasm-pack test --node`.

#![cfg(target_arch = "wasm32")]

use hifitime_wasm::{
    month_from_number, parse_time_scale, version, Duration, Epoch, MonthName, Polynomial,
    TimeScale, TimeSeries, Unit, Weekday,
};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn version_is_the_crate_version() {
    assert_eq!(version(), env!("HIFITIME_VERSION"));
}

#[wasm_bindgen_test]
fn parse_and_format_an_epoch() {
    let epoch = Epoch::new("2024-02-29T12:34:56.789 UTC").unwrap();
    assert_eq!(epoch.time_scale(), TimeScale::UTC);
    assert_eq!(epoch.year(), 2024);
    assert_eq!(epoch.month_name(), MonthName::February);
    assert_eq!(epoch.day_of_month(), 29);
    assert_eq!(epoch.milliseconds(), 789);
    assert_eq!(epoch.to_string(), "2024-02-29T12:34:56.789000000 UTC");
    assert_eq!(epoch.to_isoformat(), "2024-02-29T12:34:56.789000");
}

#[wasm_bindgen_test]
fn gpst_is_utc_plus_the_leap_seconds() {
    let epoch = Epoch::from_gregorian_utc(2024, 2, 29, 12, 34, 56, 789_000_000).unwrap();
    // GPST = TAI - 19 s, and TAI - UTC = 37 s since 2017, so GPST = UTC + 18 s.
    assert_eq!(
        epoch.to_gregorian_string(TimeScale::GPST),
        "2024-02-29T12:35:14.789000000 GPST"
    );
    assert_eq!(epoch.leap_seconds_iers(), 37);
}

#[wasm_bindgen_test]
fn arithmetic_is_exact() {
    let start = Epoch::new("2024-01-01T00:00:00 UTC").unwrap();
    let step = Duration::from_hours(1.5);
    let later = start.add(&step);
    assert!(later.duration_since(&start).equals(&step));
    assert_eq!(later.duration_since(&start).to_string(), "1 h 30 min");
    assert_eq!(start.compare(&later), -1);
    assert!(later.sub(&step).equals(&start));
}

#[wasm_bindgen_test]
fn duration_keeps_every_nanosecond() {
    let duration = Duration::from_parts(3, 12);
    assert_eq!(duration.total_nanoseconds(), 9_467_280_000_000_000_012);
    let round_trip = Duration::from_total_nanoseconds(duration.total_nanoseconds());
    assert!(round_trip.equals(&duration));
}

#[wasm_bindgen_test]
fn duration_parses_and_converts() {
    let duration = Duration::new("3 h 15 min").unwrap();
    assert_eq!(duration.to_unit(Unit::Minute), 195.0);
    assert_eq!(Duration::from_unit(2.0, Unit::Day).to_hours(), 48.0);
}

#[wasm_bindgen_test]
fn unix_nanoseconds_round_trip() {
    let epoch = Epoch::new("2024-02-29T12:34:56.123456789 UTC").unwrap();
    let nanoseconds = epoch.to_unix_nanoseconds();
    assert_eq!(nanoseconds, 1_709_210_096_123_456_789);
    assert!(Epoch::from_unix_nanoseconds(nanoseconds).equals(&epoch));
}

#[wasm_bindgen_test]
fn time_series_counts_inclusive_and_exclusive() {
    let start = Epoch::new("2024-01-01T00:00:00 UTC").unwrap();
    let end = Epoch::new("2024-01-01T02:00:00 UTC").unwrap();
    let step = Duration::from_minutes(30.0);
    assert_eq!(TimeSeries::inclusive(&start, &end, &step).count(), 5);
    assert_eq!(TimeSeries::exclusive(&start, &end, &step).count(), 4);

    let mut series = TimeSeries::inclusive(&start, &end, &step);
    assert!(series.next().unwrap().equals(&start));
    assert_eq!(series.count(), 4);
    assert_eq!(series.to_array().len(), 4);
}

#[wasm_bindgen_test]
fn weekday_of_a_known_date() {
    let epoch = Epoch::new("2024-02-29T12:00:00 UTC").unwrap();
    assert_eq!(epoch.weekday_utc(), Weekday::Thursday);
}

#[wasm_bindgen_test]
fn strftime_and_strptime_agree() {
    let epoch = Epoch::strptime("2024-02-29 12:34:56", "%Y-%m-%d %H:%M:%S").unwrap();
    assert_eq!(
        epoch.strftime("%Y-%m-%d %H:%M:%S").unwrap(),
        "2024-02-29 12:34:56"
    );
}

#[wasm_bindgen_test]
fn enumerations_parse() {
    assert_eq!(parse_time_scale("GPST").unwrap(), TimeScale::GPST);
    assert_eq!(month_from_number(12).unwrap(), MonthName::December);
}

#[wasm_bindgen_test]
fn polynomial_holds_its_terms() {
    let polynomial = Polynomial::new(1.0, 0.0, 0.0);
    assert_eq!(polynomial.constant().to_seconds(), 1.0);
    assert_eq!(polynomial.rate().to_seconds(), 0.0);
}
