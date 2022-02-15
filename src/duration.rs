extern crate divrem;
extern crate regex;
extern crate serde;
extern crate serde_derive;

use self::divrem::DivRemEuclid;
use self::regex::Regex;
use self::serde::{de, Deserialize, Deserializer};
use crate::{
    Errors, DAYS_PER_CENTURY, SECONDS_PER_CENTURY, SECONDS_PER_DAY, SECONDS_PER_HOUR,
    SECONDS_PER_MINUTE,
};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use std::str::FromStr;

const DAYS_PER_CENTURY_U64: u64 = 36_525;
const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;
const NANOSECONDS_PER_MILLISECOND: u64 = 1_000 * NANOSECONDS_PER_MICROSECOND;
const NANOSECONDS_PER_SECOND: u64 = 1_000 * NANOSECONDS_PER_MILLISECOND;
const NANOSECONDS_PER_MINUTE: u64 = 60 * NANOSECONDS_PER_SECOND;
const NANOSECONDS_PER_HOUR: u64 = 60 * NANOSECONDS_PER_MINUTE;
const NANOSECONDS_PER_DAY: u64 = 24 * NANOSECONDS_PER_HOUR;
const NANOSECONDS_PER_CENTURY: u64 = DAYS_PER_CENTURY_U64 * NANOSECONDS_PER_DAY;

/// Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.
///
/// **Important conventions:**
/// Conventions had to be made to define the partial order of a duration.
/// It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
/// a durationn with centuries = -1 and nanoseconds = 0 is _a smaller duration_ than centuries = -1 and nanoseconds = 1.
/// That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
/// As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
#[derive(Clone, Copy, Debug, PartialOrd, Eq, Ord)]
pub struct Duration {
    centuries: i16,
    nanoseconds: u64,
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        if self.centuries == other.centuries {
            self.nanoseconds == other.nanoseconds
        } else if (self.centuries - other.centuries).abs() == 1
            && (self.centuries == 0 || other.centuries == 0)
        {
            // Special case where we're at the zero crossing
            if self.centuries < 0 {
                // Self is negative,
                (NANOSECONDS_PER_CENTURY - self.nanoseconds) == other.nanoseconds
            } else {
                // Other is negative
                (NANOSECONDS_PER_CENTURY - other.nanoseconds) == self.nanoseconds
            }
        } else {
            false
        }
    }
}

impl Duration {
    fn normalize(&mut self) {
        let extra_centuries = self.nanoseconds.div_euclid(NANOSECONDS_PER_CENTURY);
        // We can skip this whole step if the div_euclid shows that we didn't overflow the number of nanoseconds per century
        if extra_centuries > 0 {
            let rem_nanos = self.nanoseconds.rem_euclid(NANOSECONDS_PER_CENTURY);

            if self.centuries == i16::MIN && rem_nanos > 0 {
                // We're at the min number of centuries already, and we have extra nanos, so we're saturated the duration limit
                *self = Self::MIN;
            } else if self.centuries == i16::MAX && rem_nanos > 0 {
                // Saturated max
                *self = Self::MAX;
            } else if self.centuries >= 0 {
                // Check that we can safely cast because we have that room without overflowing
                if (i16::MAX - self.centuries) as u64 >= extra_centuries {
                    // We can safely add without an overflow
                    self.centuries += extra_centuries as i16;
                    self.nanoseconds = rem_nanos;
                } else {
                    // Saturated max again
                    *self = Self::MAX;
                }
            } else {
                assert!(self.centuries < 0, "this shouldn't be possible");

                // Check that we can safely cast because we have that room without overflowing
                if (i16::MIN - self.centuries) as u64 >= extra_centuries {
                    // We can safely add without an overflow
                    self.centuries += extra_centuries as i16;
                    self.nanoseconds = rem_nanos;
                } else {
                    // Saturated max again
                    *self = Self::MIN;
                }
            }
        }
    }

    /// Create a normalized duration from its parts
    pub fn from_parts(centuries: i16, nanoseconds: u64) -> Self {
        let mut me = Self {
            centuries,
            nanoseconds,
        };
        me.normalize();
        me
    }

    /// Converts the total nanoseconds as i128 into this Duration (saving 48 bits)
    pub fn from_total_nanoseconds(nanos: i128) -> Self {
        // In this function, we simply check that the input data can be casted. The `normalize` function will check whether more work needs to be done.
        if nanos == 0 {
            Self::ZERO
        } else {
            let centuries_i128 = nanos.div_euclid(NANOSECONDS_PER_CENTURY.into());
            let remaining_nanos_i128 = nanos.rem_euclid(NANOSECONDS_PER_CENTURY.into());
            if centuries_i128 > i16::MAX.into() {
                Self::MAX
            } else if centuries_i128 < i16::MIN.into() {
                Self::MIN
            } else {
                // We know that the centuries fit, and we know that the nanos are less than the number
                // of nanos per centuries, and rem_euclid guarantees that it's positive, so the
                // casting will work fine every time.
                Self::from_parts(centuries_i128 as i16, remaining_nanos_i128 as u64)
            }
        }
    }

    /// Returns the total nanoseconds in a signed 128 bit integer
    #[must_use]
    pub fn total_nanoseconds(self) -> i128 {
        if self.centuries == -1 {
            -1 * i128::from(NANOSECONDS_PER_CENTURY - self.nanoseconds)
        } else if self.centuries >= 0 {
            i128::from(self.centuries) * i128::from(NANOSECONDS_PER_CENTURY)
                + i128::from(self.nanoseconds)
        } else {
            // Centuries negative by a decent amount
            i128::from(self.centuries + 1) * i128::from(NANOSECONDS_PER_CENTURY)
                + i128::from(self.nanoseconds)
        }
    }

    /// Returns the truncated nanoseconds in a signed 64 bit integer, if the duration fits.
    #[must_use]
    pub fn try_truncated_nanoseconds(self) -> Result<i64, Errors> {
        // If it fits, we know that the nanoseconds also fit
        if self.centuries.abs() >= 3 {
            Err(Errors::Overflow)
        } else if self.centuries == -1 {
            Ok(-1 * (NANOSECONDS_PER_CENTURY - self.nanoseconds) as i64)
        } else if self.centuries >= 0 {
            Ok(
                i64::from(self.centuries) * NANOSECONDS_PER_CENTURY as i64
                    + self.nanoseconds as i64,
            )
        } else {
            // Centuries negative by a decent amount
            Ok(
                i64::from(self.centuries + 1) * NANOSECONDS_PER_CENTURY as i64
                    + self.nanoseconds as i64,
            )
        }
    }

    /// Returns the truncated nanoseconds in a signed 64 bit integer, if the duration fits.
    /// WARNING: This function will NOT fail and will return the i64::MIN or i64::MAX depending on
    /// the sign of the centuries if the Duration does not fit on aa i64
    #[must_use]
    pub fn truncated_nanoseconds(self) -> i64 {
        match self.try_truncated_nanoseconds() {
            Ok(val) => val,
            Err(_) => {
                if self.centuries < 0 {
                    i64::MIN
                } else {
                    i64::MAX
                }
            }
        }
    }

    /// Create a new duration from the truncated nanoseconds (+/- 2927.1 years of duration)
    pub fn from_truncated_nanoseconds(nanos: i64) -> Self {
        if nanos < 0 {
            Self::from_parts(-1, NANOSECONDS_PER_CENTURY - (nanos.abs() as u64))
        } else {
            Self::from_parts(0, nanos.abs() as u64)
        }
    }

    /// Creates a new duration from the provided unit
    #[must_use]
    pub fn from_f64(value: f64, unit: TimeUnit) -> Self {
        unit * value
    }

    /// Returns this duration in f64 in the provided unit.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    #[must_use]
    pub fn in_unit_f64(&self, unit: TimeUnit) -> f64 {
        f64::from(self.in_unit(unit))
    }

    /// Returns this duration in seconds f64.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    #[must_use]
    pub fn in_seconds(&self) -> f64 {
        // Compute the seconds and nanoseconds that we know this fits on a 64bit float
        let seconds = self.nanoseconds.div_euclid(NANOSECONDS_PER_SECOND);
        let subseconds = self.nanoseconds.rem_euclid(NANOSECONDS_PER_SECOND);
        f64::from(self.centuries) * SECONDS_PER_CENTURY
            + (seconds as f64)
            + (subseconds as f64) * 1e-9
    }

    /// Returns the value of this duration in the requested unit.
    #[must_use]
    pub fn in_unit(&self, unit: TimeUnit) -> f64 {
        self.in_seconds() * unit.from_seconds()
    }

    /// Returns the absolute value of this duration
    #[must_use]
    pub fn abs(&self) -> Self {
        if self.centuries.is_negative() {
            -*self
        } else {
            *self
        }
    }

    /// Builds a new duration from the number of centuries and the number of nanoseconds
    #[must_use]
    pub fn new(centuries: i16, nanoseconds: u64) -> Self {
        let mut out = Self {
            centuries,
            nanoseconds,
        };
        out.normalize();
        out
    }

    /// Decomposes a Duration in its sign,  days,
    #[must_use]
    pub fn decompose(&self) -> (i8, u64, u64, u64, u64, u64, u64, u64) {
        let sign = self.centuries.signum() as i8;
        let total_ns = self.total_nanoseconds();

        let ns_left = total_ns.abs();

        let (days, ns_left) = ns_left.div_rem_euclid(i128::from(NANOSECONDS_PER_DAY));
        let (hours, ns_left) = ns_left.div_rem_euclid(i128::from(NANOSECONDS_PER_HOUR));
        let (minutes, ns_left) = ns_left.div_rem_euclid(i128::from(NANOSECONDS_PER_MINUTE));
        let (seconds, ns_left) = ns_left.div_rem_euclid(i128::from(NANOSECONDS_PER_SECOND));
        let (milliseconds, ns_left) =
            ns_left.div_rem_euclid(i128::from(NANOSECONDS_PER_MILLISECOND));
        let (microseconds, ns_left) =
            ns_left.div_rem_euclid(i128::from(NANOSECONDS_PER_MICROSECOND));

        // Everything should fit in the expected types now
        (
            sign,
            days.try_into().unwrap(),
            hours.try_into().unwrap(),
            minutes.try_into().unwrap(),
            seconds.try_into().unwrap(),
            milliseconds.try_into().unwrap(),
            microseconds.try_into().unwrap(),
            ns_left.try_into().unwrap(),
        )
    }

    /// A duration of exactly zero nanoseconds
    const ZERO: Self = Self {
        centuries: 0,
        nanoseconds: 0,
    };

    /// Maximum duration that can be represented
    pub const MAX: Self = Self {
        centuries: i16::MAX,
        nanoseconds: NANOSECONDS_PER_CENTURY,
    };

    /// Minimum duration that can be represented
    pub const MIN: Self = Self {
        centuries: i16::MIN,
        nanoseconds: NANOSECONDS_PER_CENTURY,
    };

    /// Smallest duration that can be represented
    pub const EPSILON: Self = Self {
        centuries: 0,
        nanoseconds: 1,
    };

    /// Minimum positive duration is one nanoseconds
    pub const MIN_POSITIVE: Self = Self::EPSILON;

    /// Minimum negative duration is minus one nanosecond
    pub const MIN_NEGATIVE: Self = Self {
        centuries: -1,
        nanoseconds: NANOSECONDS_PER_CENTURY - 1,
    };
}

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

            /// Converts the input values to i128 and creates a duration from that
            /// This method will necessarily ignore durations below nanoseconds
            fn mul(self, q: $type) -> Duration {
                let total_ns = match self {
                    TimeUnit::Century => q * (NANOSECONDS_PER_CENTURY as $type),
                    TimeUnit::Day => q * (NANOSECONDS_PER_DAY as $type),
                    TimeUnit::Hour => q * (NANOSECONDS_PER_HOUR as $type),
                    TimeUnit::Minute => q * (NANOSECONDS_PER_MINUTE as $type),
                    TimeUnit::Second => q * (NANOSECONDS_PER_SECOND as $type),
                    TimeUnit::Millisecond => q * (NANOSECONDS_PER_MILLISECOND as $type),
                    TimeUnit::Microsecond => q * (NANOSECONDS_PER_MICROSECOND as $type),
                    TimeUnit::Nanosecond => q,
                };
                if total_ns.abs() < (i64::MAX as $type) {
                    Duration::from_truncated_nanoseconds(total_ns as i64)
                } else {
                    Duration::from_total_nanoseconds(total_ns as i128)
                }
            }
        }

        impl Mul<TimeUnit> for $type {
            type Output = Duration;
            fn mul(self, q: TimeUnit) -> Duration {
                // Apply the reflexive property
                q * self
            }
        }

        impl Mul<$type> for Duration {
            type Output = Duration;
            fn mul(self, q: $type) -> Self::Output {
                Duration::from_total_nanoseconds(
                    self.total_nanoseconds()
                        .saturating_mul((q * TimeUnit::Nanosecond).total_nanoseconds()),
                )
            }
        }

        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Self::Output {
                Duration::from_total_nanoseconds(
                    self.total_nanoseconds()
                        .saturating_div((q * TimeUnit::Nanosecond).total_nanoseconds()),
                )
            }
        }

        impl Mul<Duration> for $type {
            type Output = Duration;
            fn mul(self, q: Self::Output) -> Self::Output {
                // Apply the reflexive property
                q * self
            }
        }

        impl TimeUnitHelper for $type {}
    };
}

impl fmt::Display for Duration {
    // Prints this duration with automatic selection of the units, i.e. everything that isn't zero is ignored
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.total_nanoseconds() == 0 {
            write!(f, "0 ns")
        } else {
            let (sign, days, hours, minutes, seconds, milli, us, nano) = self.decompose();
            if sign == -1 {
                write!(f, "-")?;
            }

            let values = [days, hours, minutes, seconds, milli, us, nano];
            let units = ["days", "h", "min", "s", "ms", "μs", "ns"];

            let mut insert_space = false;
            for (val, unit) in values.iter().zip(units.iter()) {
                if *val > 0 {
                    if insert_space {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", val, unit)?;
                    insert_space = true;
                }
            }
            Ok(())
        }
    }
}

impl fmt::LowerExp for Duration {
    // Prints the duration with appropriate units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds_f64 = self.in_seconds();
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
        // Check that the addition fits in an i16
        let mut me = self;
        match me.centuries.checked_add(rhs.centuries) {
            None => {
                // Overflowed, so we've hit the max
                return Self::MAX;
            }
            Some(centuries) => {
                me.centuries = centuries;
                // if self.centuries < 0 && rhs.centuries >= 0 {
                //     me.centuries += 1;
                // }
            }
        }
        // We can safely add two nanoseconds together because we can fit five centuries in one u64
        // cf. https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b4011b1d5c06c38a72f28d0a9e6a5574
        me.nanoseconds += rhs.nanoseconds;

        me.normalize();
        me
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
        // Check that the subtraction fits in an i16
        let mut me = self;
        match me.centuries.checked_sub(rhs.centuries) {
            None => {
                // Underflowed, so we've hit the max
                return Self::MIN;
            }
            Some(centuries) => {
                me.centuries = centuries;
            }
        }

        match me.nanoseconds.checked_sub(rhs.nanoseconds) {
            None => {
                // Decrease the number of centuries, and realign
                me.centuries -= 1;
                me.nanoseconds = me.nanoseconds + NANOSECONDS_PER_CENTURY - rhs.nanoseconds;
            }
            Some(nanos) => {
                if self.centuries >= 0 && rhs.centuries < 0 {
                    // Account for zero crossing
                    me.nanoseconds = nanos + 1
                } else {
                    me.nanoseconds = nanos
                }
            }
        };

        me.normalize();
        me
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
    type Output = Self;

    #[must_use]
    fn neg(self) -> Self::Output {
        Self::from_parts(
            -self.centuries - 1,
            NANOSECONDS_PER_CENTURY - self.nanoseconds,
        )
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
    // TODO: Find a better name for this, it's a pain to import and use
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
    #[must_use]
    pub fn in_seconds(self) -> f64 {
        match self {
            TimeUnit::Century => DAYS_PER_CENTURY * SECONDS_PER_DAY,
            TimeUnit::Day => SECONDS_PER_DAY,
            TimeUnit::Hour => SECONDS_PER_HOUR,
            TimeUnit::Minute => SECONDS_PER_MINUTE,
            TimeUnit::Second => 1.0,
            TimeUnit::Millisecond => 1e-3,
            TimeUnit::Microsecond => 1e-6,
            TimeUnit::Nanosecond => 1e-9,
        }
    }

    // #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn from_seconds(self) -> f64 {
        1.0 / self.in_seconds()
    }
}

impl_ops_for_type!(f64);
impl_ops_for_type!(i64);

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
    dbg!(d);
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
    dbg!(quarter_hour, third_hour, sum);
    assert!((sum.in_unit_f64(TimeUnit::Minute) - 35.0).abs() < EPSILON);

    let quarter_hour = -0.25 * TimeUnit::Hour;
    let third_hour: Duration = -1 * TimeUnit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta =
        sum.in_unit(TimeUnit::Millisecond).floor() - sum.in_unit(TimeUnit::Second).floor() * 1000.0;
    assert!(delta < std::f64::EPSILON);
    println!("{:?}", sum.in_seconds());
    assert!((sum.in_unit_f64(TimeUnit::Minute) + 35.0).abs() < EPSILON);
}

#[test]
fn duration_print() {
    // Check printing adds precision
    assert_eq!(
        format!("{}", TimeUnit::Day * 10.0 + TimeUnit::Hour * 5),
        "10 days 5 h"
    );

    assert_eq!(
        format!("{}", TimeUnit::Hour * 5 + TimeUnit::Millisecond * 256),
        "5 h 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * 5 + TimeUnit::Millisecond * 256 + TimeUnit::Nanosecond
        ),
        "5 h 256 ms 1 ns"
    );

    assert_eq!(format!("{}", TimeUnit::Hour + TimeUnit::Second), "1 h 1 s");

    // NOTE: We _try_ to specify down to a half of a nanosecond, but duration is NOT
    // more precise than that, so it only actually stores to that number.
    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * 5
                + TimeUnit::Millisecond * 256
                + TimeUnit::Microsecond
                + TimeUnit::Nanosecond * 3.5
        ),
        "5 h 256 ms 1 μs 3 ns"
    );

    // Check printing negative durations only shows one negative sign
    assert_eq!(
        format!("{}", TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256),
        "-5 h 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * -3
        ),
        "-5 h 256 ms 3 ns"
    );

    assert_eq!(
        format!(
            "{}",
            (TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256)
                - (TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * 2)
        ),
        "-2 ns"
    );

    assert_eq!(format!("{}", TimeUnit::Nanosecond * 2), "2 ns");

    // Check that we support nanoseconds pas GPS time
    let now = TimeUnit::Nanosecond * 1286495254000000123;
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
    let quarter_hour = 0.25 * TimeUnit::Hour;
    let third_hour = (1.0 / 3.0) * TimeUnit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    println!(
        "Proof that Duration is more precise than f64: {} vs {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum), "35 min");

    let quarter_hour = -0.25 * TimeUnit::Hour;
    let third_hour: Duration = -1 * TimeUnit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    dbg!(sum.total_nanoseconds());
    let delta =
        sum.in_unit(TimeUnit::Millisecond).floor() - sum.in_unit(TimeUnit::Second).floor() * 1000.0;
    println!("{:?}", (delta * -1.0) == 0.0);
    dbg!(sum);
    assert_eq!(format!("{}", sum), "-35 min");
}

#[test]
fn test_ops() {
    assert_eq!(
        (0.25 * TimeUnit::Hour).total_nanoseconds(),
        (15 * NANOSECONDS_PER_MINUTE).into()
    );

    assert_eq!(
        (-0.25 * TimeUnit::Hour).total_nanoseconds(),
        i128::from(15 * NANOSECONDS_PER_MINUTE) * -1
    );

    assert_eq!(
        (-0.25 * TimeUnit::Hour - 0.25 * TimeUnit::Hour).total_nanoseconds(),
        i128::from(30 * NANOSECONDS_PER_MINUTE) * -1
    );

    println!("{}", -0.25 * TimeUnit::Hour + (-0.25 * TimeUnit::Hour));

    assert_eq!(
        Duration::MIN_POSITIVE + 4 * Duration::MIN_POSITIVE,
        5 * TimeUnit::Nanosecond
    );

    assert_eq!(
        Duration::MIN_NEGATIVE + 4 * Duration::MIN_NEGATIVE,
        -5 * TimeUnit::Nanosecond
    );
}

#[test]
fn test_neg() {
    assert_eq!(Duration::MIN_NEGATIVE, -Duration::MIN_POSITIVE);
    assert_eq!(Duration::MIN_POSITIVE, -Duration::MIN_NEGATIVE);
    assert_eq!(2.nanoseconds(), -(2.0.nanoseconds()));
}

#[test]
fn deser_test() {
    use self::serde_derive::Deserialize;
    #[derive(Deserialize)]
    struct _D {
        pub _d: Duration,
    }
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
    dbg!(Duration::MIN_NEGATIVE, Duration::MIN_POSITIVE);
    assert_eq!(
        Duration::MIN_POSITIVE - Duration::MIN_NEGATIVE,
        2 * TimeUnit::Nanosecond
    );
    assert_eq!(
        Duration::MIN_NEGATIVE - Duration::MIN_POSITIVE,
        -2 * TimeUnit::Nanosecond
    );
    assert_eq!(
        Duration::from_total_nanoseconds(2),
        2 * TimeUnit::Nanosecond
    );
    // Check that we do not support more precise than nanosecond
    assert_eq!(TimeUnit::Nanosecond * 3.5, TimeUnit::Nanosecond * 3);

    assert_eq!(
        Duration::MIN_POSITIVE + Duration::MIN_NEGATIVE,
        Duration::ZERO
    );

    assert_eq!(
        Duration::MIN_NEGATIVE + Duration::MIN_NEGATIVE,
        -2 * TimeUnit::Nanosecond
    );

    // Add i64 tests
    let d = Duration::from_truncated_nanoseconds(i64::MAX);
    println!("{}", d);
    assert_eq!(
        Duration::from_truncated_nanoseconds(d.truncated_nanoseconds()),
        d
    );
}
