extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate twofloat;

// use self::qd::Quad;
use self::regex::Regex;
use self::serde::{de, Deserialize, Deserializer};
use self::twofloat::TwoFloat;
use crate::{Errors, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use std::str::FromStr;

const DAYS_PER_CENTURY_U: u128 = 36_525;
const SECONDS_PER_MINUTE_U: u128 = 60;
const SECONDS_PER_HOUR_U: u128 = 3_600;
const SECONDS_PER_DAY_U: u128 = 86_400;
const ONE: u128 = 1_u128;

/// Defines generally usable durations for high precision math with Epoch (all data is stored in seconds)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Duration(TwoFloat);

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

macro_rules! impl_ops_for_type {
    ($type:ident) => {
        impl Mul<$type> for TimeUnit {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Century => {
                        Duration::from_days(TwoFloat::from(q) * TwoFloat::from(DAYS_PER_CENTURY_U))
                    }
                    TimeUnit::Day => Duration::from_days(TwoFloat::from(q)),
                    TimeUnit::Hour => Duration::from_hours(TwoFloat::from(q)),
                    TimeUnit::Minute => Duration::from_minutes(TwoFloat::from(q)),
                    TimeUnit::Second => Duration::from_seconds(TwoFloat::from(q)),
                    TimeUnit::Millisecond => Duration::from_milliseconds(TwoFloat::from(q)),
                    TimeUnit::Microsecond => Duration::from_microseconds(TwoFloat::from(q)),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds(TwoFloat::from(q)),
                }
            }
        }

        impl Mul<TimeUnit> for $type {
            type Output = Duration;
            fn mul(self, q: TimeUnit) -> Duration {
                match q {
                    TimeUnit::Century => Duration::from_days(
                        TwoFloat::from(self) * TwoFloat::from(DAYS_PER_CENTURY_U),
                    ),
                    TimeUnit::Day => Duration::from_days(TwoFloat::from(self)),
                    TimeUnit::Hour => Duration::from_hours(TwoFloat::from(self)),
                    TimeUnit::Minute => Duration::from_minutes(TwoFloat::from(self)),
                    TimeUnit::Second => Duration::from_seconds(TwoFloat::from(self)),
                    TimeUnit::Millisecond => Duration::from_milliseconds(TwoFloat::from(self)),
                    TimeUnit::Microsecond => Duration::from_microseconds(TwoFloat::from(self)),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds(TwoFloat::from(self)),
                }
            }
        }

        impl Mul<$type> for Duration {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                Self {
                    0: self.0 * TwoFloat::from(q),
                }
            }
        }

        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Duration {
                Self {
                    0: self.0 / TwoFloat::from(q),
                }
            }
        }

        impl Mul<Duration> for $type {
            type Output = Duration;
            fn mul(self, q: Duration) -> Duration {
                Duration {
                    0: q.0 * TwoFloat::from(self),
                }
            }
        }

        impl TimeUnitHelper for $type {}
    };
}

impl Duration {
    pub fn from_days(days: TwoFloat) -> Self {
        Self {
            0: days * TwoFloat::from(SECONDS_PER_DAY_U),
        }
    }
    pub fn from_hours(hours: TwoFloat) -> Self {
        Self {
            0: hours * TwoFloat::from(SECONDS_PER_HOUR_U),
        }
    }
    pub fn from_minutes(minutes: TwoFloat) -> Self {
        Self {
            0: minutes * TwoFloat::from(SECONDS_PER_MINUTE_U),
        }
    }
    pub fn from_seconds(seconds: TwoFloat) -> Self {
        Self { 0: seconds }
    }
    pub fn from_milliseconds(ms: TwoFloat) -> Self {
        Self {
            0: ms * TwoFloat::from(1e-3),
        }
    }
    pub fn from_microseconds(us: TwoFloat) -> Self {
        Self {
            0: us * TwoFloat::from(1e-6),
        }
    }
    pub fn from_nanoseconds(ns: TwoFloat) -> Self {
        Self {
            0: ns * TwoFloat::from(1e-9),
        }
    }

    /// Creates a new duration from the provided unit
    pub fn from_f64(value: f64, unit: TimeUnit) -> Self {
        unit * value
    }

    /// Returns this duration in f64 in the provided unit.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    pub fn in_unit_f64(&self, unit: TimeUnit) -> f64 {
        f64::from(self.in_unit(unit))
    }

    /// Returns this duration in seconds f64.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    pub fn in_seconds(&self) -> f64 {
        f64::from(self.0)
    }

    /// Returns the value of this duration in the requested unit.
    pub fn in_unit(&self, unit: TimeUnit) -> TwoFloat {
        self.0 * unit.from_seconds()
    }

    /// Returns the absolute value of this duration
    pub fn abs(&self) -> Self {
        if self.0 < TwoFloat::from(0) {
            Self { 0: -self.0 }
        } else {
            *self
        }
    }

    /// Builds a new duration from the hi and lo two-float values
    pub fn try_from_hi_lo(hi: f64, lo: f64) -> Result<Self, Errors> {
        match TwoFloat::try_from((hi, lo)) {
            Ok(t) => Ok(Self(t)),
            Err(_) => Err(Errors::ConversionOverlapError(hi, lo)),
        }
    }
}

impl TryFrom<(f64, f64)> for Duration {
    type Error = Errors;

    fn try_from(value: (f64, f64)) -> Result<Self, Self::Error> {
        Self::try_from_hi_lo(value.0, value.1)
    }
}

impl fmt::Display for Duration {
    // Prints this duration with automatic selection of the highest and sub-second unit
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let eps = 1e-1;
        let is_neg = self.0.is_sign_negative();

        let days_tf = self.in_unit(TimeUnit::Day);
        let mut days = days_tf.hi() + days_tf.lo();
        if days.abs() < eps {
            days = 0.0
        } else if is_neg {
            days = days.ceil()
        } else {
            days = days.floor()
        };
        let hours_tf = self.in_unit(TimeUnit::Hour) - days * TwoFloat::from(24.0);
        let mut hours = hours_tf.hi() + hours_tf.lo();
        if hours.abs() < eps {
            hours = 0.0
        } else if is_neg {
            hours = hours.ceil()
        } else {
            hours = hours.floor()
        };

        let minutes_tf = self.in_unit(TimeUnit::Minute)
            - hours * TwoFloat::from(60.0)
            - days * TwoFloat::from(24.0 * 60.0);
        let mut minutes = minutes_tf.hi() + minutes_tf.lo();
        if minutes.abs() < eps {
            minutes = 0.0
        } else if is_neg {
            minutes = minutes.ceil()
        } else {
            minutes = minutes.floor()
        };

        let seconds_tf = self.in_unit(TimeUnit::Second)
            - minutes * TwoFloat::from(60.0)
            - hours * TwoFloat::from(3600.0)
            - days * TwoFloat::from(24.0 * 3600.0);
        let mut seconds = seconds_tf.hi() + seconds_tf.lo();
        if seconds.abs() < eps {
            seconds = 0.0
        } else if is_neg {
            seconds = seconds.ceil()
        } else {
            seconds = seconds.floor()
        };

        let milli_tf = self.in_unit(TimeUnit::Millisecond)
            - seconds * TwoFloat::from(1e3)
            - minutes * TwoFloat::from(60.0 * 1e3)
            - hours * TwoFloat::from(3600.0 * 1e3)
            - days * TwoFloat::from(24.0 * 3600.0 * 1e3);
        let mut milli = milli_tf.hi() + milli_tf.lo();
        if milli.abs() < eps {
            milli = 0.0
        } else if is_neg {
            milli = milli.ceil()
        } else {
            milli = milli.floor()
        };

        // Compute the microseconds for precise nanosecond printing, but we don't actually print the microseconds
        let micro_tf = self.in_unit(TimeUnit::Microsecond)
            - milli * TwoFloat::from(1e3)
            - seconds * TwoFloat::from(1e6)
            - minutes * TwoFloat::from(60.0 * 1e6)
            - hours * TwoFloat::from(3600.0 * 1e6)
            - days * TwoFloat::from(24.0 * 3600.0 * 1e6);
        let micro = micro_tf.hi() + micro_tf.lo();

        let mut nano = 1e3 * micro;

        if nano.abs() < eps || (nano < 0.0 && !is_neg) {
            nano = 0.0
        } else {
            nano = format!("{:.3}", nano).parse().unwrap();
        }

        let mut print_all = false;
        let nil = TwoFloat::try_from((std::f64::EPSILON, 0.0)).unwrap();

        let neg_one = TwoFloat::from(-1);

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
                let neg_minutes = minutes * neg_one;
                fmt::Display::fmt(&(neg_minutes.hi() + neg_minutes.lo()), f)?;
            } else {
                fmt::Display::fmt(&minutes, f)?;
            }
            write!(f, " min ")?;
            print_all = true;
        }
        // If the milliseconds and nanoseconds are nil, then we stop at the second level
        if milli.abs() < nil && nano.abs() < nil {
            if is_neg && print_all {
                let neg_seconds = seconds * neg_one;
                fmt::Display::fmt(&(neg_seconds.hi() + neg_seconds.lo()), f)?;
            } else {
                fmt::Display::fmt(&seconds, f)?;
            }
            write!(f, " s")
        } else {
            if seconds.abs() > nil || print_all {
                if is_neg && print_all {
                    let neg_seconds = seconds * neg_one;
                    fmt::Display::fmt(&(neg_seconds.hi() + neg_seconds.lo()), f)?;
                } else {
                    fmt::Display::fmt(&seconds, f)?;
                }
                write!(f, " s ")?;
                print_all = true;
            }
            if nano.abs() < nil || (is_neg && nano * neg_one <= nil) {
                // Only stop at the millisecond level
                if is_neg && print_all {
                    let neg_milli = milli * neg_one;
                    fmt::Display::fmt(&(neg_milli.hi() + neg_milli.lo()), f)?;
                } else {
                    fmt::Display::fmt(&milli, f)?;
                }
                write!(f, " ms")
            } else {
                if milli.abs() > nil || print_all {
                    if is_neg && print_all {
                        let neg_milli = milli * neg_one;
                        fmt::Display::fmt(&(neg_milli.hi() + neg_milli.lo()), f)?;
                    } else {
                        fmt::Display::fmt(&milli, f)?;
                    }
                    write!(f, " ms ")?;
                }
                if is_neg && print_all {
                    let neg_nano = nano * neg_one;
                    fmt::Display::fmt(&(neg_nano.hi() + neg_nano.lo()), f)?;
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
        let seconds_f64 = f64::from(self.0);
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

impl Neg for Duration {
    type Output = Duration;

    fn neg(self) -> Self::Output {
        Self { 0: -self.0 }
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

/// A trait to automatically convert some primitives to a duration
///
/// ```
/// use hifitime::prelude::*;
/// use std::str::FromStr;
///
/// assert_eq!(Duration::from_str("1 d").unwrap(), 1.days());
/// assert_eq!(Duration::from_str("10.598 days").unwrap(), 10.598_f64.days());
/// assert_eq!(Duration::from_str("10.598 min").unwrap(), 10.598_f64.minutes());
/// assert_eq!(Duration::from_str("10.598 us").unwrap(), 10.598_f64.microseconds());
/// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), 10.598_f64.seconds());
/// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), 10.598_f64.nanoseconds());
/// ```
pub trait TimeUnitHelper: Copy + Mul<TimeUnit, Output = Duration> {
    fn centuries(self) -> Duration {
        self * TimeUnit::Century
    }
    fn days(self) -> Duration {
        self * TimeUnit::Day
    }
    fn hours(self) -> Duration {
        self * TimeUnit::Hour
    }
    fn minutes(self) -> Duration {
        self * TimeUnit::Minute
    }
    fn seconds(self) -> Duration {
        self * TimeUnit::Second
    }
    fn milliseconds(self) -> Duration {
        self * TimeUnit::Millisecond
    }
    fn microseconds(self) -> Duration {
        self * TimeUnit::Microsecond
    }
    fn nanoseconds(self) -> Duration {
        self * TimeUnit::Nanosecond
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
    pub fn in_seconds(self) -> TwoFloat {
        match self {
            TimeUnit::Century => TwoFloat::from(DAYS_PER_CENTURY_U * SECONDS_PER_DAY_U),
            TimeUnit::Day => TwoFloat::from(SECONDS_PER_DAY_U),
            TimeUnit::Hour => TwoFloat::from(SECONDS_PER_HOUR_U),
            TimeUnit::Minute => TwoFloat::from(SECONDS_PER_MINUTE_U),
            TimeUnit::Second => TwoFloat::from(ONE),
            TimeUnit::Millisecond => TwoFloat::from(1e-3),
            TimeUnit::Microsecond => TwoFloat::from(1e-6),
            TimeUnit::Nanosecond => TwoFloat::from(1e-9),
        }
    }

    pub fn in_seconds_f64(self) -> f64 {
        f64::from(self.in_seconds())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_seconds(self) -> TwoFloat {
        TwoFloat::from(1) / self.in_seconds()
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

    let d: Duration = 1.0 * TimeUnit::Hour / 3 - 20 * TimeUnit::Minute;
    assert!(d.abs() < TimeUnit::Nanosecond);
    assert_eq!(3 * (20 * TimeUnit::Minute), TimeUnit::Hour);

    // Test operations
    let seven_hours = TimeUnit::Hour * 7;
    let six_minutes = TimeUnit::Minute * 6;
    // let five_seconds = TimeUnit::Second * 5.0;
    let five_seconds = 5.0.seconds();
    let sum: Duration = seven_hours + six_minutes + five_seconds;
    assert!((sum.in_seconds() - 25565.0).abs() < EPSILON);

    let neg_sum = -sum;
    assert!((neg_sum.in_seconds() + 25565.0).abs() < EPSILON);

    assert_eq!(neg_sum.abs(), sum, "abs failed");

    let sub: Duration = seven_hours - six_minutes - five_seconds;
    assert!((sub.in_seconds() - 24835.0).abs() < EPSILON);

    // Test fractional
    let quarter_hour = 0.25 * TimeUnit::Hour;
    let third_hour = (1.0 / 3.0) * TimeUnit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    assert!((sum.in_unit_f64(TimeUnit::Minute) - 35.0).abs() < EPSILON);
    println!(
        "Duration: {}\nFloating: {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );

    let quarter_hour = -0.25 * TimeUnit::Hour;
    let third_hour: Duration = -1 * TimeUnit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.in_unit(TimeUnit::Millisecond).floor()
        - sum.in_unit(TimeUnit::Second).floor() * TwoFloat::from(1000.0);
    println!("{:?}", delta * TwoFloat::from(-1) == TwoFloat::from(0));
    assert!((sum.in_unit_f64(TimeUnit::Minute) + 35.0).abs() < EPSILON);
}

#[test]
fn duration_print() {
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
        format!("{}", Duration::from_nanoseconds(TwoFloat::from(2))),
        "2 ns"
    );

    // Check that we support nanoseconds pas GPS time
    let now = TimeUnit::Nanosecond * 1286495254000000123_u128;
    assert_eq!(
        format!("{}", now),
        "14889 days 23 h 47 min 34 s 0 ms 203.125 ns"
    );

    let arbitrary = 14889.days()
        + 23.hours()
        + 47.minutes()
        + 34.seconds()
        + 0.milliseconds()
        + 123.nanoseconds();
    assert_eq!(
        format!("{}", arbitrary),
        "14889 days 23 h 47 min 34 s 0 ms 123 ns"
    );

    // Test fractional
    let quarter_hour = 0.25 * TimeUnit::Hour;
    let third_hour = (1.0 / 3.0) * TimeUnit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    println!(
        "Duration: {}\nFloating: {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum), "35 min 0 s"); // Note the automatic unit selection

    let quarter_hour = -0.25 * TimeUnit::Hour;
    let third_hour: Duration = -1 * TimeUnit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.in_unit(TimeUnit::Millisecond).floor()
        - sum.in_unit(TimeUnit::Second).floor() * TwoFloat::from(1000.0);
    println!("{:?}", delta * TwoFloat::from(-1) == TwoFloat::from(0));
    assert_eq!(format!("{}", sum), "-35 min 0 s"); // Note the automatic unit selection
}

#[test]
fn deser_test() {
    use self::serde_derive::Deserialize;
    #[derive(Deserialize)]
    struct _D {
        pub _d: Duration,
    }
}
