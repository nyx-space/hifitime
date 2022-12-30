use core::str::FromStr;
use hifitime::{Duration, Unit};
use iai::black_box;

fn duration_from_f64_seconds() {
    let value: f64 = 6311433599.999999;
    black_box(Duration::from_seconds(value));
}

fn duration_from_f64_seconds_via_units() {
    let value: f64 = 6311433599.999999;
    black_box(value * Unit::Second);
}

fn duration_from_i64_nanoseconds() {
    let value: i64 = 6311433599;
    black_box(Duration::from_truncated_nanoseconds(value));
}

fn duration_from_i64_nanoseconds_via_units() {
    let value: i64 = 6311433599;
    black_box(value * Unit::Nanosecond);
}

fn duration_parse_days_only() {
    black_box(Duration::from_str("15 d").unwrap());
}

fn duration_parse_complex() {
    black_box(Duration::from_str("1 d 15.5 hours 25 ns").unwrap());
}

fn large_duration_to_seconds() {
    let d: Duration = 50.15978 * Unit::Century;
    black_box(d.to_seconds());
}

fn duration_to_seconds() {
    let d: Duration = 50.15978 * Unit::Second;
    black_box(d.to_seconds());
}

fn small_duration_to_seconds() {
    let d: Duration = 50.159 * Unit::Microsecond;
    black_box(d.to_seconds());
}

iai::main!(
    duration_from_f64_seconds,
    duration_from_i64_nanoseconds,
    duration_from_f64_seconds_via_units,
    duration_from_i64_nanoseconds_via_units,
    duration_parse_days_only,
    duration_parse_complex,
    large_duration_to_seconds,
    duration_to_seconds,
    small_duration_to_seconds,
);
