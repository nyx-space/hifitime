/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::ops::{Add, Mul, Sub};

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::{
    Duration, DAYS_PER_CENTURY, DAYS_PER_WEEK, DAYS_PER_WEEK_I128, NANOSECONDS_PER_CENTURY,
    NANOSECONDS_PER_DAY, NANOSECONDS_PER_HOUR, NANOSECONDS_PER_MICROSECOND,
    NANOSECONDS_PER_MILLISECOND, NANOSECONDS_PER_MINUTE, NANOSECONDS_PER_SECOND, SECONDS_PER_DAY,
    SECONDS_PER_HOUR, SECONDS_PER_MINUTE, ZEPTOSECONDS_PER_ATTOSECONDS,
    ZEPTOSECONDS_PER_FEMPTOSECONDS, ZEPTOSECONDS_PER_NANOSECONDS, ZEPTOSECONDS_PER_PICOSECONDS,
};

/// An Enum to perform time unit conversions.
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
pub enum Unit {
    Zeptosecond,
    Attosecond,
    Femtosecond,
    Picosecond,
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    /// 36525 days, is the number of days per century in the Julian calendar
    Century,
}

/// An Enum to convert frequencies to their approximate duration, **rounded to the closest nanosecond**.
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
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
    fn weeks(self) -> Duration {
        self * Unit::Week
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
            Unit::Week => DAYS_PER_WEEK * SECONDS_PER_DAY,
            Unit::Day => SECONDS_PER_DAY,
            Unit::Hour => SECONDS_PER_HOUR,
            Unit::Minute => SECONDS_PER_MINUTE,
            Unit::Second => 1.0,
            Unit::Millisecond => 1e-3,
            Unit::Microsecond => 1e-6,
            Unit::Nanosecond => 1e-9,
            Unit::Picosecond => 1e-12,
            Unit::Femtosecond => 1e-15,
            Unit::Attosecond => 1e-18,
            Unit::Zeptosecond => 1e-21,
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

/// Allows conversion of a Unit into a u8 where 0 is a zeptosecond and 12 is a century.
impl From<Unit> for u8 {
    fn from(unit: Unit) -> Self {
        match unit {
            Unit::Zeptosecond => 0,
            Unit::Attosecond => 1,
            Unit::Femtosecond => 2,
            Unit::Picosecond => 3,
            Unit::Nanosecond => 4,
            Unit::Microsecond => 5,
            Unit::Millisecond => 6,
            Unit::Second => 7,
            Unit::Minute => 8,
            Unit::Hour => 9,
            Unit::Day => 10,
            Unit::Week => 11,
            Unit::Century => 12,
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
            0 => Unit::Zeptosecond,
            1 => Unit::Attosecond,
            2 => Unit::Femtosecond,
            3 => Unit::Picosecond,
            4 => Unit::Nanosecond,
            5 => Unit::Microsecond,
            6 => Unit::Millisecond,
            7 => Unit::Second,
            8 => Unit::Minute,
            9 => Unit::Hour,
            10 => Unit::Day,
            11 => Unit::Week,
            _ => Unit::Century,
        }
    }
}

impl Mul<i128> for Unit {
    type Output = Duration;

    /// Converts the input values to i128 and creates a duration from that
    /// This method will necessarily ignore durations below nanoseconds
    fn mul(self, q: i128) -> Duration {
        let factor_zs = match self {
            Unit::Century => NANOSECONDS_PER_CENTURY * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Week => DAYS_PER_WEEK_I128 * NANOSECONDS_PER_DAY * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Day => NANOSECONDS_PER_DAY * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Hour => NANOSECONDS_PER_HOUR * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Minute => NANOSECONDS_PER_MINUTE * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Second => NANOSECONDS_PER_SECOND * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Millisecond => NANOSECONDS_PER_MILLISECOND * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Microsecond => NANOSECONDS_PER_MICROSECOND * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Nanosecond => ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Picosecond => ZEPTOSECONDS_PER_PICOSECONDS,
            Unit::Femtosecond => ZEPTOSECONDS_PER_FEMPTOSECONDS,
            Unit::Attosecond => ZEPTOSECONDS_PER_ATTOSECONDS,
            Self::Zeptosecond => 1,
        };

        match q.checked_mul(factor_zs) {
            Some(zeptoseconds) => Duration { zeptoseconds },
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
        let factor_zs = match self {
            Unit::Century => NANOSECONDS_PER_CENTURY * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Week => DAYS_PER_WEEK_I128 * NANOSECONDS_PER_DAY * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Day => NANOSECONDS_PER_DAY * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Hour => NANOSECONDS_PER_HOUR * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Minute => NANOSECONDS_PER_MINUTE * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Second => NANOSECONDS_PER_SECOND * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Millisecond => NANOSECONDS_PER_MILLISECOND * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Microsecond => NANOSECONDS_PER_MICROSECOND * ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Nanosecond => ZEPTOSECONDS_PER_NANOSECONDS,
            Unit::Picosecond => ZEPTOSECONDS_PER_PICOSECONDS,
            Unit::Femtosecond => ZEPTOSECONDS_PER_FEMPTOSECONDS,
            Unit::Attosecond => ZEPTOSECONDS_PER_ATTOSECONDS,
            Self::Zeptosecond => 1,
        };

        // Bound checking to prevent overflows
        if q >= f64::MAX / (factor_zs as f64) {
            Duration::MAX
        } else if q <= f64::MIN / (factor_zs as f64) {
            Duration::MIN
        } else {
            Duration {
                zeptoseconds: (q * (factor_zs as f64)) as i128,
            }
        }
    }
}

#[test]
fn test_unit_conversion() {
    for unit_u8 in 0..u8::MAX {
        let unit = Unit::from(unit_u8);
        let unit_u8_back: u8 = unit.into();
        // If the u8 is greater than 9, it isn't valid and necessarily encoded as Second.
        if unit_u8 < 13 {
            assert_eq!(unit_u8_back, unit_u8, "got {unit_u8_back} want {unit_u8}");
        } else {
            assert_eq!(unit, Unit::Century);
        }
    }
}
