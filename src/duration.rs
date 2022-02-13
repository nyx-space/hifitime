extern crate divrem;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate twofloat;

use self::divrem::DivRemEuclid;
use self::regex::Regex;
use self::serde::{de, Deserialize, Deserializer};
#[allow(unused_imports)]
use self::twofloat::TwoFloat;
use crate::{
    Errors, DAYS_PER_CENTURY, SECONDS_PER_CENTURY, SECONDS_PER_DAY, SECONDS_PER_HOUR,
    SECONDS_PER_MINUTE,
};
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use std::str::FromStr;

const DAYS_PER_CENTURY_U: u128 = 36_525;
const DAYS_PER_CENTURY_U64: u64 = 36_525;
const HOURS_PER_CENTURY: f64 = 24.0 * DAYS_PER_CENTURY;
const MINUTES_PER_CENTURY: f64 = 60.0 * HOURS_PER_CENTURY;
const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;
const NANOSECONDS_PER_MILLISECOND: u64 = 1_000 * NANOSECONDS_PER_MICROSECOND;
const NANOSECONDS_PER_SECOND: u64 = 1_000 * NANOSECONDS_PER_MILLISECOND;
const NANOSECONDS_PER_MINUTE: u64 = 60 * NANOSECONDS_PER_SECOND;
const NANOSECONDS_PER_HOUR: u64 = 60 * NANOSECONDS_PER_MINUTE;
const NANOSECONDS_PER_DAY: u64 = 24 * NANOSECONDS_PER_HOUR;
const NANOSECONDS_PER_CENTURY: u64 = DAYS_PER_CENTURY_U64 * NANOSECONDS_PER_DAY;

/// Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Duration {
    pub centuries: i16,
    pub nanoseconds: u64,
}

impl Duration {
    fn normalize(&mut self) {
        if self.nanoseconds > NANOSECONDS_PER_CENTURY {
            let (centuries, nanoseconds) = self.nanoseconds.div_rem_euclid(NANOSECONDS_PER_CENTURY);
            self.nanoseconds = nanoseconds;
            // If the quotient is larger than an i16, set the centuries to the max.
            // Similarly, we will ensure with a saturating add that the centuries always fits in an i16
            match i16::try_from(centuries) {
                Ok(centuries) => self.centuries = self.centuries.saturating_add(centuries),
                Err(_) => self.centuries = i16::MAX,
            }
        }
    }

    /// Converts the total nanoseconds as i128 into this Duration (saving 48 bits)
    pub fn from_total_nanoseconds(nanos: i128) -> Self {
        if nanos < 0 {
            let mut centuries = -1;
            let mut nanoseconds: u64;
            let truncated;
            let usable_nanos = if nanos == i128::MIN {
                truncated = true;
                nanos + 1
            } else {
                truncated = false;
                nanos
            };
            if usable_nanos.abs() >= u64::MAX.into() {
                centuries -= 1;
                nanoseconds = (usable_nanos.abs() - i128::from(NANOSECONDS_PER_CENTURY)) as u64;
            } else {
                // We know it fits!
                nanoseconds = usable_nanos.abs() as u64;
            }
            if truncated {
                nanoseconds -= 1;
            }
            let mut me = Self {
                centuries,
                nanoseconds,
            };
            me.normalize();
            me
        } else {
            let mut centuries = 0;
            let nanoseconds: u64;
            // We must check that we fit in a u64, or the normalize function cannot work!
            if nanos >= u64::MAX.into() {
                centuries += 1;
                nanoseconds = (nanos - i128::from(NANOSECONDS_PER_CENTURY)) as u64;
            } else {
                // We know it fits!
                nanoseconds = nanos as u64;
            }
            let mut me = Self {
                centuries,
                nanoseconds,
            };
            me.normalize();
            me
        }
    }

    /// Returns the total nanoseconds in a signed 128 bit integer
    pub fn total_nanoseconds(self) -> i128 {
        i128::from(self.centuries) * i128::from(NANOSECONDS_PER_CENTURY)
            + i128::from(self.nanoseconds)
    }

    /// Creates a new duration from the provided unit
    #[must_use]
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
        // Compute the seconds and nanoseconds that we know this fits on a 64bit float
        let (seconds_u64, nanoseconds_u64) =
            self.nanoseconds.div_rem_euclid(NANOSECONDS_PER_CENTURY);
        f64::from(self.centuries) * SECONDS_PER_DAY * DAYS_PER_CENTURY
            + (seconds_u64 as f64)
            + (nanoseconds_u64 as f64) * 1e-9
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

    pub fn decompose(&self) -> (i8, u64, u64, u64, u64, u64, u64, u64) {
        let total_ns = self.total_nanoseconds();

        let sign = total_ns.signum() as i8;
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

    /// Maximum duration that can be represented
    pub const MAX: Self = Self {
        centuries: i16::MAX,
        nanoseconds: u64::MAX,
    };

    /// Minimum duration that can be represented
    pub const MIN: Self = Self {
        centuries: i16::MIN,
        nanoseconds: u64::MAX,
    };

    /// Smallest duration that can be represented
    pub const EPSILON: Self = Self {
        centuries: 0,
        nanoseconds: 1,
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

            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Century => self * q * DAYS_PER_CENTURY_U,
                    TimeUnit::Day => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid(DAYS_PER_CENTURY as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds =
                            q.rem_euclid(DAYS_PER_CENTURY as $type) as u64 * NANOSECONDS_PER_DAY;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
                    TimeUnit::Hour => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid(HOURS_PER_CENTURY as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds =
                            q.rem_euclid(HOURS_PER_CENTURY as $type) as u64 * NANOSECONDS_PER_HOUR;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
                    TimeUnit::Minute => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid(MINUTES_PER_CENTURY as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds = q.rem_euclid(MINUTES_PER_CENTURY as $type) as u64
                            * NANOSECONDS_PER_MINUTE;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
                    TimeUnit::Second => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid(SECONDS_PER_CENTURY as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds = q.rem_euclid(SECONDS_PER_CENTURY as $type) as u64
                            * NANOSECONDS_PER_SECOND;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
                    TimeUnit::Millisecond => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid((SECONDS_PER_CENTURY * 1e-3) as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds =
                            q.rem_euclid(DAYS_PER_CENTURY as $type) as u64 * NANOSECONDS_PER_SECOND;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
                    TimeUnit::Microsecond => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid((SECONDS_PER_CENTURY * 1e-6) as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds = q.rem_euclid(DAYS_PER_CENTURY as $type) as u64
                            * NANOSECONDS_PER_MICROSECOND;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
                    TimeUnit::Nanosecond => {
                        // The centuries will be a round number here so the `as` conversion should work.
                        let centuries_typed = q.div_euclid((SECONDS_PER_CENTURY * 1e-9) as $type);
                        let centuries = if centuries_typed > (i16::MAX as $type) {
                            return Duration::MAX;
                        } else if centuries_typed < (i16::MIN as $type) {
                            return Duration::MIN;
                        } else {
                            centuries_typed as i16
                        };

                        // rem_euclid returns the nonnegative number, so we can cast that directly into u64
                        let nanoseconds = q.rem_euclid(DAYS_PER_CENTURY as $type) as u64;
                        Duration {
                            centuries,
                            nanoseconds,
                        }
                    }
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
                // Compute as nanoseconds to align with Duration, and divide as needed.
                let mut as_duration = q * TimeUnit::Nanosecond;
                let mut me = self;
                me.centuries = me.centuries.saturating_mul(as_duration.centuries);
                if me.centuries == i16::MAX {
                    return Self::Output::MAX;
                } else if me.centuries == i16::MIN {
                    return Self::Output::MIN;
                }
                me.nanoseconds = me.nanoseconds.saturating_mul(as_duration.nanoseconds);
                if me.nanoseconds == u64::MAX {
                    // Increment the centuries and decrease nanoseconds
                    as_duration.centuries += 1;
                    as_duration.nanoseconds -= NANOSECONDS_PER_CENTURY;
                    // And repeat
                    me.centuries = me.centuries.saturating_mul(as_duration.centuries);
                    if me.centuries == i16::MAX {
                        return Self::Output::MAX;
                    } else if me.centuries == i16::MIN {
                        return Self::Output::MIN;
                    }
                    me.nanoseconds = me.nanoseconds.saturating_mul(as_duration.nanoseconds);
                }
                me
            }
        }

        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Self::Output {
                // Compute as nanoseconds to align with Duration, and divide as needed.
                let mut as_duration = q * TimeUnit::Nanosecond;
                let mut me = self;
                if as_duration.centuries > 0 {
                    me.centuries = me.centuries.saturating_div(as_duration.centuries);
                    if me.centuries == i16::MAX {
                        return Self::Output::MAX;
                    } else if me.centuries == i16::MIN {
                        return Self::Output::MIN;
                    }
                }
                if as_duration.nanoseconds > 0 {
                    me.nanoseconds = me.nanoseconds.saturating_div(as_duration.nanoseconds);
                    if me.nanoseconds == u64::MAX {
                        // Increment the centuries and decrease nanoseconds
                        as_duration.centuries += 1;
                        as_duration.nanoseconds -= NANOSECONDS_PER_CENTURY;
                        // And repeat
                        me.centuries = me.centuries.saturating_mul(as_duration.centuries);
                        if me.centuries == i16::MAX {
                            return Self::Output::MAX;
                        } else if me.centuries == i16::MIN {
                            return Self::Output::MIN;
                        }
                        me.nanoseconds = me.nanoseconds.saturating_mul(as_duration.nanoseconds);
                    }
                }
                me
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
            let units = ["days", "h", "min", "s", "ms", "us", "ns"];
            let mut prev_ignored = true;
            for (val, unit) in values.iter().zip(units.iter()) {
                if *val > 0 {
                    if !prev_ignored {
                        // Add space
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", val, unit)?;
                    prev_ignored = false;
                } else {
                    prev_ignored = true;
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
        let mut me = self;
        me.centuries = me.centuries.saturating_add(rhs.centuries);
        if me.centuries == i16::MAX {
            return Duration::MAX;
        }
        me.nanoseconds = me.nanoseconds.saturating_add(rhs.nanoseconds);

        if me.nanoseconds == u64::MAX {
            // Increment the centuries and decrease nanoseconds
            me.centuries += 1;
            if me.centuries == i16::MAX {
                return Self::Output::MAX;
            }

            me.nanoseconds = me
                .nanoseconds
                .saturating_add(rhs.nanoseconds - NANOSECONDS_PER_CENTURY);
        }

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
        let mut me = self;

        me.centuries = me.centuries.saturating_sub(rhs.centuries);
        if me.centuries == i16::MIN {
            return Duration::MIN;
        }
        if rhs.nanoseconds == me.nanoseconds {
            // Special case to avoid getting confused with the saturating sub call
            me.nanoseconds = 0
        } else {
            me.nanoseconds = me.nanoseconds.saturating_sub(rhs.nanoseconds);

            if me.nanoseconds == 0 {
                // Oh, we might over underflowed
                me.centuries += 1;
                if me.centuries == i16::MAX {
                    return Self::Output::MAX;
                }

                me.nanoseconds = me
                    .nanoseconds
                    .saturating_add(rhs.nanoseconds + NANOSECONDS_PER_CENTURY);
            }
        }
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
        Self {
            centuries: -self.centuries - 1,
            nanoseconds: NANOSECONDS_PER_CENTURY - self.nanoseconds,
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

impl_ops_for_type!(f32);
impl_ops_for_type!(f64);
// impl_ops_for_type!(u8);
// impl_ops_for_type!(i8);
// impl_ops_for_type!(u16);
// impl_ops_for_type!(i16);
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
    dbg!(quarter_hour, third_hour, sum);
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
        "5 h 256 ms 1003.5 ns"
    );

    // Check printing negative durations only shows one negative sign
    assert_eq!(
        format!("{}", TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256),
        "-5 h 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * -3.5
        ),
        "-5 h 256 ms 3.5 ns"
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
    let delta =
        sum.in_unit(TimeUnit::Millisecond).floor() - sum.in_unit(TimeUnit::Second).floor() * 1000.0;
    println!("{:?}", (delta * -1.0) == 0.0);
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

#[test]
fn test_i128_extremes() {
    let d = Duration::from_total_nanoseconds(i128::MAX);
    println!("{}", d);
    assert_eq!(Duration::from_total_nanoseconds(d.total_nanoseconds()), d);
    let d = Duration::from_total_nanoseconds(i128::MIN + 1);
    println!("{}", d);
    // Test truncation
    let d_min = Duration::from_total_nanoseconds(i128::MIN);
    assert_eq!(d - d_min, 1 * TimeUnit::Nanosecond);
    assert_eq!(d_min - d, -1 * TimeUnit::Nanosecond);
    println!("{}", d - d_min);
    assert_eq!(Duration::from_total_nanoseconds(d.total_nanoseconds()), d);
    println!("{}", Duration::MAX);
}
