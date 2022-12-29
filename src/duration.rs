/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::ParsingErrors;
use crate::{Errors, SECONDS_PER_CENTURY, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};

pub use crate::{Freq, Frequencies, TimeUnits, Unit};

#[cfg(feature = "std")]
extern crate core;
use core::cmp::Ordering;
use core::convert::TryInto;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

use core::str::FromStr;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

#[cfg(not(feature = "std"))]
use num_traits::Float;

#[cfg(kani)]
use kani::Arbitrary;

pub const DAYS_PER_CENTURY_U64: u64 = 36_525;
pub const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;
pub const NANOSECONDS_PER_MILLISECOND: u64 = 1_000 * NANOSECONDS_PER_MICROSECOND;
pub const NANOSECONDS_PER_SECOND: u64 = 1_000 * NANOSECONDS_PER_MILLISECOND;
pub(crate) const NANOSECONDS_PER_SECOND_U32: u32 = 1_000_000_000;
pub const NANOSECONDS_PER_MINUTE: u64 = 60 * NANOSECONDS_PER_SECOND;
pub const NANOSECONDS_PER_HOUR: u64 = 60 * NANOSECONDS_PER_MINUTE;
pub const NANOSECONDS_PER_DAY: u64 = 24 * NANOSECONDS_PER_HOUR;
pub const NANOSECONDS_PER_CENTURY: u64 = DAYS_PER_CENTURY_U64 * NANOSECONDS_PER_DAY;

/// Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.
///
/// **Important conventions:**
/// 1. The negative durations can be mentally modeled "BC" years. One hours before 01 Jan 0000, it was "-1" years but  365 days and 23h into the current day.
/// It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
/// a duration with centuries = -1 and nanoseconds = 0 is _a smaller duration_ (further from zero) than centuries = -1 and nanoseconds = 1.
/// Duration zero minus one nanosecond returns a century of -1 and a nanosecond set to the number of nanoseconds in one century minus one.
/// That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
/// As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
/// 2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.
#[derive(Clone, Copy, Debug, PartialOrd, Eq, Ord)]
#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Duration {
    pub(crate) centuries: i16,
    pub(crate) nanoseconds: u64,
}

#[cfg(kani)]
impl Arbitrary for Duration {
    #[inline(always)]
    fn any() -> Self {
        let centuries: i16 = kani::any();
        let nanoseconds: u64 = kani::any();

        Duration::from_parts(centuries, nanoseconds)
    }
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

// Defines the methods that should be staticmethods in Python, but must be redefined as per https://github.com/PyO3/pyo3/issues/1003#issuecomment-844433346
impl Duration {
    /// Builds a new duration from the number of centuries and the number of nanoseconds
    #[must_use]
    #[deprecated(note = "Prefer from_parts()", since = "3.6.0")]
    pub fn new(centuries: i16, nanoseconds: u64) -> Self {
        let mut out = Self {
            centuries,
            nanoseconds,
        };
        out.normalize();
        out
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

    /// Creates a new duration from the provided unit
    #[must_use]
    pub fn from_f64(value: f64, unit: Unit) -> Self {
        unit * value
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

#[cfg_attr(feature = "python", pymethods)]
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
    pub fn try_truncated_nanoseconds(&self) -> Result<i64, Errors> {
        // If it fits, we know that the nanoseconds also fit. abs() will fail if the centuries are min'ed out.
        if self.centuries == i16::MIN || self.centuries.abs() >= 3 {
            Err(Errors::Overflow)
        } else if self.centuries == -1 {
            Ok(-((NANOSECONDS_PER_CENTURY - self.nanoseconds) as i64))
        } else if self.centuries >= 0 {
            match i64::from(self.centuries).checked_mul(NANOSECONDS_PER_CENTURY as i64) {
                Some(centuries_as_ns) => {
                    match centuries_as_ns.checked_add(self.nanoseconds as i64) {
                        Some(truncated_ns) => Ok(truncated_ns),
                        None => Err(Errors::Overflow),
                    }
                }
                None => Err(Errors::Overflow),
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
        // match self
        //     .total_nanoseconds()
        //     .checked_sub(self.total_nanoseconds() % duration.abs().total_nanoseconds())
        // {
        //     Some(total_ns) => Self::from_total_nanoseconds(total_ns),
        //     None => Self::MIN,
        // }

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

    /// Returns the minimum of the two durations.
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
    ///
    /// _Note:_ this uses a pointer to `self` which will be copied immediately because Python requires a pointer.
    pub fn min(&self, other: Self) -> Self {
        if *self < other {
            *self
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
    ///
    /// _Note:_ this uses a pointer to `self` which will be copied immediately because Python requires a pointer.
    pub fn max(&self, other: Self) -> Self {
        if *self > other {
            *self
        } else {
            other
        }
    }

    /// Returns whether this is a negative or positive duration.
    pub const fn is_negative(&self) -> bool {
        self.centuries.is_negative()
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

    // Python helpers

    #[cfg(feature = "python")]
    #[new]
    fn new_py(string_repr: String) -> PyResult<Self> {
        match Self::from_str(&string_repr) {
            Ok(d) => Ok(d),
            Err(e) => Err(PyErr::from(e)),
        }
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self}")
    }

    #[cfg(feature = "python")]
    fn __add__(&self, other: Self) -> Duration {
        *self + other
    }

    #[cfg(feature = "python")]
    fn __sub__(&self, other: Self) -> Duration {
        *self - other
    }

    #[cfg(feature = "python")]
    fn __mul__(&self, other: f64) -> Duration {
        *self * other
    }

    #[cfg(feature = "python")]
    fn __div__(&self, other: f64) -> Duration {
        *self / other
    }

    #[cfg(feature = "python")]
    fn __eq__(&self, other: Self) -> bool {
        *self == other
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Lt => *self < other,
            CompareOp::Le => *self <= other,
            CompareOp::Eq => *self == other,
            CompareOp::Ne => *self != other,
            CompareOp::Gt => *self > other,
            CompareOp::Ge => *self >= other,
        }
    }

    // Python constructors

    #[cfg(feature = "python")]
    #[staticmethod]
    fn zero() -> Duration {
        Duration::ZERO
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    fn epsilon() -> Duration {
        Duration::EPSILON
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    fn init_from_max() -> Duration {
        Duration::MAX
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    fn init_from_min() -> Duration {
        Duration::MIN
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    fn min_positive() -> Duration {
        Duration::MIN_POSITIVE
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    fn min_negative() -> Duration {
        Duration::MIN_NEGATIVE
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    /// Create a normalized duration from its parts
    fn init_from_parts(centuries: i16, nanoseconds: u64) -> Self {
        Self::from_parts(centuries, nanoseconds)
    }

    /// Creates a new duration from its parts
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "python")]
    #[staticmethod]
    #[must_use]
    fn init_from_all_parts(
        sign: i8,
        days: u64,
        hours: u64,
        minutes: u64,
        seconds: u64,
        milliseconds: u64,
        microseconds: u64,
        nanoseconds: u64,
    ) -> Self {
        Self::compose(
            sign,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    fn init_from_total_nanoseconds(nanos: i128) -> Self {
        Self::from_total_nanoseconds(nanos)
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    /// Create a new duration from the truncated nanoseconds (+/- 2927.1 years of duration)
    fn init_from_truncated_nanoseconds(nanos: i64) -> Self {
        Self::from_truncated_nanoseconds(nanos)
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

impl Add for Duration {
    type Output = Duration;

    /// # Addition of Durations
    /// Durations are centered on zero duration. Of the tuple, only the centuries may be negative, the nanoseconds are always positive
    /// and represent the nanoseconds _into_ the current centuries.
    ///
    /// ## Examples
    /// + `Duration { centuries: 0, nanoseconds: 1 }` is a positive duration of zero centuries and one nanosecond.
    /// + `Duration { centuries: -1, nanoseconds: 1 }` is a negative duration representing "one century before zero minus one nanosecond"
    fn add(self, rhs: Self) -> Duration {
        // Check that the addition fits in an i16
        let mut me = self;
        match me.centuries.checked_add(rhs.centuries) {
            None => {
                // Overflowed, so we've hit the bound.
                if me.centuries < 0 {
                    // We've hit the negative bound, so return MIN.
                    return Self::MIN;
                } else {
                    // We've hit the positive bound, so return MAX.
                    return Self::MAX;
                }
            }
            Some(centuries) => {
                me.centuries = centuries;
            }
        }

        if me.centuries == Self::MIN.centuries && self.nanoseconds < Self::MIN.nanoseconds {
            // Then we do the operation backward
            match me
                .nanoseconds
                .checked_sub(NANOSECONDS_PER_CENTURY - rhs.nanoseconds)
            {
                Some(nanos) => me.nanoseconds = nanos,
                None => {
                    me.centuries += 1; // Safe because we're at the MIN
                    me.nanoseconds = rhs.nanoseconds
                }
            }
        } else {
            match me.nanoseconds.checked_add(rhs.nanoseconds) {
                Some(nanoseconds) => me.nanoseconds = nanoseconds,
                None => {
                    // Rare case where somehow the input data was not normalized. So let's normalize it and call add again.
                    let mut rhs = rhs;
                    rhs.normalize();

                    match me.centuries.checked_add(rhs.centuries) {
                        None => return Self::MAX,
                        Some(centuries) => me.centuries = centuries,
                    };
                    // Now it will fit!
                    me.nanoseconds += rhs.nanoseconds;
                }
            }
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
    type Output = Self;

    /// # Subtraction
    /// This operation is a notch confusing with negative durations.
    /// As described in the `Duration` structure, a Duration of (-1, NANOSECONDS_PER_CENTURY-1) is closer to zero
    /// than (-1, 0).
    ///
    /// ## Algorithm
    ///
    /// ### A > B, and both are positive
    ///
    /// If A > B, then A.centuries is subtracted by B.centuries, and A.nanoseconds is subtracted by B.nanoseconds.
    /// If an overflow occurs, e.g. A.nanoseconds < B.nanoseconds, the number of nanoseconds is increased by the number of nanoseconds per century,
    /// and the number of centuries is decreased by one.
    ///
    /// ```
    /// use hifitime::{Duration, NANOSECONDS_PER_CENTURY};
    ///
    /// let a = Duration::from_parts(1, 1);
    /// let b = Duration::from_parts(0, 10);
    /// let c = Duration::from_parts(0, NANOSECONDS_PER_CENTURY - 9);
    /// assert_eq!(a - b, c);
    /// ```
    ///
    /// ### A < B, and both are positive
    ///
    /// In this case, the resulting duration will be negative. The number of centuries is a signed integer, so it is set to the difference of A.centuries - B.centuries.
    /// The number of nanoseconds however must be wrapped by the number of nanoseconds per century.
    /// For example:, let A = (0, 1) and B = (1, 10), then the resulting duration will be (-2, NANOSECONDS_PER_CENTURY - (10 - 1)). In this case, the centuries are set
    /// to -2 because B is _two_ centuries into the future (the number of centuries into the future is zero-indexed).
    /// ```
    /// use hifitime::{Duration, NANOSECONDS_PER_CENTURY};
    ///
    /// let a = Duration::from_parts(0, 1);
    /// let b = Duration::from_parts(1, 10);
    /// let c = Duration::from_parts(-2, NANOSECONDS_PER_CENTURY - 9);
    /// assert_eq!(a - b, c);
    /// ```
    ///
    /// ### A > B, both are negative
    ///
    /// In this case, we try to stick to normal arithmatics: (-9 - -10) = (-9 + 10) = +1.
    /// In this case, we can simply add the components of the duration together.
    /// For example, let A = (-1, NANOSECONDS_PER_CENTURY - 2), and B = (-1, NANOSECONDS_PER_CENTURY - 1). Respectively, A is _two_ nanoseconds _before_ Duration::ZERO
    /// and B is _one_ nanosecond before Duration::ZERO. Then, A-B should be one nanoseconds before zero, i.e. (-1, NANOSECONDS_PER_CENTURY - 1).
    /// This is because we _subtract_ "negative one nanosecond" from a "negative minus two nanoseconds", which corresponds to _adding_ the opposite, and the
    /// opposite of "negative one nanosecond" is "positive one nanosecond".
    ///
    /// ```
    /// use hifitime::{Duration, NANOSECONDS_PER_CENTURY};
    ///
    /// let a = Duration::from_parts(-1, NANOSECONDS_PER_CENTURY - 9);
    /// let b = Duration::from_parts(-1, NANOSECONDS_PER_CENTURY - 10);
    /// let c = Duration::from_parts(0, 1);
    /// assert_eq!(a - b, c);
    /// ```
    ///
    /// ### A < B, both are negative
    ///
    /// Just like in the prior case, we try to stick to normal arithmatics: (-10 - -9) = (-10 + 9) = -1.
    ///
    /// ```
    /// use hifitime::{Duration, NANOSECONDS_PER_CENTURY};
    ///
    /// let a = Duration::from_parts(-1, NANOSECONDS_PER_CENTURY - 10);
    /// let b = Duration::from_parts(-1, NANOSECONDS_PER_CENTURY - 9);
    /// let c = Duration::from_parts(-1, NANOSECONDS_PER_CENTURY - 1);
    /// assert_eq!(a - b, c);
    /// ```
    ///
    /// ### MIN is the minimum
    ///
    /// One cannot subtract anything from the MIN.
    ///
    /// ```
    /// use hifitime::Duration;
    ///
    /// let one_ns = Duration::from_parts(0, 1);
    /// assert_eq!(Duration::MIN - one_ns, Duration::MIN);
    /// ```
    fn sub(self, rhs: Self) -> Self {
        let mut me = self;
        match me.centuries.checked_sub(rhs.centuries) {
            None => {
                // Underflowed, so we've hit the min
                return Self::MIN;
            }
            Some(centuries) => {
                me.centuries = centuries;
            }
        }

        match me.nanoseconds.checked_sub(rhs.nanoseconds) {
            None => {
                // Decrease the number of centuries, and realign
                match me.centuries.checked_sub(1) {
                    Some(centuries) => {
                        me.centuries = centuries;
                        me.nanoseconds = me.nanoseconds + NANOSECONDS_PER_CENTURY - rhs.nanoseconds;
                    }
                    None => {
                        // We're at the min number of centuries already, and we have extra nanos, so we're saturated the duration limit
                        return Self::MIN;
                    }
                };
                // me.nanoseconds = me.nanoseconds + NANOSECONDS_PER_CENTURY - rhs.nanoseconds;
            }
            Some(nanos) => me.nanoseconds = nanos,
        };

        me.normalize();
        me
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

// Allow adding with a Unit directly
impl Add<Unit> for Duration {
    type Output = Self;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: Unit) -> Self {
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
        if self == Self::MIN {
            Self::MAX
        } else if self == Self::MAX {
            Self::MIN
        } else {
            match NANOSECONDS_PER_CENTURY.checked_sub(self.nanoseconds) {
                Some(nanoseconds) => {
                    // yay
                    Self::from_parts(-self.centuries - 1, nanoseconds)
                }
                None => {
                    if self > Duration::ZERO {
                        let dur_to_max = Self::MAX - self;
                        Self::MIN + dur_to_max
                    } else {
                        let dur_to_min = Self::MIN + self;
                        Self::MAX - dur_to_min
                    }
                }
            }
        }
    }
}

#[cfg(not(kani))]
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
    ///  + `+` or `-` indicates a timezone offset
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
    /// assert_eq!(Duration::from_str("5 h 256 ms 1 ns").unwrap(), 5 * Unit::Hour + 256 * Unit::Millisecond + Unit::Nanosecond);
    /// assert_eq!(Duration::from_str("-01:15:30").unwrap(), -(1 * Unit::Hour + 15 * Unit::Minute + 30 * Unit::Second));
    /// assert_eq!(Duration::from_str("+3615").unwrap(), 36 * Unit::Hour + 15 * Unit::Minute);
    /// ```
    fn from_str(s_in: &str) -> Result<Self, Self::Err> {
        // Each part of a duration as days, hours, minutes, seconds, millisecond, microseconds, and nanoseconds
        let mut decomposed = [0.0_f64; 7];

        let mut prev_idx = 0;
        let mut seeking_number = true;
        let mut latest_value = 0.0;

        let s = s_in.trim();

        if s.is_empty() {
            return Err(Errors::ParseError(ParsingErrors::ValueError));
        }

        // There is at least one character, so we can unwrap this.
        if let Some(char) = s.chars().next() {
            if char == '+' || char == '-' {
                // This is a timezone offset.
                let offset_sign = if char == '-' { -1 } else { 1 };

                let indexes: (usize, usize, usize) = (1, 3, 5);
                let colon = if s.len() == 3 || s.len() == 5 || s.len() == 7 {
                    // There is a zero or even number of separators between the hours, minutes, and seconds.
                    // Only zero (or one) characters separator is supported. This will return a ValueError later if there is
                    // an even but greater than one character separator.
                    0
                } else if s.len() == 4 || s.len() == 6 || s.len() == 9 {
                    // There is an odd number of characters as a separator between the hours, minutes, and seconds.
                    // Only one character separator is supported. This will return a ValueError later if there is
                    // an odd but greater than one character separator.
                    1
                } else {
                    // This invalid
                    return Err(Errors::ParseError(ParsingErrors::ValueError));
                };

                // Fetch the hours
                let hours: i64 = match lexical_core::parse(s[indexes.0..indexes.1].as_bytes()) {
                    Ok(val) => val,
                    Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
                };

                let mut minutes: i64 = 0;
                let mut seconds: i64 = 0;

                match s.get(indexes.1 + colon..indexes.2 + colon) {
                    None => {
                        //Do nothing, we've reached the end of the useful data.
                    }
                    Some(subs) => {
                        // Fetch the minutes
                        match lexical_core::parse(subs.as_bytes()) {
                            Ok(val) => minutes = val,
                            Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
                        }

                        match s.get(indexes.2 + 2 * colon..) {
                            None => {
                                // Do nothing, there are no seconds inthis offset
                            }
                            Some(subs) => {
                                if !subs.is_empty() {
                                    // Fetch the seconds
                                    match lexical_core::parse(subs.as_bytes()) {
                                        Ok(val) => seconds = val,
                                        Err(_) => {
                                            return Err(Errors::ParseError(
                                                ParsingErrors::ValueError,
                                            ))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Return the constructed offset
                if offset_sign == -1 {
                    return Ok(-(hours * Unit::Hour
                        + minutes * Unit::Minute
                        + seconds * Unit::Second));
                } else {
                    return Ok(hours * Unit::Hour
                        + minutes * Unit::Minute
                        + seconds * Unit::Second);
                }
            }
        };

        for (idx, char) in s.chars().enumerate() {
            if char == ' ' || idx == s.len() - 1 {
                if seeking_number {
                    if prev_idx == idx {
                        // We've reached the end of the string and it didn't end with a unit
                        return Err(Errors::ParseError(ParsingErrors::UnknownOrMissingUnit));
                    }
                    // We've found a new space so let's parse whatever precedes it
                    match lexical_core::parse(s[prev_idx..idx].as_bytes()) {
                        Ok(val) => latest_value = val,
                        Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
                    }
                    // We'll now seek a unit
                    seeking_number = false;
                } else {
                    // We're seeking a unit not a number, so let's parse the unit we just found and remember the position.
                    let end_idx = if idx == s.len() - 1 { idx + 1 } else { idx };
                    let pos = match &s[prev_idx..end_idx] {
                        "d" | "days" | "day" => 0,
                        "h" | "hours" | "hour" => 1,
                        "min" | "mins" | "minute" | "minutes" => 2,
                        "s" | "second" | "seconds" => 3,
                        "ms" | "millisecond" | "milliseconds" => 4,
                        "us" | "microsecond" | "microseconds" => 5,
                        "ns" | "nanosecond" | "nanoseconds" => 6,
                        _ => {
                            return Err(Errors::ParseError(ParsingErrors::UnknownOrMissingUnit));
                        }
                    };
                    // Store the value
                    decomposed[pos] = latest_value;
                    // Now we switch to seeking a value
                    seeking_number = true;
                }
                prev_idx = idx + 1;
            }
        }

        Ok(Duration::compose_f64(
            1,
            decomposed[0],
            decomposed[1],
            decomposed[2],
            decomposed[3],
            decomposed[4],
            decomposed[5],
            decomposed[6],
        ))
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
impl From<Duration> for std::time::Duration {
    /// Converts a duration into an std::time::Duration
    ///
    /// # Limitations
    /// 1. If the duration is negative, this will return a std::time::Duration::ZERO.
    /// 2. If the duration larger than the MAX duration, this will return std::time::Duration::MAX
    fn from(hf_duration: Duration) -> Self {
        let (sign, days, hours, minutes, seconds, milli, us, nano) = hf_duration.decompose();
        if sign < 0 {
            std::time::Duration::ZERO
        } else {
            // Build the seconds separately from the nanos.
            let above_ns_f64: f64 =
                Duration::compose(sign, days, hours, minutes, seconds, milli, us, 0).to_seconds();
            std::time::Duration::new(above_ns_f64 as u64, nano as u32)
        }
    }
}

#[cfg(feature = "std")]
impl From<std::time::Duration> for Duration {
    /// Converts a duration into an std::time::Duration
    ///
    /// # Limitations
    /// 1. If the duration is negative, this will return a std::time::Duration::ZERO.
    /// 2. If the duration larger than the MAX duration, this will return std::time::Duration::MAX
    fn from(std_duration: std::time::Duration) -> Self {
        std_duration.as_secs_f64() * Unit::Second
    }
}

#[test]
#[cfg(feature = "serde")]
fn test_serdes() {
    let dt = Duration::from_seconds(10.1);
    let content = r#"{"centuries":0,"nanoseconds":10100000000}"#;
    assert_eq!(content, serde_json::to_string(&dt).unwrap());
    let parsed: Duration = serde_json::from_str(content).unwrap();
    assert_eq!(dt, parsed);
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

#[cfg(kani)]
#[kani::proof]
fn formal_duration_normalize_any() {
    let dur: Duration = kani::any();
    // Check that decompose never fails
    dur.decompose();
}

#[cfg(kani)]
#[kani::proof]
fn formal_duration_truncated_ns_reciprocity() {
    let nanoseconds: i64 = kani::any();
    let dur_from_part = Duration::from_truncated_nanoseconds(nanoseconds);

    let u_ns = dur_from_part.nanoseconds;
    let centuries = dur_from_part.centuries;
    if centuries <= -3 || centuries >= 3 {
        // Then it does not fit on a i64, so this function should return an error
        assert_eq!(
            dur_from_part.try_truncated_nanoseconds(),
            Err(Errors::Overflow)
        );
    } else if centuries == -1 {
        // If we are negative by just enough that the centuries is negative, then the truncated seconds
        // should be the unsigned nanoseconds wrapped by the number of nanoseconds per century.

        let expect_rslt = -((NANOSECONDS_PER_CENTURY - u_ns) as i64);

        let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();
        assert_eq!(recip_ns, expect_rslt);
    } else if centuries < 0 {
        // We fit on a i64 but we need to account for the number of nanoseconds wrapped to the negative centuries.

        let nanos = u_ns.rem_euclid(NANOSECONDS_PER_CENTURY);
        let expect_rslt = i64::from(centuries + 1) * NANOSECONDS_PER_CENTURY as i64 + nanos as i64;

        let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();
        assert_eq!(recip_ns, expect_rslt);
    } else {
        // Positive duration but enough to fit on an i64.
        let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();

        assert_eq!(recip_ns, nanoseconds);
    }
}

// #[cfg(kani)]
// #[kani::proof]
#[test]
fn formal_duration_seconds() {
    // let seconds: f64 = kani::any();
    let seconds =
        f64::from_bits(0b01000000010111111011010000110111101001100000110111100000_00000001);

    // kani::assume(seconds > 1e-9);
    // kani::assume(seconds < 1e14);

    if seconds.is_finite() {
        let big_seconds = seconds * 1e9;
        let floored = big_seconds.floor();
        // Remove the sub nanoseconds -- but this can lead to rounding errors!
        let truncated_ns = floored * 1e-9;

        let duration: Duration = Duration::from_seconds(truncated_ns);
        let truncated_out = duration.to_seconds();
        let floored_out = truncated_out * 1e9;
        // So we check that the data times 1e9 matches the rounded data
        if floored != floored_out {
            let floored_out_bits = floored_out.to_bits();
            let floored_bits = floored.to_bits();
            // Allow for ONE bit error on the LSB
            if floored_out_bits > floored_bits {
                assert_eq!(floored_out_bits - floored_bits, 1);
            } else {
                assert_eq!(floored_bits - floored_out_bits, 1);
            }
        }
        assert_eq!(floored_out, floored);
    }
}
