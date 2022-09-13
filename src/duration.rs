/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

#[cfg(feature = "std")]
use crate::ParsingErrors;
use crate::{
    Errors, DAYS_PER_CENTURY, SECONDS_PER_CENTURY, SECONDS_PER_DAY, SECONDS_PER_HOUR,
    SECONDS_PER_MINUTE,
};

#[cfg(feature = "std")]
extern crate core;
use core::cmp::Ordering;
use core::convert::TryInto;
use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[cfg(feature = "std")]
use super::regex::Regex;
#[cfg(feature = "std")]
use super::serde::{de, Deserialize, Deserializer};
#[cfg(feature = "std")]
use std::str::FromStr;

#[cfg(not(feature = "std"))]
use num_traits::Float;

pub const DAYS_PER_CENTURY_U64: u64 = 36_525;
pub const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;
pub const NANOSECONDS_PER_MILLISECOND: u64 = 1_000 * NANOSECONDS_PER_MICROSECOND;
pub const NANOSECONDS_PER_SECOND: u64 = 1_000 * NANOSECONDS_PER_MILLISECOND;
pub const NANOSECONDS_PER_MINUTE: u64 = 60 * NANOSECONDS_PER_SECOND;
pub const NANOSECONDS_PER_HOUR: u64 = 60 * NANOSECONDS_PER_MINUTE;
pub const NANOSECONDS_PER_DAY: u64 = 24 * NANOSECONDS_PER_HOUR;
pub const NANOSECONDS_PER_CENTURY: u64 = DAYS_PER_CENTURY_U64 * NANOSECONDS_PER_DAY;

/// Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.
///
/// **Important conventions:**
/// Conventions had to be made to define the partial order of a duration.
/// 1. It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
/// a durationn with centuries = -1 and nanoseconds = 0 is _a smaller duration_ than centuries = -1 and nanoseconds = 1.
/// That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
/// As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
/// 2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.
#[derive(Clone, Copy, Debug, PartialOrd, Eq, Ord)]
#[repr(C)]
pub struct Duration {
    pub(crate) centuries: i16,
    pub(crate) nanoseconds: u64,
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

impl Default for Duration {
    fn default() -> Self {
        Duration::ZERO
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
                    self.centuries = self.centuries.checked_add(extra_centuries as i16).unwrap();
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
                    self.centuries = self.centuries.checked_add(extra_centuries as i16).unwrap();
                    self.nanoseconds = rem_nanos;
                } else {
                    // Saturated max again
                    *self = Self::MIN;
                }
            }
        }
    }

    #[must_use]
    /// Create a normalized duration from its parts
    pub fn from_parts(centuries: i16, nanoseconds: u64) -> Self {
        let mut me = Self {
            centuries,
            nanoseconds,
        };
        me.normalize();
        me
    }

    #[must_use]
    /// Returns the centuries and nanoseconds of this duration
    /// NOTE: These items are not public to prevent incorrect durations from being created by modifying the values of the structure directly.
    pub const fn to_parts(&self) -> (i16, u64) {
        (self.centuries, self.nanoseconds)
    }

    #[must_use]
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
    pub fn total_nanoseconds(&self) -> i128 {
        if self.centuries == -1 {
            -i128::from(NANOSECONDS_PER_CENTURY - self.nanoseconds)
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
    pub fn try_truncated_nanoseconds(&self) -> Result<i64, Errors> {
        // If it fits, we know that the nanoseconds also fit
        if self.centuries.abs() >= 3 {
            Err(Errors::Overflow)
        } else if self.centuries == -1 {
            Ok(-((NANOSECONDS_PER_CENTURY - self.nanoseconds) as i64))
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
    pub fn truncated_nanoseconds(&self) -> i64 {
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

    #[must_use]
    /// Create a new duration from the truncated nanoseconds (+/- 2927.1 years of duration)
    pub fn from_truncated_nanoseconds(nanos: i64) -> Self {
        if nanos < 0 {
            let ns = nanos.unsigned_abs();
            let extra_centuries = ns.div_euclid(NANOSECONDS_PER_CENTURY);
            if extra_centuries > i16::MAX as u64 {
                Self::MIN
            } else {
                let rem_nanos = ns.rem_euclid(NANOSECONDS_PER_CENTURY);
                Self::from_parts(
                    -1 - (extra_centuries as i16),
                    NANOSECONDS_PER_CENTURY - rem_nanos,
                )
            }
        } else {
            Self::from_parts(0, nanos.unsigned_abs())
        }
    }

    /// Creates a new duration from the provided unit
    #[must_use]
    pub fn from_f64(value: f64, unit: Unit) -> Self {
        unit * value
    }

    /// Returns this duration in seconds f64.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    #[must_use]
    pub fn in_seconds(&self) -> f64 {
        // Compute the seconds and nanoseconds that we know this fits on a 64bit float
        let seconds = self.nanoseconds.div_euclid(NANOSECONDS_PER_SECOND);
        let subseconds = self.nanoseconds.rem_euclid(NANOSECONDS_PER_SECOND);
        if self.centuries == 0 {
            (seconds as f64) + (subseconds as f64) * 1e-9
        } else {
            f64::from(self.centuries) * SECONDS_PER_CENTURY
                + (seconds as f64)
                + (subseconds as f64) * 1e-9
        }
    }

    /// Returns the value of this duration in the requested unit.
    #[must_use]
    pub fn in_unit(&self, unit: Unit) -> f64 {
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

    /// Returns the sign of this duration
    #[must_use]
    pub const fn signum(&self) -> i8 {
        self.centuries.signum() as i8
    }

    /// Decomposes a Duration in its sign, days, hours, minutes, seconds, ms, us, ns
    #[must_use]
    pub fn decompose(&self) -> (i8, u64, u64, u64, u64, u64, u64, u64) {
        let sign = self.signum();

        match self.try_truncated_nanoseconds() {
            Ok(total_ns) => {
                let ns_left = total_ns.abs();

                let (days, ns_left) = div_rem_i64(ns_left, NANOSECONDS_PER_DAY as i64);
                let (hours, ns_left) = div_rem_i64(ns_left, NANOSECONDS_PER_HOUR as i64);
                let (minutes, ns_left) = div_rem_i64(ns_left, NANOSECONDS_PER_MINUTE as i64);
                let (seconds, ns_left) = div_rem_i64(ns_left, NANOSECONDS_PER_SECOND as i64);
                let (milliseconds, ns_left) =
                    div_rem_i64(ns_left, NANOSECONDS_PER_MILLISECOND as i64);
                let (microseconds, ns_left) =
                    div_rem_i64(ns_left, NANOSECONDS_PER_MICROSECOND as i64);

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
            Err(_) => {
                // Doesn't fit on a i64, so let's use the slower i128
                let total_ns = self.total_nanoseconds();
                let ns_left = total_ns.abs();

                let (days, ns_left) = div_rem_i128(ns_left, i128::from(NANOSECONDS_PER_DAY));
                let (hours, ns_left) = div_rem_i128(ns_left, i128::from(NANOSECONDS_PER_HOUR));
                let (minutes, ns_left) = div_rem_i128(ns_left, i128::from(NANOSECONDS_PER_MINUTE));
                let (seconds, ns_left) = div_rem_i128(ns_left, i128::from(NANOSECONDS_PER_SECOND));
                let (milliseconds, ns_left) =
                    div_rem_i128(ns_left, i128::from(NANOSECONDS_PER_MILLISECOND));
                let (microseconds, ns_left) =
                    div_rem_i128(ns_left, i128::from(NANOSECONDS_PER_MICROSECOND));

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
        }
    }

    /// Creates a new duration from its parts
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn compose(
        sign: i8,
        days: u64,
        hours: u64,
        minutes: u64,
        seconds: u64,
        milliseconds: u64,
        microseconds: u64,
        nanoseconds: u64,
    ) -> Self {
        let me: Self = (days as i64).days()
            + (hours as i64).hours()
            + (minutes as i64).minutes()
            + (seconds as i64).seconds()
            + (milliseconds as i64).seconds()
            + (microseconds as i64).microseconds()
            + (nanoseconds as i64).nanoseconds();
        if sign == -1 {
            -me
        } else {
            me
        }
    }

    /// Floors this duration to the closest duration from the bottom
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.floor(1.hours()), 2.hours());
    /// assert_eq!(two_hours_three_min.floor(30.minutes()), 2.hours());
    /// // This is zero because we floor by a duration longer than the current duration, rounding it down
    /// assert_eq!(two_hours_three_min.floor(4.hours()), 0.hours());
    /// assert_eq!(two_hours_three_min.floor(1.seconds()), two_hours_three_min);
    /// assert_eq!(two_hours_three_min.floor(1.hours() + 1.minutes()), 2.hours() + 2.minutes());
    /// assert_eq!(two_hours_three_min.floor(1.hours() + 5.minutes()), 1.hours() + 5.minutes());
    /// ```
    pub fn floor(&self, duration: Self) -> Self {
        // Note that we don't use checked_sub because, at most, this will be zero.
        Self::from_total_nanoseconds(
            self.total_nanoseconds() - self.total_nanoseconds() % duration.total_nanoseconds(),
        )
    }

    /// Ceils this duration to the closest provided duration
    ///
    /// This simply floors then adds the requested duration
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.ceil(1.hours()), 3.hours());
    /// assert_eq!(two_hours_three_min.ceil(30.minutes()), 2.hours() + 30.minutes());
    /// assert_eq!(two_hours_three_min.ceil(4.hours()), 4.hours());
    /// assert_eq!(two_hours_three_min.ceil(1.seconds()), two_hours_three_min + 1.seconds());
    /// assert_eq!(two_hours_three_min.ceil(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
    /// ```
    pub fn ceil(&self, duration: Self) -> Self {
        let floored = self.floor(duration);
        match floored
            .total_nanoseconds()
            .checked_add(duration.abs().total_nanoseconds())
        {
            Some(total_ns) => Self::from_total_nanoseconds(total_ns),
            None => Self::MAX,
        }
    }

    /// Rounds this duration to the closest provided duration
    ///
    /// This performs both a `ceil` and `floor` and returns the value which is the closest to current one.
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.round(1.hours()), 2.hours());
    /// assert_eq!(two_hours_three_min.round(30.minutes()), 2.hours());
    /// assert_eq!(two_hours_three_min.round(4.hours()), 4.hours());
    /// assert_eq!(two_hours_three_min.round(1.seconds()), two_hours_three_min);
    /// assert_eq!(two_hours_three_min.round(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
    /// ```
    pub fn round(&self, duration: Self) -> Self {
        let floored = self.floor(duration);
        let ceiled = self.ceil(duration);
        if *self - floored < (ceiled - *self).abs() {
            floored
        } else {
            ceiled
        }
    }

    /// A duration of exactly zero nanoseconds
    pub const ZERO: Self = Self {
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

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Mul<i64> for Duration {
    type Output = Duration;
    fn mul(self, q: i64) -> Self::Output {
        Duration::from_total_nanoseconds(
            self.total_nanoseconds()
                .saturating_mul((q * Unit::Nanosecond).total_nanoseconds()),
        )
    }
}

impl Mul<f64> for Duration {
    type Output = Duration;
    fn mul(self, q: f64) -> Self::Output {
        // Make sure that we don't trim the number by finding its precision
        let mut p: i32 = 0;
        let mut new_val = q;
        let ten: f64 = 10.0;

        loop {
            if (new_val.floor() - new_val).abs() < f64::EPSILON {
                // Yay, we've found the precision of this number
                break;
            }
            // Multiply by the precision
            // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=b760579f103b7192c20413ebbe167b90
            p += 1;
            new_val = q * ten.powi(p);
        }

        Duration::from_total_nanoseconds(
            self.total_nanoseconds()
                .saturating_mul(new_val as i128)
                .saturating_div(10_i128.pow(p.try_into().unwrap())),
        )
    }
}

impl Mul<i64> for Unit {
    type Output = Duration;

    /// Converts the input values to i128 and creates a duration from that
    /// This method will necessarily ignore durations below nanoseconds
    fn mul(self, q: i64) -> Duration {
        let total_ns = match self {
            Unit::Century => q * (NANOSECONDS_PER_CENTURY as i64),
            Unit::Day => q * (NANOSECONDS_PER_DAY as i64),
            Unit::Hour => q * (NANOSECONDS_PER_HOUR as i64),
            Unit::Minute => q * (NANOSECONDS_PER_MINUTE as i64),
            Unit::Second => q * (NANOSECONDS_PER_SECOND as i64),
            Unit::Millisecond => q * (NANOSECONDS_PER_MILLISECOND as i64),
            Unit::Microsecond => q * (NANOSECONDS_PER_MICROSECOND as i64),
            Unit::Nanosecond => q,
        };
        if total_ns.abs() < (i64::MAX as i64) {
            Duration::from_truncated_nanoseconds(total_ns as i64)
        } else {
            Duration::from_total_nanoseconds(total_ns as i128)
        }
    }
}

impl Mul<f64> for Unit {
    type Output = Duration;

    /// Converts the input values to i128 and creates a duration from that
    /// This method will necessarily ignore durations below nanoseconds
    fn mul(self, q: f64) -> Duration {
        let total_ns = match self {
            Unit::Century => q * (NANOSECONDS_PER_CENTURY as f64),
            Unit::Day => q * (NANOSECONDS_PER_DAY as f64),
            Unit::Hour => q * (NANOSECONDS_PER_HOUR as f64),
            Unit::Minute => q * (NANOSECONDS_PER_MINUTE as f64),
            Unit::Second => q * (NANOSECONDS_PER_SECOND as f64),
            Unit::Millisecond => q * (NANOSECONDS_PER_MILLISECOND as f64),
            Unit::Microsecond => q * (NANOSECONDS_PER_MICROSECOND as f64),
            Unit::Nanosecond => q,
        };
        if total_ns.abs() < (i64::MAX as f64) {
            Duration::from_truncated_nanoseconds(total_ns as i64)
        } else {
            Duration::from_total_nanoseconds(total_ns as i128)
        }
    }
}

macro_rules! impl_ops_for_type {
    ($type:ident) => {
        impl Mul<Unit> for $type {
            type Output = Duration;
            fn mul(self, q: Unit) -> Duration {
                // Apply the reflexive property
                q * self
            }
        }

        impl Mul<$type> for Freq {
            type Output = Duration;

            /// Converts the input values to i128 and creates a duration from that
            /// This method will necessarily ignore durations below nanoseconds
            fn mul(self, q: $type) -> Duration {
                let total_ns = match self {
                    Freq::GigaHertz => 1.0 / (q as f64),
                    Freq::MegaHertz => (NANOSECONDS_PER_MICROSECOND as f64) / (q as f64),
                    Freq::KiloHertz => NANOSECONDS_PER_MILLISECOND as f64 / (q as f64),
                    Freq::Hertz => (NANOSECONDS_PER_SECOND as f64) / (q as f64),
                };
                if total_ns.abs() < (i64::MAX as f64) {
                    Duration::from_truncated_nanoseconds(total_ns as i64)
                } else {
                    Duration::from_total_nanoseconds(total_ns as i128)
                }
            }
        }

        impl Mul<Freq> for $type {
            type Output = Duration;
            fn mul(self, q: Freq) -> Duration {
                // Apply the reflexive property
                q * self
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Self::Output {
                Duration::from_total_nanoseconds(
                    self.total_nanoseconds()
                        .saturating_div((q * Unit::Nanosecond).total_nanoseconds()),
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

        impl TimeUnits for $type {}

        impl Frequencies for $type {}
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

// Allow adding with a Unit directly
impl Add<Unit> for Duration {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: Unit) -> Duration {
        self + rhs * 1
    }
}

impl AddAssign<Unit> for Duration {
    #[allow(clippy::identity_op)]
    fn add_assign(&mut self, rhs: Unit) {
        *self = *self + rhs * 1;
    }
}

impl Sub<Unit> for Duration {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn sub(self, rhs: Unit) -> Duration {
        self - rhs * 1
    }
}

impl SubAssign<Unit> for Duration {
    #[allow(clippy::identity_op)]
    fn sub_assign(&mut self, rhs: Unit) {
        *self = *self - rhs * 1;
    }
}

impl PartialEq<Unit> for Duration {
    #[allow(clippy::identity_op)]
    fn eq(&self, unit: &Unit) -> bool {
        *self == *unit * 1
    }
}

impl PartialOrd<Unit> for Duration {
    #[allow(clippy::identity_op, clippy::comparison_chain)]
    fn partial_cmp(&self, unit: &Unit) -> Option<Ordering> {
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

#[cfg(feature = "std")]
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
    /// use hifitime::{Duration, Unit};
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Duration::from_str("1 d").unwrap(), Unit::Day * 1);
    /// assert_eq!(Duration::from_str("10.598 days").unwrap(), Unit::Day * 10.598);
    /// assert_eq!(Duration::from_str("10.598 min").unwrap(), Unit::Minute * 10.598);
    /// assert_eq!(Duration::from_str("10.598 us").unwrap(), Unit::Microsecond * 10.598);
    /// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), Unit::Second * 10.598);
    /// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), Unit::Nanosecond * 10.598);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg = Regex::new(r"^(\d+\.?\d*)\W*(\w+)$").unwrap();
        match reg.captures(s) {
            Some(cap) => {
                let value = cap[1].to_owned().parse::<f64>().unwrap();
                match cap[2].to_owned().to_lowercase().as_str() {
                    "d" | "days" | "day" => Ok(Unit::Day * value),
                    "h" | "hours" | "hour" => Ok(Unit::Hour * value),
                    "min" | "mins" | "minute" | "minutes" => Ok(Unit::Minute * value),
                    "s" | "second" | "seconds" => Ok(Unit::Second * value),
                    "ms" | "millisecond" | "milliseconds" => Ok(Unit::Millisecond * value),
                    "us" | "microsecond" | "microseconds" => Ok(Unit::Microsecond * value),
                    "ns" | "nanosecond" | "nanoseconds" => Ok(Unit::Nanosecond * value),
                    _ => Err(Errors::ParseError(ParsingErrors::UnknownUnit)),
                }
            }
            None => Err(Errors::ParseError(ParsingErrors::UnknownFormat)),
        }
    }
}

/// A trait to automatically convert some primitives to a duration
///
/// ```
/// #[cfg(feature = "std")]
/// {
/// use hifitime::prelude::*;
/// use std::str::FromStr;
///
/// assert_eq!(Duration::from_str("1 d").unwrap(), 1.days());
/// assert_eq!(Duration::from_str("10.598 days").unwrap(), 10.598.days());
/// assert_eq!(Duration::from_str("10.598 min").unwrap(), 10.598.minutes());
/// assert_eq!(Duration::from_str("10.598 us").unwrap(), 10.598.microseconds());
/// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), 10.598.seconds());
/// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), 10.598.nanoseconds());
/// }
/// ```
pub trait TimeUnits: Copy + Mul<Unit, Output = Duration> {
    fn centuries(self) -> Duration {
        self * Unit::Century
    }
    fn days(self) -> Duration {
        self * Unit::Day
    }
    fn hours(self) -> Duration {
        self * Unit::Hour
    }
    fn minutes(self) -> Duration {
        self * Unit::Minute
    }
    fn seconds(self) -> Duration {
        self * Unit::Second
    }
    fn milliseconds(self) -> Duration {
        self * Unit::Millisecond
    }
    fn microseconds(self) -> Duration {
        self * Unit::Microsecond
    }
    fn nanoseconds(self) -> Duration {
        self * Unit::Nanosecond
    }
}

/// A trait to automatically convert some primitives to an approximate frequency as a duration, **rounded to the closest nanosecond**
/// Does not support more than 1 GHz (because max precision of a duration is 1 nanosecond)
///
/// ```
/// use hifitime::prelude::*;
/// use std::str::FromStr;
///
/// assert_eq!(1.Hz(), 1.seconds());
/// assert_eq!(10.Hz(), 0.1.seconds());
/// assert_eq!(100.Hz(), 0.01.seconds());
/// assert_eq!(1.MHz(), 1.microseconds());
/// assert_eq!(250.MHz(), 4.nanoseconds());
/// assert_eq!(1.GHz(), 1.nanoseconds());
/// // LIMITATIONS
/// assert_eq!(240.MHz(), 4.nanoseconds()); // 240 MHz is actually 4.1666.. nanoseconds, not 4 exactly!
/// assert_eq!(10.GHz(), 0.nanoseconds()); // NOTE: anything greater than 1 GHz is NOT supported
/// ```
#[allow(non_snake_case)]
pub trait Frequencies: Copy + Mul<Freq, Output = Duration> {
    fn GHz(self) -> Duration {
        self * Freq::GigaHertz
    }
    fn MHz(self) -> Duration {
        self * Freq::MegaHertz
    }
    fn kHz(self) -> Duration {
        self * Freq::KiloHertz
    }
    fn Hz(self) -> Duration {
        self * Freq::Hertz
    }
}

/// An Enum to convert frequencies to their approximate duration, **rounded to the closest nanosecond**.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Freq {
    GigaHertz,
    MegaHertz,
    KiloHertz,
    Hertz,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    /// 36525 days, it the number of days per century in the Julian calendar
    Century,
}

impl Add for Unit {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: Self) -> Duration {
        self * 1 + rhs * 1
    }
}

impl Sub for Unit {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn sub(self, rhs: Self) -> Duration {
        self * 1 - rhs * 1
    }
}

impl Unit {
    #[must_use]
    pub fn in_seconds(&self) -> f64 {
        match self {
            Unit::Century => DAYS_PER_CENTURY * SECONDS_PER_DAY,
            Unit::Day => SECONDS_PER_DAY,
            Unit::Hour => SECONDS_PER_HOUR,
            Unit::Minute => SECONDS_PER_MINUTE,
            Unit::Second => 1.0,
            Unit::Millisecond => 1e-3,
            Unit::Microsecond => 1e-6,
            Unit::Nanosecond => 1e-9,
        }
    }

    #[must_use]
    pub fn from_seconds(&self) -> f64 {
        1.0 / self.in_seconds()
    }
}

impl_ops_for_type!(f64);
impl_ops_for_type!(i64);

const fn div_rem_i128(me: i128, rhs: i128) -> (i128, i128) {
    (me.div_euclid(rhs), me.rem_euclid(rhs))
}

const fn div_rem_i64(me: i64, rhs: i64) -> (i64, i64) {
    (me.div_euclid(rhs), me.rem_euclid(rhs))
}

#[cfg(feature = "std")]
#[test]
fn deser_test() {
    use serde_derive::Deserialize;
    #[derive(Deserialize)]
    struct _D {
        pub _d: Duration,
    }
}

#[cfg(kani)]
#[kani::proof]
fn formal_normalize_min() {
    // Test that a normalization from the min does not fail
    let centuries = i16::MIN;
    let nanoseconds: u64 = kani::any();
    let _dur = Duration::from_parts(centuries, nanoseconds);
}

#[cfg(kani)]
#[kani::proof]
fn formal_normalize_max() {
    // Test that a normalization from the min does not fail
    let centuries = i16::MAX;
    let nanoseconds: u64 = kani::any();
    let _dur = Duration::from_parts(centuries, nanoseconds);
}

#[cfg(kani)]
#[kani::proof]
fn formal_normalize_any() {
    // Test that a normalization from the min does not fail
    let centuries: i16 = kani::any();
    let nanoseconds: u64 = kani::any();
    let _dur = Duration::from_parts(centuries, nanoseconds);
}
