/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use super::{Duration, PreciseDuration, Unit};

use core::fmt;
use core::hash::Hash;
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

#[cfg(doc)]
use crate::{Epoch, GenericEpoch, PreciseEpoch};

/// Internal representation of the duration stored in a [GenericEpoch].
///
/// Two implementations are provided:
/// + [Duration]: nanosecond precision on 80 bits, used by [Epoch];
/// + [PreciseDuration]: zeptosecond precision on 128 bits, used by [PreciseEpoch].
///
/// Any [Duration] must be convertible without loss into the implementing type, such that all of the
/// time scale constants (reference epochs, offsets, leap seconds) can be shared between both implementations.
///
/// The methods of this trait are named after the inherent methods of [Duration] and behave identically
/// for [Duration]: implementing this trait on [Duration] does not change any of its existing behavior.
pub trait DurationTrait:
    Copy
    + Clone
    + fmt::Debug
    + fmt::Display
    + Default
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Add<Output = Self>
    + Sub<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
    + From<Duration>
    + Send
    + Sync
    + 'static
{
    /// A duration of exactly zero
    const ZERO: Self;
    /// Smallest representable positive duration
    const EPSILON: Self;
    /// Minimum duration that can be represented
    const MIN: Self;
    /// Maximum duration that can be represented
    const MAX: Self;

    /// Number of decimal digits that this representation stores below the nanosecond:
    /// 0 for [Duration], 12 for [PreciseDuration] (zeptoseconds).
    const SUBNANOSECOND_DIGITS: u32;

    /// Converts into a nanosecond [Duration], flooring toward negative infinity
    /// (lossless for [Duration] itself).
    fn to_duration(&self) -> Duration;

    /// Returns the zeptoseconds within the current nanosecond, in `[0; 1e12)`.
    /// Always zero for [Duration].
    fn subnanoseconds(&self) -> u64;

    /// Adds the provided number of zeptoseconds to this duration, if this representation can store them.
    /// This is a no-op for [Duration].
    fn add_subnanoseconds(self, zeptoseconds: u64) -> Self;

    /// Creates a duration from a floating point value in the provided unit.
    fn from_f64(value: f64, unit: Unit) -> Self;

    /// Creates a duration from a floating point number of seconds.
    fn from_seconds(value: f64) -> Self {
        Self::from_f64(value, Unit::Second)
    }

    /// Returns this duration as a floating point number of seconds.
    fn to_seconds(&self) -> f64;

    /// Returns this duration as a floating point value in the provided unit.
    fn to_unit(&self, unit: Unit) -> f64;

    /// Multiplies this duration by a floating point factor.
    fn mul_f64(self, q: f64) -> Self;

    /// Floors this duration to the provided duration (see [Duration::floor]).
    fn floor(&self, duration: Self) -> Self;

    /// Ceils this duration to the provided duration (see [Duration::ceil]).
    fn ceil(&self, duration: Self) -> Self;

    /// Rounds this duration to the provided duration (see [Duration::round]).
    fn round(&self, duration: Self) -> Self;

    /// Returns the absolute value of this duration.
    fn abs(&self) -> Self;

    /// Returns -1, 0, or 1 depending on the sign of this duration.
    fn signum(&self) -> i8;
}

impl DurationTrait for Duration {
    const ZERO: Self = Duration::ZERO;
    const EPSILON: Self = Duration::EPSILON;
    const MIN: Self = Duration::MIN;
    const MAX: Self = Duration::MAX;
    const SUBNANOSECOND_DIGITS: u32 = 0;

    #[inline]
    fn to_duration(&self) -> Duration {
        *self
    }

    #[inline]
    fn subnanoseconds(&self) -> u64 {
        0
    }

    #[inline]
    fn add_subnanoseconds(self, _zeptoseconds: u64) -> Self {
        self
    }

    #[inline]
    fn from_f64(value: f64, unit: Unit) -> Self {
        unit * value
    }

    #[inline]
    fn to_seconds(&self) -> f64 {
        Duration::to_seconds(self)
    }

    #[inline]
    fn to_unit(&self, unit: Unit) -> f64 {
        Duration::to_unit(self, unit)
    }

    #[inline]
    fn mul_f64(self, q: f64) -> Self {
        self * q
    }

    #[inline]
    fn floor(&self, duration: Self) -> Self {
        Duration::floor(self, duration)
    }

    #[inline]
    fn ceil(&self, duration: Self) -> Self {
        Duration::ceil(self, duration)
    }

    #[inline]
    fn round(&self, duration: Self) -> Self {
        Duration::round(self, duration)
    }

    #[inline]
    fn abs(&self) -> Self {
        Duration::abs(self)
    }

    #[inline]
    fn signum(&self) -> i8 {
        Duration::signum(self)
    }
}

impl DurationTrait for PreciseDuration {
    const ZERO: Self = PreciseDuration::ZERO;
    const EPSILON: Self = PreciseDuration::EPSILON;
    const MIN: Self = PreciseDuration::MIN;
    const MAX: Self = PreciseDuration::MAX;
    const SUBNANOSECOND_DIGITS: u32 = 12;

    #[inline]
    fn to_duration(&self) -> Duration {
        PreciseDuration::to_duration(self)
    }

    #[inline]
    fn subnanoseconds(&self) -> u64 {
        PreciseDuration::subnanoseconds(self)
    }

    #[inline]
    fn add_subnanoseconds(self, zeptoseconds: u64) -> Self {
        self + PreciseDuration::from_total_zeptoseconds(zeptoseconds as i128)
    }

    #[inline]
    fn from_f64(value: f64, unit: Unit) -> Self {
        PreciseDuration::from_f64(value, unit)
    }

    #[inline]
    fn to_seconds(&self) -> f64 {
        PreciseDuration::to_seconds(self)
    }

    #[inline]
    fn to_unit(&self, unit: Unit) -> f64 {
        PreciseDuration::to_unit(self, unit)
    }

    #[inline]
    fn mul_f64(self, q: f64) -> Self {
        PreciseDuration::mul_f64(self, q)
    }

    #[inline]
    fn floor(&self, duration: Self) -> Self {
        PreciseDuration::floor(self, duration)
    }

    #[inline]
    fn ceil(&self, duration: Self) -> Self {
        PreciseDuration::ceil(self, duration)
    }

    #[inline]
    fn round(&self, duration: Self) -> Self {
        PreciseDuration::round(self, duration)
    }

    #[inline]
    fn abs(&self) -> Self {
        PreciseDuration::abs(self)
    }

    #[inline]
    fn signum(&self) -> i8 {
        PreciseDuration::signum(self)
    }
}
