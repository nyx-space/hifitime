/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::ops::{Add, Mul, Sub};

#[cfg(not(feature = "std"))]
use num_traits::Float;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::{
    Duration, DAYS_PER_CENTURY, NANOSECONDS_PER_CENTURY, NANOSECONDS_PER_DAY, NANOSECONDS_PER_HOUR,
    NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND, NANOSECONDS_PER_MINUTE,
    NANOSECONDS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};

/// An Enum to perform time unit conversions.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "python", pyclass)]
pub enum Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    /// 36525 days, is the number of days per century in the Julian calendar
    Century,
}

/// An Enum to convert frequencies to their approximate duration, **rounded to the closest nanosecond**.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "python", pyclass)]
pub enum Freq {
    GigaHertz,
    MegaHertz,
    KiloHertz,
    Hertz,
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

impl Default for Unit {
    fn default() -> Self {
        Self::Second
    }
}

impl Default for Freq {
    fn default() -> Self {
        Self::Hertz
    }
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

#[cfg_attr(feature = "python", pymethods)]
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
}

/// Allows conversion of a Unit into a u8 with the following mapping.
/// 0: Second; 1: Nanosecond; 2: Microsecond; 3: Millisecond; 4: Minute; 5: Hour; 6: Day; 7: Century
impl From<Unit> for u8 {
    fn from(unit: Unit) -> Self {
        match unit {
            Unit::Nanosecond => 1,
            Unit::Microsecond => 2,
            Unit::Millisecond => 3,
            Unit::Minute => 4,
            Unit::Hour => 5,
            Unit::Day => 6,
            Unit::Century => 7,
            Unit::Second => 0,
        }
    }
}

impl From<&Unit> for u8 {
    fn from(unit: &Unit) -> Self {
        u8::from(*unit)
    }
}

/// Allows conversion of a u8 into a Unit. Defaults to Second if the u8 is not a valid Unit representation.
impl From<u8> for Unit {
    fn from(val: u8) -> Self {
        match val {
            1 => Unit::Nanosecond,
            2 => Unit::Microsecond,
            3 => Unit::Millisecond,
            4 => Unit::Minute,
            5 => Unit::Hour,
            6 => Unit::Day,
            7 => Unit::Century,
            _ => Unit::Second,
        }
    }
}

impl Mul<i64> for Unit {
    type Output = Duration;

    /// Converts the input values to i128 and creates a duration from that
    /// This method will necessarily ignore durations below nanoseconds
    fn mul(self, q: i64) -> Duration {
        let factor = match self {
            Unit::Century => NANOSECONDS_PER_CENTURY as i64,
            Unit::Day => NANOSECONDS_PER_DAY as i64,
            Unit::Hour => NANOSECONDS_PER_HOUR as i64,
            Unit::Minute => NANOSECONDS_PER_MINUTE as i64,
            Unit::Second => NANOSECONDS_PER_SECOND as i64,
            Unit::Millisecond => NANOSECONDS_PER_MILLISECOND as i64,
            Unit::Microsecond => NANOSECONDS_PER_MICROSECOND as i64,
            Unit::Nanosecond => 1,
        };

        match q.checked_mul(factor) {
            Some(total_ns) => {
                if total_ns.abs() < i64::MAX {
                    Duration::from_truncated_nanoseconds(total_ns)
                } else {
                    Duration::from_total_nanoseconds(i128::from(total_ns))
                }
            }
            None => {
                if q.is_negative() {
                    Duration::MIN
                } else {
                    Duration::MAX
                }
            }
        }
    }
}

impl Mul<f64> for Unit {
    type Output = Duration;

    /// Creates a duration from that f64
    ///
    /// ## Limitations
    /// 1. If the input value times the unit does not fit on a Duration, then Duration::MAX or Duration::MIN will be returned
    /// depending on whether the value would have overflowed or underflowed (respectively).
    /// 2. Floating point operations may round differently on different processors. It's advised to use integer initialization of Durations whenever possible.
    fn mul(self, q: f64) -> Duration {
        let factor = match self {
            Unit::Century => NANOSECONDS_PER_CENTURY as f64,
            Unit::Day => NANOSECONDS_PER_DAY as f64,
            Unit::Hour => NANOSECONDS_PER_HOUR as f64,
            Unit::Minute => NANOSECONDS_PER_MINUTE as f64,
            Unit::Second => NANOSECONDS_PER_SECOND as f64,
            Unit::Millisecond => NANOSECONDS_PER_MILLISECOND as f64,
            Unit::Microsecond => NANOSECONDS_PER_MICROSECOND as f64,
            Unit::Nanosecond => 1.0,
        };

        // Bound checking to prevent overflows
        if q >= f64::MAX / factor {
            Duration::MAX
        } else if q <= f64::MIN / factor {
            Duration::MIN
        } else {
            let total_ns = q * factor;
            if total_ns.abs() < (i64::MAX as f64) {
                Duration::from_truncated_nanoseconds(total_ns as i64)
            } else {
                Duration::from_total_nanoseconds(total_ns as i128)
            }
        }
    }
}

#[test]
fn test_unit_conversion() {
    for unit_u8 in 0..u8::MAX {
        let unit = Unit::from(unit_u8);
        let unit_u8_back: u8 = unit.into();
        // If the u8 is greater than 8, it isn't valid and necessarily encoded as Second.
        if unit_u8 < 8 {
            assert_eq!(unit_u8_back, unit_u8, "got {unit_u8_back} want {unit_u8}");
        } else {
            assert_eq!(unit, Unit::Second);
        }
    }
}
