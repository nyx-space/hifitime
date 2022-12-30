use hifitime::{
    Duration, Errors, Freq, Frequencies, ParsingErrors, TimeUnits, Unit, NANOSECONDS_PER_CENTURY,
    NANOSECONDS_PER_MINUTE,
};

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
    assert!((sum.to_seconds() - 25565.0).abs() < EPSILON);

    let neg_sum = -sum;
    assert!((neg_sum.to_seconds() + 25565.0).abs() < EPSILON);

    assert_eq!(neg_sum.abs(), sum, "abs failed");

    let sub: Duration = seven_hours - six_minutes - five_seconds;
    assert!((sub.to_seconds() - 24835.0).abs() < EPSILON);

    // Test fractional
    let quarter_hour = 0.25 * Unit::Hour;
    let third_hour = (1.0 / 3.0) * Unit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    assert!((sum.to_unit(Unit::Minute) - 35.0).abs() < EPSILON);

    let quarter_hour = -0.25 * Unit::Hour;
    let third_hour: Duration = -1 * Unit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.to_unit(Unit::Millisecond).floor() - sum.to_unit(Unit::Second).floor() * 1000.0;
    assert!(delta < EPSILON);
    assert!((sum.to_unit(Unit::Minute) + 35.0).abs() < EPSILON);
}

#[test]
fn duration_format() {
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

    assert_eq!(
        arbitrary,
        Duration::compose(0, 14889, 23, 47, 34, 0, 0, 123)
    );

    // Test fractional
    let quarter_hour = 0.25 * Unit::Hour;
    let third_hour = (1.0 / 3.0) * Unit::Hour;
    let sum: Duration = quarter_hour + third_hour;

    println!(
        "Proof that Duration is more precise than f64: {} vs {}",
        sum.to_unit(Unit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum), "35 min");

    let quarter_hour = -0.25 * Unit::Hour;
    let third_hour: Duration = -1 * Unit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.to_unit(Unit::Millisecond).floor() - sum.to_unit(Unit::Second).floor() * 1000.0;
    assert_eq!(delta * -1.0, 0.0);
    assert_eq!(format!("{}", sum), "-35 min");

    assert_eq!(format!("{}", Duration::MAX), "1196851200 days");
    assert_eq!(format!("{}", Duration::MIN), "-1196851200 days");
    assert_eq!(format!("{}", Duration::ZERO), "0 ns");

    // The `e` format will print this as a floating point value.
    let mut sum2 = sum;
    sum2 -= 1 * Unit::Nanosecond;
    assert_eq!(sum2, sum - 1 * Unit::Nanosecond);
    assert_eq!(sum2, sum - Unit::Nanosecond);
    assert_eq!(format!("{:e}", sum2), "-35.00000000001667 min");
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

    let half_hour = 0.5.hours();
    let quarter_hour = 0.5 * half_hour;
    assert_eq!(quarter_hour, 15.minutes());
    #[cfg(feature = "std")]
    println!("{}", quarter_hour);

    let min_quarter_hour = -0.5 * half_hour;
    assert_eq!(min_quarter_hour, -15.minutes());
    #[cfg(feature = "std")]
    println!("{}", min_quarter_hour);
}

#[test]
fn test_ops_near_bounds() {
    assert_eq!(Duration::MAX - Duration::MAX, 0 * Unit::Nanosecond);
    assert_eq!(Duration::MIN - Duration::MIN, 0 * Unit::Nanosecond);

    // Check that the special cases of the bounds themselves don't prevent correct math.
    assert_eq!(
        (Duration::MIN - 1 * Unit::Nanosecond) - (Duration::MIN - 1 * Unit::Nanosecond),
        0 * Unit::Nanosecond
    );

    let tt_offset_ns: u64 = 32_184_000_000;
    let duration = Duration::from_parts(-32767, 0);
    let exp = Duration::from_parts(-32768, NANOSECONDS_PER_CENTURY - tt_offset_ns);
    assert_eq!(
        duration - Duration::from_total_nanoseconds(tt_offset_ns.into()),
        exp
    );

    // Test the zero crossing with a large negative value
    assert_eq!(
        2 * Unit::Nanosecond - (-1 * Unit::Century),
        1 * Unit::Century + 2 * Unit::Nanosecond
    );

    // Check that we saturate one way but not the other for MIN
    assert_eq!(Duration::MIN - 1 * Unit::Nanosecond, Duration::MIN);
    assert_ne!(Duration::MIN + 1 * Unit::Nanosecond, Duration::MIN);

    // Check that we saturate one way but not the other for MAX
    assert_eq!(Duration::MAX + 1 * Unit::Nanosecond, Duration::MAX);
    assert_ne!(Duration::MAX - 1 * Unit::Nanosecond, Duration::MAX);
}

#[test]
fn test_neg() {
    assert_eq!(Duration::MIN_NEGATIVE, -Duration::MIN_POSITIVE);
    assert_eq!(Duration::MIN_POSITIVE, -Duration::MIN_NEGATIVE);
    assert_eq!(2.nanoseconds(), -(2.0.nanoseconds()));
    assert_eq!(Duration::MIN, -Duration::MAX);
    assert_eq!(Duration::MAX, -Duration::MIN);
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

    let past_min = Duration::from_total_nanoseconds(i128::MIN);
    assert_eq!(past_min, Duration::MIN);
}

#[test]
fn duration_enum_eq() {
    // Check the equality compiles (if one compiles, then all asserts will work)
    assert!(Freq::GigaHertz == Freq::GigaHertz);
    assert!(Unit::Century == Unit::Century);
    assert!(1 * Unit::Century == Unit::Century);
    assert!(1 * Unit::Century >= Unit::Century);
    assert!(1 * Unit::Century <= Unit::Century);
    assert!(1 * Unit::Century > Unit::Day);
}

#[test]
fn duration_enum_orq() {
    // Check the equality compiles (if one compiles, then all asserts will work)
    assert!(Unit::Century > Unit::Day);
    assert_eq!(Unit::Century.min(Unit::Day), Unit::Day);
    assert_eq!(Unit::Century.max(Unit::Day), Unit::Century);
    // Frequencies are converted to durations, and that's what compared!
    assert!(Freq::GigaHertz < Freq::MegaHertz);
}

#[test]
fn freq_mul() {
    assert_eq!(1_000 * Freq::MegaHertz, Unit::Nanosecond);
    assert_eq!(1_000 * Freq::KiloHertz, Unit::Microsecond);
    assert_eq!(1_000 * Freq::Hertz, Unit::Millisecond);

    assert_eq!(Freq::MegaHertz * 1_000, Unit::Nanosecond);
    assert_eq!(Freq::KiloHertz * 1_000, Unit::Microsecond);
    assert_eq!(Freq::Hertz * 1_000, Unit::Millisecond);

    assert_eq!(1_000.MHz(), Unit::Nanosecond);
    assert_eq!(1_000.kHz(), Unit::Microsecond);
    assert_eq!(1_000.Hz(), Unit::Millisecond);
}

#[test]
fn duration_recip() {
    // Ensure that for arbitrary durations and constant ones, the from and to total nanoseconds is reciprocal.
    let arbitrary = 14889.days()
        + 23.hours()
        + 47.minutes()
        + 34.seconds()
        + 0.milliseconds()
        + 123.nanoseconds();

    for duration in [
        arbitrary,
        Duration::ZERO,
        Duration::MAX,
        Duration::MIN,
        Duration::MIN_NEGATIVE,
        Duration::MIN_POSITIVE,
    ] {
        assert_eq!(
            duration,
            Duration::from_total_nanoseconds(duration.total_nanoseconds()),
        );
    }
}

#[test]
fn duration_floor_ceil_round() {
    // These are from here: https://www.geeksforgeeks.org/time-round-function-in-golang-with-examples/
    let d = 5.minutes() + 7.seconds();
    assert_eq!(d.floor(6.seconds()), 5.minutes() + 6.seconds());
    assert_eq!(d.floor(-6.seconds()), 5.minutes() + 6.seconds());
    assert_eq!(d.ceil(6.seconds()), 5.minutes() + 12.seconds());
    println!("{}", d.ceil(-6.seconds()));
    println!("{}", 5.minutes() + 12.seconds());
    assert_eq!(d.ceil(-6.seconds()), 5.minutes() + 12.seconds());

    let d = 3.minutes() + 73.671.seconds();
    assert_eq!(d, 4.minutes() + 13.seconds() + 671.milliseconds());

    // Floor
    // Rounding to the closest microsecond should return the same duration
    assert_eq!(d.floor(1.microseconds()), d);
    assert_eq!(d.floor(1.seconds()), 4.minutes() + 13.seconds());
    assert_eq!(d.floor(3.seconds()), 4.minutes() + 12.seconds());
    assert_eq!(d.floor(9.minutes()), 0.minutes());
    assert_eq!(
        (Duration::MIN + 10.seconds()).floor(10.seconds()),
        Duration::MIN
    );

    // Ceil
    assert_eq!(d.ceil(1.minutes()), 5.minutes());
    assert_eq!(d.ceil(30.seconds()), 4.minutes() + 30.seconds());
    assert_eq!(d.ceil(4.minutes()), 8.minutes());
    assert_eq!(d.ceil(1.seconds()), 4.minutes() + 14.seconds());
    assert_eq!(Duration::MAX.ceil(1.seconds()), Duration::MAX);

    // Round
    assert_eq!(d.round(1.minutes()), 4.minutes());
    assert_eq!(d.round(30.seconds()), 4.minutes());
    assert_eq!(d.round(4.minutes()), 4.minutes());
    assert_eq!(d.round(1.seconds()), 4.minutes() + 14.seconds());
}

#[test]
fn duration_from_str() {
    use core::str::FromStr;
    use hifitime::{Duration, Unit};

    assert_eq!(Duration::from_str("1 d").unwrap(), Unit::Day * 1);
    assert_eq!(
        Duration::from_str("10.598 days").unwrap(),
        Unit::Day * 10.598
    );
    assert_eq!(
        Duration::from_str("10.598 min").unwrap(),
        Unit::Minute * 10.598
    );
    assert_eq!(
        Duration::from_str("10.598 us").unwrap(),
        Unit::Microsecond * 10.598
    );
    assert_eq!(
        Duration::from_str("10.598 seconds").unwrap(),
        Unit::Second * 10.598
    );
    assert_eq!(
        Duration::from_str("10.598 nanosecond").unwrap(),
        Unit::Nanosecond * 10.598
    );

    assert_eq!(
        Duration::from_str("1 d 15.5 hours 25 ns").unwrap(),
        Unit::Day * 1 + 15.5 * Unit::Hour + 25 * Unit::Nanosecond
    );

    assert_eq!(
        Duration::from_str("5 h 256 ms 1 ns").unwrap(),
        5 * Unit::Hour + 256 * Unit::Millisecond + Unit::Nanosecond
    );

    // It supports extra white spaces before and after the duration
    assert_eq!(
        Duration::from_str("  5 days 1 ns ").unwrap(),
        5 * Unit::Day + 1 * Unit::Nanosecond
    );

    assert!(
        Duration::from_str("5 days 1")
            == Err(Errors::ParseError(ParsingErrors::UnknownOrMissingUnit)),
        "should return an unknown unit error"
    );

    assert!(
        Duration::from_str("5 days 1 ")
            == Err(Errors::ParseError(ParsingErrors::UnknownOrMissingUnit)),
        "should return an unknown unit error"
    );

    // Test the offset initialization
    assert_eq!(
        Duration::from_str("-01:15:30").unwrap(),
        -(1 * Unit::Hour + 15 * Unit::Minute + 30 * Unit::Second)
    );

    assert_eq!(
        Duration::from_str("+01:15:30").unwrap(),
        1 * Unit::Hour + 15 * Unit::Minute + 30 * Unit::Second
    );

    assert_eq!(
        Duration::from_str("-01:15").unwrap(),
        -(1 * Unit::Hour + 15 * Unit::Minute)
    );

    assert_eq!(
        Duration::from_str("+01:15").unwrap(),
        1 * Unit::Hour + 15 * Unit::Minute
    );

    // Test offsets without colon
    assert_eq!(
        Duration::from_str("-011530").unwrap(),
        -(1 * Unit::Hour + 15 * Unit::Minute + 30 * Unit::Second)
    );

    assert_eq!(
        Duration::from_str("+011530").unwrap(),
        1 * Unit::Hour + 15 * Unit::Minute + 30 * Unit::Second
    );

    assert_eq!(
        Duration::from_str("-0115").unwrap(),
        -(1 * Unit::Hour + 15 * Unit::Minute)
    );

    assert_eq!(
        Duration::from_str("+0115").unwrap(),
        1 * Unit::Hour + 15 * Unit::Minute
    );

    assert_eq!(
        Duration::from_str("+2515").unwrap(),
        25 * Unit::Hour + 15 * Unit::Minute
    );

    assert_eq!(
        Duration::from_tz_offset(1, 1, 15),
        1 * Unit::Hour + 15 * Unit::Minute
    );

    assert_eq!(
        Duration::from_tz_offset(-1, 1, 15),
        -(1 * Unit::Hour + 15 * Unit::Minute)
    );

    assert_eq!(
        Duration::from_str(""),
        Err(Errors::ParseError(ParsingErrors::ValueError))
    );

    assert_eq!(
        Duration::from_str("+"),
        Err(Errors::ParseError(ParsingErrors::ValueError))
    );
}

#[cfg(feature = "std")]
#[test]
fn std_time_duration() {
    use std::time::Duration as StdDuration;

    let hf_duration = 5 * Unit::Day + 1 * Unit::Nanosecond;
    let std_duration: StdDuration = hf_duration.into();
    assert_eq!(std_duration, StdDuration::new(432_000, 1));

    let hf_return: Duration = std_duration.into();
    assert_eq!(hf_return, hf_duration);

    // Check that a negative hifitime duration is zero in std time
    let std_duration: StdDuration = (-hf_duration).into();
    assert_eq!(std_duration, StdDuration::ZERO);
}

#[test]
fn test_decompose() {
    let pos = 5 * Unit::Hour + 256 * Unit::Millisecond + Unit::Nanosecond;

    let (sign, days, hours, minutes, seconds, milliseconds, microseconds, nanos) = pos.decompose();
    assert_eq!(sign, 0);
    assert_eq!(days, 0);
    assert_eq!(hours, 5);
    assert_eq!(minutes, 0);
    assert_eq!(seconds, 0);
    assert_eq!(milliseconds, 256);
    assert_eq!(microseconds, 0);
    assert_eq!(nanos, 1);

    // A negative duration works in the same way, only the sign is different.
    let neg = -(5 * Unit::Hour + 256 * Unit::Millisecond + Unit::Nanosecond);
    assert_eq!(neg, -pos);
    assert_eq!(neg.abs(), pos);
    assert!(neg.is_negative());

    let (sign, days, hours, minutes, seconds, milliseconds, microseconds, nanos) = neg.decompose();
    assert_eq!(sign, -1);
    assert_eq!(days, 0);
    assert_eq!(hours, 5);
    assert_eq!(minutes, 0);
    assert_eq!(seconds, 0);
    assert_eq!(milliseconds, 256);
    assert_eq!(microseconds, 0);
    assert_eq!(nanos, 1);
}

#[test]
fn test_min_max() {
    use hifitime::TimeUnits;

    let d0 = 20.seconds();
    let d1 = 21.seconds();

    assert_eq!(d0, d1.min(d0));
    assert_eq!(d0, d0.min(d1));

    assert_eq!(d1, d1.max(d0));
    assert_eq!(d1, d0.max(d1));
}
