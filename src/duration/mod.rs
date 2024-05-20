/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

use crate::errors::{DurationError, EpochError};
use crate::{SECONDS_PER_CENTURY, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};

pub use crate::{Freq, Frequencies, TimeUnits, Unit};

#[cfg(feature = "std")]
mod std;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "serde")]
use core::str::FromStr;

#[cfg(not(kani))]
pub mod parse;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
use pyo3::prelude::pyclass;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

#[cfg(kani)]
mod kani_verif;

pub const DAYS_PER_CENTURY_U64: u64 = 36_525;
pub const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;
pub const NANOSECONDS_PER_MILLISECOND: u64 = 1_000 * NANOSECONDS_PER_MICROSECOND;
pub const NANOSECONDS_PER_SECOND: u64 = 1_000 * NANOSECONDS_PER_MILLISECOND;
pub(crate) const NANOSECONDS_PER_SECOND_U32: u32 = 1_000_000_000;
pub const NANOSECONDS_PER_MINUTE: u64 = 60 * NANOSECONDS_PER_SECOND;
pub const NANOSECONDS_PER_HOUR: u64 = 60 * NANOSECONDS_PER_MINUTE;
pub const NANOSECONDS_PER_DAY: u64 = 24 * NANOSECONDS_PER_HOUR;
pub const NANOSECONDS_PER_CENTURY: u64 = DAYS_PER_CENTURY_U64 * NANOSECONDS_PER_DAY;

pub mod ops;

/// Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.
///
/// **Important conventions:**
/// 1. The negative durations can be mentally modeled "BC" years. One hours before 01 Jan 0000, it was "-1" years but  365 days and 23h into the current day.
/// It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
/// a duration with centuries = -1 and nanoseconds = 0 is _a greater duration_ (further from zero) than centuries = -1 and nanoseconds = 1.
/// Duration zero minus one nanosecond returns a century of -1 and a nanosecond set to the number of nanoseconds in one century minus one.
/// That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
/// As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
/// 2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.
#[derive(Clone, Copy, Debug, PartialOrd, Eq, Ord)]
#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "hifitime"))]
pub struct Duration {
    pub(crate) centuries: i16,
    pub(crate) nanoseconds: u64,
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        if self.centuries == other.centuries {
            self.nanoseconds == other.nanoseconds
        } else if (self.centuries.saturating_sub(other.centuries)).saturating_abs() == 1
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

impl Hash for Duration {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.centuries.hash(hasher);
        self.nanoseconds.hash(hasher);
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration::ZERO
    }
}

#[cfg(not(kani))]
#[cfg(feature = "serde")]
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

#[cfg(not(kani))]
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Duration::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// Defines the methods that should be classmethods in Python, but must be redefined as per https://github.com/PyO3/pyo3/issues/1003#issuecomment-844433346
impl Duration {
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
        nanoseconds: 0,
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

    #[must_use]
    /// Create a new duration from the truncated nanoseconds (+/- 2927.1 years of duration)
    pub fn from_truncated_nanoseconds(nanos: i64) -> Self {
        if nanos < 0 {
            let ns = nanos.unsigned_abs();
            // Note: i64::MIN corresponds to a duration just past -3 centuries, so we can't hit the Duration::MIN here.
            let extra_centuries = ns.div_euclid(NANOSECONDS_PER_CENTURY);
            let rem_nanos = ns.rem_euclid(NANOSECONDS_PER_CENTURY);
            Self::from_parts(
                -1 - (extra_centuries as i16),
                NANOSECONDS_PER_CENTURY - rem_nanos,
            )
        } else {
            Self::from_parts(0, nanos.unsigned_abs())
        }
    }

    /// Creates a new duration from the provided number of days
    #[must_use]
    pub fn from_days(value: f64) -> Self {
        value * Unit::Day
    }

    /// Creates a new duration from the provided number of hours
    #[must_use]
    pub fn from_hours(value: f64) -> Self {
        value * Unit::Hour
    }

    /// Creates a new duration from the provided number of seconds
    #[must_use]
    pub fn from_seconds(value: f64) -> Self {
        value * Unit::Second
    }

    /// Creates a new duration from the provided number of milliseconds
    #[must_use]
    pub fn from_milliseconds(value: f64) -> Self {
        value * Unit::Millisecond
    }

    /// Creates a new duration from the provided number of microsecond
    #[must_use]
    pub fn from_microseconds(value: f64) -> Self {
        value * Unit::Microsecond
    }

    /// Creates a new duration from the provided number of nanoseconds
    #[must_use]
    pub fn from_nanoseconds(value: f64) -> Self {
        value * Unit::Nanosecond
    }

    /// Creates a new duration from its parts. Set the sign to a negative number for the duration to be negative.
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
        Self::compose_f64(
            sign,
            days as f64,
            hours as f64,
            minutes as f64,
            seconds as f64,
            milliseconds as f64,
            microseconds as f64,
            nanoseconds as f64,
        )
    }

    /// Creates a new duration from its parts. Set the sign to a negative number for the duration to be negative.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn compose_f64(
        sign: i8,
        days: f64,
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> Self {
        let me: Self = days.days()
            + hours.hours()
            + minutes.minutes()
            + seconds.seconds()
            + milliseconds.milliseconds()
            + microseconds.microseconds()
            + nanoseconds.nanoseconds();
        if sign < 0 {
            -me
        } else {
            me
        }
    }

    /// Initializes a Duration from a timezone offset
    #[must_use]
    pub fn from_tz_offset(sign: i8, hours: i64, minutes: i64) -> Self {
        let dur = hours * Unit::Hour + minutes * Unit::Minute;
        if sign < 0 {
            -dur
        } else {
            dur
        }
    }
}

impl Duration {
    fn normalize(&mut self) {
        let extra_centuries = self.nanoseconds.div_euclid(NANOSECONDS_PER_CENTURY);
        // We can skip this whole step if the div_euclid shows that we didn't overflow the number of nanoseconds per century
        if extra_centuries > 0 {
            let rem_nanos = self.nanoseconds.rem_euclid(NANOSECONDS_PER_CENTURY);

            if self.centuries == i16::MAX {
                if self.nanoseconds.saturating_add(rem_nanos) > Self::MAX.nanoseconds {
                    // Saturated max
                    *self = Self::MAX;
                }
                // Else, we're near the MAX but we're within the MAX in nanoseconds, so let's not do anything here.
            } else if *self != Self::MAX && *self != Self::MIN {
                // The bounds are valid as is, no wrapping needed when rem_nanos is not zero.
                match self.centuries.checked_add(extra_centuries as i16) {
                    Some(centuries) => {
                        self.centuries = centuries;
                        self.nanoseconds = rem_nanos;
                    }
                    None => {
                        if self.centuries >= 0 {
                            // Saturated max again
                            *self = Self::MAX;
                        } else {
                            // Saturated min
                            *self = Self::MIN;
                        }
                    }
                }
            }
        }
    }

    #[must_use]
    /// Returns the centuries and nanoseconds of this duration
    /// NOTE: These items are not public to prevent incorrect durations from being created by modifying the values of the structure directly.
    pub const fn to_parts(&self) -> (i16, u64) {
        (self.centuries, self.nanoseconds)
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
            i128::from(self.centuries) * i128::from(NANOSECONDS_PER_CENTURY)
                - i128::from(self.nanoseconds)
        }
    }

    /// Returns the truncated nanoseconds in a signed 64 bit integer, if the duration fits.
    pub fn try_truncated_nanoseconds(&self) -> Result<i64, EpochError> {
        // If it fits, we know that the nanoseconds also fit. abs() will fail if the centuries are min'ed out.
        if self.centuries == i16::MIN || self.centuries.abs() >= 3 {
            Err(EpochError::Duration {
                source: DurationError::Underflow,
            })
        } else if self.centuries == -1 {
            Ok(-((NANOSECONDS_PER_CENTURY - self.nanoseconds) as i64))
        } else if self.centuries >= 0 {
            match i64::from(self.centuries).checked_mul(NANOSECONDS_PER_CENTURY as i64) {
                Some(centuries_as_ns) => {
                    match centuries_as_ns.checked_add(self.nanoseconds as i64) {
                        Some(truncated_ns) => Ok(truncated_ns),
                        None => Err(EpochError::Duration {
                            source: DurationError::Overflow,
                        }),
                    }
                }
                None => Err(EpochError::Duration {
                    source: DurationError::Underflow,
                }),
            }
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

    /// Returns this duration in seconds f64.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    #[must_use]
    pub fn to_seconds(&self) -> f64 {
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

    #[must_use]
    pub fn to_unit(&self, unit: Unit) -> f64 {
        self.to_seconds() * unit.from_seconds()
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

    /// Returns the sign of this duration
    /// + 0 if the number is zero
    /// + 1 if the number is positive
    /// + -1 if the number is negative
    #[must_use]
    pub const fn signum(&self) -> i8 {
        self.centuries.signum() as i8
    }

    /// Decomposes a Duration in its sign, days, hours, minutes, seconds, ms, us, ns
    #[must_use]
    pub fn decompose(&self) -> (i8, u64, u64, u64, u64, u64, u64, u64) {
        let mut me = *self;
        let sign = me.signum();
        me = me.abs();
        let days = me.to_unit(Unit::Day).floor();
        me -= days.days();
        let hours = me.to_unit(Unit::Hour).floor();
        me -= hours.hours();
        let minutes = me.to_unit(Unit::Minute).floor();
        me -= minutes.minutes();
        let seconds = me.to_unit(Unit::Second).floor();
        me -= seconds.seconds();
        let milliseconds = me.to_unit(Unit::Millisecond).floor();
        me -= milliseconds.milliseconds();
        let microseconds = me.to_unit(Unit::Microsecond).floor();
        me -= microseconds.microseconds();
        let nanoseconds = me.to_unit(Unit::Nanosecond).round();

        // Everything should fit in the expected types now
        (
            sign,
            days as u64,
            hours as u64,
            minutes as u64,
            seconds as u64,
            milliseconds as u64,
            microseconds as u64,
            nanoseconds as u64,
        )
    }

    /// Returns the subdivision of duration in this unit, if such is available. Does not work with Week or Century.
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits, Unit};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.subdivision(Unit::Hour), Some(2.hours()));
    /// assert_eq!(two_hours_three_min.subdivision(Unit::Minute), Some(3.minutes()));
    /// assert_eq!(two_hours_three_min.subdivision(Unit::Second), Some(Duration::ZERO));
    /// assert_eq!(two_hours_three_min.subdivision(Unit::Week), None);
    /// ```
    #[must_use]
    pub fn subdivision(&self, unit: Unit) -> Option<Duration> {
        let (_, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) =
            self.decompose();

        match unit {
            Unit::Nanosecond => Some((nanoseconds as i64) * unit),
            Unit::Microsecond => Some((microseconds as i64) * unit),
            Unit::Millisecond => Some((milliseconds as i64) * unit),
            Unit::Second => Some((seconds as i64) * unit),
            Unit::Minute => Some((minutes as i64) * unit),
            Unit::Hour => Some((hours as i64) * unit),
            Unit::Day => Some((days as i64) * unit),
            Unit::Week | Unit::Century => None,
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

    /// Rounds this duration to the largest units represented in this duration.
    ///
    /// This is useful to provide an approximate human duration. Under the hood, this function uses `round`,
    /// so the "tipping point" of the rounding is half way to the next increment of the greatest unit.
    /// As shown below, one example is that 35 hours and 59 minutes rounds to 1 day, but 36 hours and 1 minute rounds
    /// to 2 days because 2 days is closer to 36h 1 min than 36h 1 min is to 1 day.
    ///
    /// # Example
    ///
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// assert_eq!((2.hours() + 3.minutes()).approx(), 2.hours());
    /// assert_eq!((24.hours() + 3.minutes()).approx(), 1.days());
    /// assert_eq!((35.hours() + 59.minutes()).approx(), 1.days());
    /// assert_eq!((36.hours() + 1.minutes()).approx(), 2.days());
    /// assert_eq!((47.hours() + 3.minutes()).approx(), 2.days());
    /// assert_eq!((49.hours() + 3.minutes()).approx(), 2.days());
    /// ```
    pub fn approx(&self) -> Self {
        let (_, days, hours, minutes, seconds, milli, us, _) = self.decompose();

        let round_to = if days > 0 {
            1 * Unit::Day
        } else if hours > 0 {
            1 * Unit::Hour
        } else if minutes > 0 {
            1 * Unit::Minute
        } else if seconds > 0 {
            1 * Unit::Second
        } else if milli > 0 {
            1 * Unit::Millisecond
        } else if us > 0 {
            1 * Unit::Microsecond
        } else {
            1 * Unit::Nanosecond
        };

        self.round(round_to)
    }

    // Returns the minimum of the two durations.
    ///
    /// ```
    /// use hifitime::TimeUnits;
    ///
    /// let d0 = 20.seconds();
    /// let d1 = 21.seconds();
    ///
    /// assert_eq!(d0, d1.min(d0));
    /// assert_eq!(d0, d0.min(d1));
    /// ```
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    /// Returns the maximum of the two durations.
    ///
    /// ```
    /// use hifitime::TimeUnits;
    ///
    /// let d0 = 20.seconds();
    /// let d1 = 21.seconds();
    ///
    /// assert_eq!(d1, d1.max(d0));
    /// assert_eq!(d1, d0.max(d1));
    /// ```
    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }

    /// Returns whether this is a negative or positive duration.
    pub const fn is_negative(&self) -> bool {
        self.centuries.is_negative()
    }
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
            let units = [
                if days > 1 { "days" } else { "day" },
                "h",
                "min",
                "s",
                "ms",
                "μs",
                "ns",
            ];

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
        let seconds_f64 = self.to_seconds();
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

#[cfg(test)]
mod ut_duration {
    use super::{Duration, TimeUnits, Unit, NANOSECONDS_PER_CENTURY};

    #[test]
    #[cfg(feature = "serde")]
    fn test_serdes() {
        for (dt, content) in [
            (Duration::from_seconds(10.1), r#""10 s 100 ms""#),
            (1.0_f64.days() + 99.nanoseconds(), r#""1 day 99 ns""#),
            (
                1.0_f64.centuries() + 99.seconds(),
                r#""36525 days 1 min 39 s""#,
            ),
        ] {
            assert_eq!(content, serde_json::to_string(&dt).unwrap());
            let parsed: Duration = serde_json::from_str(content).unwrap();
            assert_eq!(dt, parsed);
        }
    }

    #[test]
    fn test_bounds() {
        let min = Duration::MIN;
        assert_eq!(min.centuries, i16::MIN);
        assert_eq!(min.nanoseconds, 0);

        let max = Duration::MAX;
        assert_eq!(max.centuries, i16::MAX);
        assert_eq!(max.nanoseconds, NANOSECONDS_PER_CENTURY);

        let min_p = Duration::MIN_POSITIVE;
        assert_eq!(min_p.centuries, 0);
        assert_eq!(min_p.nanoseconds, 1);

        let min_n = Duration::MIN_NEGATIVE;
        assert_eq!(min_n.centuries, -1);
        assert_eq!(min_n.nanoseconds, NANOSECONDS_PER_CENTURY - 1);

        let min_n1 = Duration::MIN - 1 * Unit::Nanosecond;
        assert_eq!(min_n1, Duration::MIN);

        let max_n1 = Duration::MAX - 1 * Unit::Nanosecond;
        assert_eq!(max_n1.centuries, i16::MAX);
        assert_eq!(max_n1.nanoseconds, NANOSECONDS_PER_CENTURY - 1);
    }

    #[test]
    fn test_decompose() {
        let d = -73000.days();
        let out_days = d.to_unit(Unit::Day);
        assert_eq!(out_days, -73000.0);
        let (sign, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) =
            d.decompose();
        assert_eq!(sign, -1);
        assert_eq!(days, 73000);
        assert_eq!(hours, 0);
        assert_eq!(minutes, 0);
        assert_eq!(seconds, 0);
        assert_eq!(milliseconds, 0);
        assert_eq!(microseconds, 0);
        assert_eq!(nanoseconds, 0);
    }
}
