extern crate regex;

use self::regex::Regex;
use crate::fraction::ToPrimitive;
use crate::{
    Decimal, Errors, Fraction, DAYS_PER_CENTURY, SECONDS_PER_DAY, SECONDS_PER_HOUR,
    SECONDS_PER_MINUTE,
};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use std::str::FromStr;

/// Defines generally usable durations for high precision math with Epoch (all data is stored in seconds)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Duration(Decimal);

macro_rules! impl_ops_for_type {
    ($type:ident) => {
        impl Mul<$type> for TimeUnit {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Century => Duration::from_days(Decimal::from(DAYS_PER_CENTURY)),
                    TimeUnit::Day => Duration::from_days(Decimal::from(q)),
                    TimeUnit::Hour => Duration::from_hours(Decimal::from(q)),
                    TimeUnit::Minute => Duration::from_minutes(Decimal::from(q)),
                    TimeUnit::Second => Duration::from_seconds(Decimal::from(q)),
                    TimeUnit::Millisecond => Duration::from_milliseconds(Decimal::from(q)),
                    TimeUnit::Microsecond => Duration::from_microseconds(Decimal::from(q)),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds(Decimal::from(q)),
                }
            }
        }

        impl Mul<TimeUnit> for $type {
            type Output = Duration;
            fn mul(self, q: TimeUnit) -> Duration {
                match q {
                    TimeUnit::Century => Duration::from_days(Decimal::from(DAYS_PER_CENTURY)),
                    TimeUnit::Day => Duration::from_days(Decimal::from(self)),
                    TimeUnit::Hour => Duration::from_hours(Decimal::from(self)),
                    TimeUnit::Minute => Duration::from_minutes(Decimal::from(self)),
                    TimeUnit::Second => Duration::from_seconds(Decimal::from(self)),
                    TimeUnit::Millisecond => Duration::from_milliseconds(Decimal::from(self)),
                    TimeUnit::Microsecond => Duration::from_microseconds(Decimal::from(self)),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds(Decimal::from(self)),
                }
            }
        }
    };
}

impl Duration {
    pub fn from_days(days: Decimal) -> Self {
        Self {
            0: days * Decimal::from(SECONDS_PER_DAY),
        }
    }
    pub fn from_hours(hours: Decimal) -> Self {
        Self {
            0: hours * Decimal::from(SECONDS_PER_HOUR),
        }
    }
    pub fn from_minutes(minutes: Decimal) -> Self {
        Self {
            0: minutes * Decimal::from(SECONDS_PER_MINUTE),
        }
    }
    pub fn from_seconds(seconds: Decimal) -> Self {
        Self { 0: seconds }
    }
    pub fn from_milliseconds(ms: Decimal) -> Self {
        Self {
            0: ms * Decimal::from(1e-3),
        }
    }
    pub fn from_microseconds(ms: Decimal) -> Self {
        Self {
            0: ms * Decimal::from(1e-6),
        }
    }
    pub fn from_nanoseconds(ns: Decimal) -> Self {
        Self {
            0: ns * Decimal::from(1e-9),
        }
    }

    /// Creates a new duration from the provided unit
    pub fn from_f64(value: f64, unit: TimeUnit) -> Self {
        unit * value
    }

    /// Creates a new duration from the provided fraction and unit
    pub fn from_fraction(num: i64, denom: i64, unit: TimeUnit) -> Self {
        let num_u = num.abs() as u128;
        let denom_u = denom.abs() as u128;
        if (num < 0 && denom < 0) || (num > 0 && denom > 0) {
            Self(Decimal::from_fraction(Fraction::new(num_u, denom_u)) * unit.in_seconds())
        } else {
            Self(Decimal::from_fraction(Fraction::new_neg(num_u, denom_u)) * unit.in_seconds())
        }
    }

    /// Returns this duration in f64 in the provided unit.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    pub fn in_unit_f64(&self, unit: TimeUnit) -> f64 {
        self.in_unit(unit).to_f64().unwrap()
    }

    /// Returns the value of this duration in the requested unit.
    pub fn in_unit(&self, unit: TimeUnit) -> Decimal {
        self.0 * unit.from_seconds()
    }
}

impl fmt::Display for Duration {
    // Prints this duration with automatic selection of the highest and sub-second unit
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We should print all of the fields
        let days = self.in_unit(TimeUnit::Day).floor();
        let hours = self.in_unit(TimeUnit::Hour).floor() - days * Decimal::from(24.0);
        let minutes = self.in_unit(TimeUnit::Minute).floor()
            - self.in_unit(TimeUnit::Hour).floor() * Decimal::from(60.0);
        let seconds = self.in_unit(TimeUnit::Second).floor()
            - self.in_unit(TimeUnit::Minute).floor() * Decimal::from(60.0);
        let milli = self.in_unit(TimeUnit::Millisecond).floor()
            - self.in_unit(TimeUnit::Second).floor() * Decimal::from(1e3);
        let nano = self.in_unit(TimeUnit::Nanosecond)
            - self.in_unit(TimeUnit::Millisecond).floor() * Decimal::from(1e6);

        let mut print_all = false;
        let nil = Decimal::from(0);
        let is_neg = self.0 < nil;
        let neg_one = Decimal::from(-1);

        if days.abs() > nil {
            fmt::Display::fmt(&days, f)?;
            write!(f, " days ")?;
            print_all = true;
        }
        if hours.abs() > nil || print_all {
            if is_neg && print_all {
                // We have already printed the negative sign
                // So let's oppose this number
                fmt::Display::fmt(&(hours * neg_one), f)?;
            } else {
                fmt::Display::fmt(&hours, f)?;
            }
            write!(f, " h ")?;
            print_all = true;
        }
        if minutes.abs() > nil || print_all {
            if is_neg && print_all {
                fmt::Display::fmt(&(minutes * neg_one), f)?;
            } else {
                fmt::Display::fmt(&minutes, f)?;
            }
            write!(f, " min ")?;
            print_all = true;
        }
        // If the milliseconds and nanoseconds are nil, then we stop at the second level
        if milli.abs() == nil && nano.abs() == nil {
            if is_neg && print_all {
                fmt::Display::fmt(&(seconds * neg_one), f)?;
            } else {
                fmt::Display::fmt(&seconds, f)?;
            }
            write!(f, " s")
        } else {
            if seconds.abs() > nil || print_all {
                if is_neg && print_all {
                    fmt::Display::fmt(&(seconds * neg_one), f)?;
                } else {
                    fmt::Display::fmt(&seconds, f)?;
                }
                write!(f, " s ")?;
                print_all = true;
            }
            if nano == nil || (is_neg && nano * neg_one <= nil) {
                // Only stop at the millisecond level
                if is_neg && print_all {
                    fmt::Display::fmt(&(milli * neg_one), f)?;
                } else {
                    fmt::Display::fmt(&milli, f)?;
                }
                write!(f, " ms")
            } else {
                if milli.abs() > nil || print_all {
                    if is_neg && print_all {
                        fmt::Display::fmt(&(milli * neg_one), f)?;
                    } else {
                        fmt::Display::fmt(&milli, f)?;
                    }
                    write!(f, " ms ")?;
                }
                if is_neg && print_all {
                    fmt::Display::fmt(&(nano * neg_one), f)?;
                } else {
                    fmt::Display::fmt(&nano, f)?;
                }
                write!(f, " ns")
            }
        }
    }
}

impl fmt::LowerExp for Duration {
    // Prints the duration with appropriate units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds_f64 = self.0.to_f64().unwrap();
        let seconds_f64_abs = seconds_f64.abs();
        if seconds_f64_abs < 1e-5 {
            fmt::Display::fmt(&(seconds_f64 * 1e9), f)?;
            write!(f, " ns")
        } else if seconds_f64_abs < 1e-2 {
            fmt::Display::fmt(&(seconds_f64 * 1e3), f)?;
            write!(f, " ms")
        } else if seconds_f64_abs < 3.0 * SECONDS_PER_MINUTE {
            fmt::Display::fmt(&(seconds_f64), f)?;
            write!(f, " s")
        } else if seconds_f64_abs < SECONDS_PER_HOUR {
            fmt::Display::fmt(&(seconds_f64 / SECONDS_PER_MINUTE), f)?;
            write!(f, " min")
        } else if seconds_f64_abs < SECONDS_PER_DAY {
            fmt::Display::fmt(&(seconds_f64 / SECONDS_PER_HOUR), f)?;
            write!(f, " h")
        } else {
            fmt::Display::fmt(&(seconds_f64 / SECONDS_PER_DAY), f)?;
            write!(f, " days")
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Duration {
        Self { 0: self.0 + rhs.0 }
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        Self { 0: self.0 - rhs.0 }
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

// Allow adding with a TimeUnit directly
impl Add<TimeUnit> for Duration {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: TimeUnit) -> Duration {
        self + rhs * 1
    }
}

impl AddAssign<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn add_assign(&mut self, rhs: TimeUnit) {
        *self = *self + rhs * 1;
    }
}

impl Sub<TimeUnit> for Duration {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn sub(self, rhs: TimeUnit) -> Duration {
        self - rhs * 1
    }
}

impl SubAssign<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn sub_assign(&mut self, rhs: TimeUnit) {
        *self = *self - rhs * 1;
    }
}

impl PartialEq<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn eq(&self, unit: &TimeUnit) -> bool {
        *self == *unit * 1
    }
}

impl PartialOrd<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn partial_cmp(&self, unit: &TimeUnit) -> Option<Ordering> {
        let unit_deref = *unit;
        let unit_as_duration: Duration = unit_deref * 1;
        if self < &unit_as_duration {
            Some(Ordering::Less)
        } else if self > &unit_as_duration {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl FromStr for Duration {
    type Err = Errors;

    /// Attempts to convert a simple string to a Duration. Does not yet support complicated durations.
    ///
    /// Identifiers:
    ///  + d, days, day
    ///  + h, hours, hour
    ///  + min, mins, minute
    ///  + s, second, seconds
    ///  + ms, millisecond, milliseconds
    ///  + us, microsecond, microseconds
    ///  + ns, nanosecond, nanoseconds
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnit};
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Duration::from_str("1 d").unwrap(), TimeUnit::Day * 1);
    /// assert_eq!(Duration::from_str("10.598 days").unwrap(), TimeUnit::Day * 10.598);
    /// assert_eq!(Duration::from_str("10.598 min").unwrap(), TimeUnit::Minute * 10.598);
    /// assert_eq!(Duration::from_str("10.598 us").unwrap(), TimeUnit::Microsecond * 10.598);
    /// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), TimeUnit::Second * 10.598);
    /// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), TimeUnit::Nanosecond * 10.598);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg = Regex::new(r"^(\d+\.?\d*)\W*(\w+)$").unwrap();
        match reg.captures(s) {
            Some(cap) => {
                let value = cap[1].to_owned().parse::<f64>().unwrap();
                match cap[2].to_owned().to_lowercase().as_str() {
                    "d" | "days" | "day" => Ok(TimeUnit::Day * value),
                    "h" | "hours" | "hour" => Ok(TimeUnit::Hour * value),
                    "min" | "mins" | "minute" | "minutes" => Ok(TimeUnit::Minute * value),
                    "s" | "second" | "seconds" => Ok(TimeUnit::Second * value),
                    "ms" | "millisecond" | "milliseconds" => Ok(TimeUnit::Millisecond * value),
                    "us" | "microsecond" | "microseconds" => Ok(TimeUnit::Microsecond * value),
                    "ns" | "nanosecond" | "nanoseconds" => Ok(TimeUnit::Nanosecond * value),
                    _ => Err(Errors::ParseError(format!(
                        "unknown duration unit in `{}`",
                        s
                    ))),
                }
            }
            None => Err(Errors::ParseError(format!(
                "Could not parse duration: `{}`",
                s
            ))),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeUnit {
    /// 36525 days, it the number of days per century in the Julian calendar
    Century,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl Add for TimeUnit {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: Self) -> Duration {
        self * 1 + rhs * 1
    }
}

impl Sub for TimeUnit {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn sub(self, rhs: Self) -> Duration {
        self * 1 - rhs * 1
    }
}

impl TimeUnit {
    pub fn in_seconds(self) -> Decimal {
        match self {
            TimeUnit::Century => Decimal::from(DAYS_PER_CENTURY * SECONDS_PER_DAY),
            TimeUnit::Day => Decimal::from(SECONDS_PER_DAY),
            TimeUnit::Hour => Decimal::from(SECONDS_PER_HOUR),
            TimeUnit::Minute => Decimal::from(SECONDS_PER_MINUTE),
            TimeUnit::Second => Decimal::from(1.0),
            TimeUnit::Millisecond => Decimal::from(1e-3),
            TimeUnit::Microsecond => Decimal::from(1e-6),
            TimeUnit::Nanosecond => Decimal::from(1e-9),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_seconds(self) -> Decimal {
        Decimal::from(1) / self.in_seconds()
    }
}

impl_ops_for_type!(f32);
impl_ops_for_type!(f64);
impl_ops_for_type!(u8);
impl_ops_for_type!(i8);
impl_ops_for_type!(u16);
impl_ops_for_type!(i16);
impl_ops_for_type!(u32);
impl_ops_for_type!(i32);
impl_ops_for_type!(u64);
impl_ops_for_type!(i64);
impl_ops_for_type!(u128);

#[test]
fn time_unit() {
    use std::f64::EPSILON;
    // Check that the same number is created for different types
    assert_eq!(TimeUnit::Day * 10.0, TimeUnit::Day * 10);
    assert_eq!(TimeUnit::Hour * -7.0, TimeUnit::Hour * -7);
    assert_eq!(TimeUnit::Minute * -2.0, TimeUnit::Minute * -2);
    assert_eq!(TimeUnit::Second * 3.0, TimeUnit::Second * 3);
    assert_eq!(TimeUnit::Millisecond * 4.0, TimeUnit::Millisecond * 4);
    assert_eq!(TimeUnit::Nanosecond * 5.0, TimeUnit::Nanosecond * 5);

    // Check the LHS multiplications match the RHS ones
    assert_eq!(10.0 * TimeUnit::Day, TimeUnit::Day * 10);
    assert_eq!(-7 * TimeUnit::Hour, TimeUnit::Hour * -7.0);
    assert_eq!(-2.0 * TimeUnit::Minute, TimeUnit::Minute * -2);
    assert_eq!(3.0 * TimeUnit::Second, TimeUnit::Second * 3);
    assert_eq!(4.0 * TimeUnit::Millisecond, TimeUnit::Millisecond * 4);
    assert_eq!(5.0 * TimeUnit::Nanosecond, TimeUnit::Nanosecond * 5);

    // Test operations
    let seven_hours = TimeUnit::Hour * 7;
    let six_minutes = TimeUnit::Minute * 6;
    let five_seconds = TimeUnit::Second * 5.0;
    let sum: Duration = seven_hours + six_minutes + five_seconds;
    assert!((sum.in_unit_f64(TimeUnit::Second) - 25565.0).abs() < EPSILON);

    let sub: Duration = seven_hours - six_minutes - five_seconds;
    assert!((sub.in_unit_f64(TimeUnit::Second) - 24835.0).abs() < EPSILON);

    // Check printing adds precision
    assert_eq!(
        format!("{}", TimeUnit::Day * 10.0 + TimeUnit::Hour * 5),
        "10 days 5 h 0 min 0 s"
    );

    assert_eq!(
        format!("{}", TimeUnit::Hour * 5 + TimeUnit::Millisecond * 256),
        "5 h 0 min 0 s 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * 5 + TimeUnit::Millisecond * 256 + TimeUnit::Nanosecond
        ),
        "5 h 0 min 0 s 256 ms 1 ns"
    );

    assert_eq!(
        format!("{}", TimeUnit::Hour + TimeUnit::Second),
        "1 h 0 min 1 s"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * 5
                + TimeUnit::Millisecond * 256
                + TimeUnit::Microsecond
                + TimeUnit::Nanosecond * 3.5
        ),
        "5 h 0 min 0 s 256 ms 1003.5 ns"
    );

    // Check printing negative durations only shows one negative sign
    assert_eq!(
        format!("{}", TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256),
        "-5 h 0 min 0 s 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * -3.5
        ),
        "-5 h 0 min 0 s 256 ms 3.5 ns"
    );

    assert_eq!(
        format!(
            "{}",
            (TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256)
                - (TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * 2)
        ),
        "-2 ns"
    );

    assert_eq!(
        format!("{}", Duration::from_nanoseconds(Decimal::from(2))),
        "2 ns"
    );

    // Check that we support nanoseconds pas GPS time
    let now = TimeUnit::Nanosecond * 1286495254000000123_u128;
    assert_eq!(
        format!("{}", now),
        "14889 days 23 h 47 min 34 s 0 ms 123 ns"
    );

    // Test fractional
    let quarter_hour = Duration::from_fraction(1, 4, TimeUnit::Hour);
    let third_hour = Duration::from_fraction(1, 3, TimeUnit::Hour);
    let sum = quarter_hour + third_hour;
    assert!((sum.in_unit_f64(TimeUnit::Minute) - 35.0).abs() < EPSILON);
    println!(
        "Duration: {}\nFloating: {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum), "35 min 0 s"); // Note the automatic unit selection

    let quarter_hour = Duration::from_fraction(-1, 4, TimeUnit::Hour);
    let third_hour = Duration::from_fraction(1, -3, TimeUnit::Hour);
    let sum = quarter_hour + third_hour;
    let delta = sum.in_unit(TimeUnit::Millisecond).floor()
        - sum.in_unit(TimeUnit::Second).floor() * Decimal::from(1000.0);
    println!("{:?}", delta * Decimal::from(-1) == Decimal::from(0));
    assert!((sum.in_unit_f64(TimeUnit::Minute) + 35.0).abs() < EPSILON);
    assert_eq!(format!("{}", sum), "-35 min 0 s"); // Note the automatic unit selection
}
