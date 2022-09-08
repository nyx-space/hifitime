extern crate hifitime;
use hifitime::{Duration, Freq, TimeUnits, Unit, NANOSECONDS_PER_MINUTE};

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "std")]
use core::f64::EPSILON;
#[cfg(not(feature = "std"))]
use std::f64::EPSILON;

#[test]
fn time_unit() {
    // Check that the same number is created for different types
    assert_eq!(Unit::Day * 10.0, Unit::Day * 10);
    assert_eq!(Unit::Hour * -7.0, Unit::Hour * -7);
    assert_eq!(Unit::Minute * -2.0, Unit::Minute * -2);
    assert_eq!(Unit::Second * 3.0, Unit::Second * 3);
    assert_eq!(Unit::Millisecond * 4.0, Unit::Millisecond * 4);
    assert_eq!(Unit::Nanosecond * 5.0, Unit::Nanosecond * 5);

    // Check the LHS multiplications match the RHS ones
    assert_eq!(10.0 * Unit::Day, Unit::Day * 10);
    assert_eq!(-7 * Unit::Hour, Unit::Hour * -7.0);
    assert_eq!(-2.0 * Unit::Minute, Unit::Minute * -2);
    assert_eq!(3.0 * Unit::Second, Unit::Second * 3);
    assert_eq!(4.0 * Unit::Millisecond, Unit::Millisecond * 4);
    assert_eq!(5.0 * Unit::Nanosecond, Unit::Nanosecond * 5);

    let d: Duration = 1.0 * Unit::Hour / 3 - 20 * Unit::Minute;
    assert!(d.abs() < Unit::Nanosecond);
    assert_eq!(3 * (20 * Unit::Minute), Unit::Hour);

    // Test operations
    let seven_hours = Unit::Hour * 7;
    let six_minutes = Unit::Minute * 6;
    // let five_seconds = Unit::Second * 5.0;
    let five_seconds = 5.0.seconds();
    let sum: Duration = seven_hours + six_minutes + five_seconds;
    assert!((sum.in_seconds() - 25565.0).abs() < EPSILON);

    let neg_sum = -sum;
    assert!((neg_sum.in_seconds() + 25565.0).abs() < EPSILON);

    assert_eq!(neg_sum.abs(), sum, "abs failed");

    let sub: Duration = seven_hours - six_minutes - five_seconds;
    assert!((sub.in_seconds() - 24835.0).abs() < EPSILON);

    // Test fractional
    let quarter_hour = 0.25 * Unit::Hour;
    let third_hour = (1.0 / 3.0) * Unit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    assert!((sum.in_unit(Unit::Minute) - 35.0).abs() < EPSILON);

    let quarter_hour = -0.25 * Unit::Hour;
    let third_hour: Duration = -1 * Unit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.in_unit(Unit::Millisecond).floor() - sum.in_unit(Unit::Second).floor() * 1000.0;
    assert!(delta < EPSILON);
    assert!((sum.in_unit(Unit::Minute) + 35.0).abs() < EPSILON);
}

#[test]
fn duration_print() {
    // Check printing adds precision
    assert_eq!(
        format!("{}", Unit::Day * 10.0 + Unit::Hour * 5),
        "10 days 5 h"
    );

    assert_eq!(
        format!("{}", Unit::Hour * 5 + Unit::Millisecond * 256),
        "5 h 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            Unit::Hour * 5 + Unit::Millisecond * 256 + Unit::Nanosecond
        ),
        "5 h 256 ms 1 ns"
    );

    assert_eq!(format!("{}", Unit::Hour + Unit::Second), "1 h 1 s");

    // NOTE: We _try_ to specify down to a half of a nanosecond, but duration is NOT
    // more precise than that, so it only actually stores to that number.
    assert_eq!(
        format!(
            "{}",
            Unit::Hour * 5 + Unit::Millisecond * 256 + Unit::Microsecond + Unit::Nanosecond * 3.5
        ),
        "5 h 256 ms 1 μs 3 ns"
    );

    // Check printing negative durations only shows one negative sign
    assert_eq!(
        format!("{}", Unit::Hour * -5 + Unit::Millisecond * -256),
        "-5 h 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            Unit::Hour * -5 + Unit::Millisecond * -256 + Unit::Nanosecond * -3
        ),
        "-5 h 256 ms 3 ns"
    );

    assert_eq!(
        format!(
            "{}",
            (Unit::Hour * -5 + Unit::Millisecond * -256)
                - (Unit::Hour * -5 + Unit::Millisecond * -256 + Unit::Nanosecond * 2)
        ),
        "-2 ns"
    );

    assert_eq!(format!("{}", Unit::Nanosecond * 2), "2 ns");

    // Check that we support nanoseconds pas GPS time
    let now = Unit::Nanosecond * 1286495254000000123;
    assert_eq!(format!("{}", now), "14889 days 23 h 47 min 34 s 123 ns");

    let arbitrary = 14889.days()
        + 23.hours()
        + 47.minutes()
        + 34.seconds()
        + 0.milliseconds()
        + 123.nanoseconds();
    assert_eq!(
        format!("{}", arbitrary),
        "14889 days 23 h 47 min 34 s 123 ns"
    );

    // Test fractional
    let quarter_hour = 0.25 * Unit::Hour;
    let third_hour = (1.0 / 3.0) * Unit::Hour;
    let sum: Duration = quarter_hour + third_hour;

    println!(
        "Proof that Duration is more precise than f64: {} vs {}",
        sum.in_unit(Unit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum), "35 min");

    let quarter_hour = -0.25 * Unit::Hour;
    let third_hour: Duration = -1 * Unit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.in_unit(Unit::Millisecond).floor() - sum.in_unit(Unit::Second).floor() * 1000.0;
    assert_eq!(delta * -1.0, 0.0);
    assert_eq!(format!("{}", sum), "-35 min");
}

#[test]
fn test_ops() {
    assert_eq!(
        (0.25 * Unit::Hour).total_nanoseconds(),
        (15 * NANOSECONDS_PER_MINUTE).into()
    );

    assert_eq!(
        (-0.25 * Unit::Hour).total_nanoseconds(),
        i128::from(15 * NANOSECONDS_PER_MINUTE) * -1
    );

    assert_eq!(
        (-0.25 * Unit::Hour - 0.25 * Unit::Hour).total_nanoseconds(),
        i128::from(30 * NANOSECONDS_PER_MINUTE) * -1
    );

    #[cfg(feature = "std")]
    println!("{}", -0.25 * Unit::Hour + (-0.25 * Unit::Hour));

    assert_eq!(
        Duration::MIN_POSITIVE + 4 * Duration::MIN_POSITIVE,
        5 * Unit::Nanosecond
    );

    assert_eq!(
        Duration::MIN_NEGATIVE + 4 * Duration::MIN_NEGATIVE,
        -5 * Unit::Nanosecond
    );

    let halfhour = 0.5.hours();
    let quarterhour = 0.5 * halfhour;
    assert_eq!(quarterhour, 15.minutes());
    #[cfg(feature = "std")]
    println!("{}", quarterhour);

    let min_quarterhour = -0.5 * halfhour;
    assert_eq!(min_quarterhour, -15.minutes());
    #[cfg(feature = "std")]
    println!("{}", min_quarterhour);
}

#[test]
fn test_neg() {
    assert_eq!(Duration::MIN_NEGATIVE, -Duration::MIN_POSITIVE);
    assert_eq!(Duration::MIN_POSITIVE, -Duration::MIN_NEGATIVE);
    assert_eq!(2.nanoseconds(), -(2.0.nanoseconds()));
}

#[test]
fn test_extremes() {
    let d = Duration::from_total_nanoseconds(i128::MAX);

    assert_eq!(Duration::from_total_nanoseconds(d.total_nanoseconds()), d);
    let d = Duration::from_total_nanoseconds(i128::MIN + 1);
    assert_eq!(d, Duration::MIN);
    // Test min positive
    let d_min = Duration::from_total_nanoseconds(Duration::MIN_POSITIVE.total_nanoseconds());
    assert_eq!(d_min, Duration::MIN_POSITIVE);
    // Test difference between min durations
    assert_eq!(
        Duration::MIN_POSITIVE - Duration::MIN_NEGATIVE,
        2 * Unit::Nanosecond
    );
    assert_eq!(
        Duration::MIN_NEGATIVE - Duration::MIN_POSITIVE,
        -2 * Unit::Nanosecond
    );
    assert_eq!(Duration::from_total_nanoseconds(2), 2 * Unit::Nanosecond);
    // Check that we do not support more precise than nanosecond
    assert_eq!(Unit::Nanosecond * 3.5, Unit::Nanosecond * 3);

    assert_eq!(
        Duration::MIN_POSITIVE + Duration::MIN_NEGATIVE,
        Duration::ZERO
    );

    assert_eq!(
        Duration::MIN_NEGATIVE + Duration::MIN_NEGATIVE,
        -2 * Unit::Nanosecond
    );

    // Add i64 tests
    let d = Duration::from_truncated_nanoseconds(i64::MAX);
    #[cfg(feature = "std")]
    println!("{}", d);
    assert_eq!(
        Duration::from_truncated_nanoseconds(d.truncated_nanoseconds()),
        d
    );
}

#[test]
fn duration_enum_eq() {
    // Check the equality compiles (if one compiles, then all asserts will work)
    assert!(Freq::GigaHertz == Freq::GigaHertz);
    assert!(Unit::Century == Unit::Century);
}

#[test]
fn duration_enum_orq() {
    // Check the equality compiles (if one compiles, then all asserts will work)
    assert!(Unit::Century > Unit::Day);
    // Frequencies are converted to durations, and that's what compared!
    assert!(Freq::GigaHertz < Freq::MegaHertz);
}

#[test]
fn duration_floor_ceil_round() {
    // These are from here: https://www.geeksforgeeks.org/time-round-function-in-golang-with-examples/
    let d = 5.minutes() + 7.seconds();
    assert_eq!(d.floor(6.seconds()), 5.minutes() + 6.seconds());
    assert_eq!(d.floor(-6.seconds()), 5.minutes() + 6.seconds());
    assert_eq!(d.ceil(6.seconds()), 5.minutes() + 12.seconds());
    assert_eq!(d.ceil(-6.seconds()), 5.minutes() + 12.seconds());

    let d = 3.minutes() + 73.671.seconds();
    assert_eq!(d, 4.minutes() + 13.seconds() + 671.milliseconds());
    // Rounding to the closest microsecond should return the same duration
    assert_eq!(d.floor(1.microseconds()), d);
    assert_eq!(d.floor(1.seconds()), 4.minutes() + 13.seconds());
    assert_eq!(d.floor(3.seconds()), 4.minutes() + 12.seconds());
    assert_eq!(d.floor(9.minutes()), 0.minutes());

    // Ceil
    assert_eq!(d.ceil(1.minutes()), 5.minutes());
    assert_eq!(d.ceil(30.seconds()), 4.minutes() + 30.seconds());
    assert_eq!(d.ceil(4.minutes()), 8.minutes());
    assert_eq!(d.ceil(1.seconds()), 4.minutes() + 14.seconds());

    // Round
    assert_eq!(d.round(1.minutes()), 4.minutes());
    assert_eq!(d.round(30.seconds()), 4.minutes());
    assert_eq!(d.round(4.minutes()), 4.minutes());
    assert_eq!(d.round(1.seconds()), 4.minutes() + 14.seconds());
}
