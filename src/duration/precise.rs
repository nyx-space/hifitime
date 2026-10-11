/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

//! Zeptosecond precision durations, stored on a single signed 128 bit integer.

use super::{Duration, Unit, NANOSECONDS_PER_CENTURY};
use crate::{DAYS_PER_WEEK_I64, SECONDS_PER_DAY_I64};

use core::cmp::Ordering;
use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[cfg(doc)]
use crate::PreciseEpoch;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

/// Number of zeptoseconds (1e-21 s) in one nanosecond.
pub const ZEPTOSECONDS_PER_NANOSECOND: i128 = 1_000_000_000_000;
/// Number of zeptoseconds (1e-21 s) in one second.
pub const ZEPTOSECONDS_PER_SECOND: i128 = 1_000_000_000 * ZEPTOSECONDS_PER_NANOSECOND;

const ZS_PER_PICOSECOND: i128 = 1_000_000_000;
const ZS_PER_FEMTOSECOND: i128 = 1_000_000;
const ZS_PER_ATTOSECOND: i128 = 1_000;

/// Defines durations with zeptosecond precision (1e-21 s), stored on a single [i128].
///
/// The representable range is about ±5.39 billion years (±1.7e17 s), which is a strict superset of
/// the range of [Duration] (±32,768 centuries). Hence, any [Duration] converts into a [PreciseDuration]
/// without loss, whereas the opposite conversion floors to the nanosecond (see [PreciseDuration::to_duration]).
///
/// This is the duration used by [PreciseEpoch], for applications that require the highest accuracy and
/// accept the cost of 128 bit arithmetic. All arithmetic operations saturate at [PreciseDuration::MIN] and [PreciseDuration::MAX].
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(C)]
pub struct PreciseDuration {
    pub(crate) zeptoseconds: i128,
}

impl PreciseDuration {
    /// A duration of exactly zero
    pub const ZERO: Self = Self { zeptoseconds: 0 };
    /// Maximum duration that can be represented
    pub const MAX: Self = Self {
        zeptoseconds: i128::MAX,
    };
    /// Minimum duration that can be represented
    pub const MIN: Self = Self {
        zeptoseconds: i128::MIN,
    };
    /// Smallest positive duration: one zeptosecond
    pub const EPSILON: Self = Self { zeptoseconds: 1 };
    /// Smallest positive duration: one zeptosecond
    pub const MIN_POSITIVE: Self = Self::EPSILON;
    /// Largest negative duration: minus one zeptosecond
    pub const MIN_NEGATIVE: Self = Self { zeptoseconds: -1 };

    /// Builds a [PreciseDuration] from a total number of zeptoseconds.
    #[must_use]
    pub const fn from_total_zeptoseconds(zeptoseconds: i128) -> Self {
        Self { zeptoseconds }
    }

    /// Returns the total number of zeptoseconds of this duration.
    #[must_use]
    pub const fn total_zeptoseconds(&self) -> i128 {
        self.zeptoseconds
    }

    /// Builds a [PreciseDuration] from a total number of nanoseconds (saturating).
    #[must_use]
    pub const fn from_total_nanoseconds(nanos: i128) -> Self {
        Self {
            zeptoseconds: nanos.saturating_mul(ZEPTOSECONDS_PER_NANOSECOND),
        }
    }

    /// Returns the total number of nanoseconds, floored toward negative infinity.
    #[must_use]
    pub const fn total_nanoseconds(&self) -> i128 {
        self.zeptoseconds.div_euclid(ZEPTOSECONDS_PER_NANOSECOND)
    }

    /// Builds a [PreciseDuration] from an integer number of seconds and a sub-second
    /// number of zeptoseconds (which may be any value, it is simply added).
    #[must_use]
    pub const fn from_seconds_zeptoseconds(seconds: i64, zeptoseconds: i128) -> Self {
        Self {
            zeptoseconds: (seconds as i128)
                .saturating_mul(ZEPTOSECONDS_PER_SECOND)
                .saturating_add(zeptoseconds),
        }
    }

    /// Converts a nanosecond [Duration] into a [PreciseDuration]. This conversion is exact.
    #[must_use]
    pub const fn from_duration(duration: Duration) -> Self {
        // Duration::MAX is i16::MAX centuries plus NANOSECONDS_PER_CENTURY: ~1.03e35 zs, fits in i128.
        let ns = (duration.centuries as i128) * (NANOSECONDS_PER_CENTURY as i128)
            + (duration.nanoseconds as i128);
        Self {
            zeptoseconds: ns * ZEPTOSECONDS_PER_NANOSECOND,
        }
    }

    /// Converts this duration into a nanosecond [Duration], flooring toward negative infinity.
    /// The remainder is returned by [PreciseDuration::subnanoseconds], such that
    /// `PreciseDuration::from(self.to_duration()) + PreciseDuration::from_total_zeptoseconds(self.subnanoseconds() as i128) == self`
    /// (as long as the value fits within the range of [Duration], which saturates otherwise).
    #[must_use]
    pub const fn to_duration(&self) -> Duration {
        Duration::from_total_nanoseconds(self.total_nanoseconds())
    }

    /// Returns the zeptoseconds within the current nanosecond, always in `[0; 1e12)`.
    #[must_use]
    pub const fn subnanoseconds(&self) -> u64 {
        self.zeptoseconds.rem_euclid(ZEPTOSECONDS_PER_NANOSECOND) as u64
    }

    /// Converts this duration into a nanosecond [Duration], rounding to the closest nanosecond (ties away from zero).
    #[must_use]
    pub fn to_duration_rounded(&self) -> Duration {
        let half = ZEPTOSECONDS_PER_NANOSECOND / 2;
        let ns = self.total_nanoseconds()
            + if self.subnanoseconds() as i128 >= half {
                1
            } else {
                0
            };
        Duration::from_total_nanoseconds(ns)
    }

    /// Number of zeptoseconds in the provided unit.
    #[must_use]
    pub const fn zeptoseconds_per_unit(unit: Unit) -> i128 {
        let ns: i128 = match unit {
            Unit::Nanosecond => 1,
            Unit::Microsecond => 1_000,
            Unit::Millisecond => 1_000_000,
            Unit::Second => 1_000_000_000,
            Unit::Minute => 60_000_000_000,
            Unit::Hour => 3_600_000_000_000,
            Unit::Day => (SECONDS_PER_DAY_I64 as i128) * 1_000_000_000,
            Unit::Week => {
                (DAYS_PER_WEEK_I64 as i128) * (SECONDS_PER_DAY_I64 as i128) * 1_000_000_000
            }
            Unit::Century => NANOSECONDS_PER_CENTURY as i128,
        };
        ns * ZEPTOSECONDS_PER_NANOSECOND
    }

    /// Creates a duration from a floating point value in the provided unit.
    ///
    /// Contrary to [Duration], this conversion is *exact* with respect to the binary value of the provided [f64]:
    /// the result is the closest zeptosecond to `value × unit`, computed without any intermediate floating point rounding.
    /// Saturates to [PreciseDuration::MIN]/[PreciseDuration::MAX], and NaN maps to zero.
    #[must_use]
    pub fn from_f64(value: f64, unit: Unit) -> Self {
        Self {
            zeptoseconds: f64_mul_int_round(value, Self::zeptoseconds_per_unit(unit) as u128),
        }
    }

    /// Creates a duration from the provided number of seconds (exact w.r.t. the [f64] value).
    #[must_use]
    pub fn from_seconds(value: f64) -> Self {
        Self::from_f64(value, Unit::Second)
    }

    /// Creates a duration from the provided number of days (exact w.r.t. the [f64] value).
    #[must_use]
    pub fn from_days(value: f64) -> Self {
        Self::from_f64(value, Unit::Day)
    }

    /// Returns this duration as a floating point value in the provided unit.
    /// The integer and fractional parts are converted separately to preserve as much precision as possible.
    #[must_use]
    pub fn to_unit(&self, unit: Unit) -> f64 {
        let per_unit = Self::zeptoseconds_per_unit(unit);
        // A single (costly) 128 bit division: the remainder keeps the sign of the dividend, which is fine here.
        let whole = self.zeptoseconds / per_unit;
        let frac = self.zeptoseconds - whole * per_unit;
        (whole as f64) + (frac as f64) / (per_unit as f64)
    }

    /// Returns this duration in seconds.
    #[must_use]
    pub fn to_seconds(&self) -> f64 {
        self.to_unit(Unit::Second)
    }

    /// Multiplies this duration by a floating point factor, rounding to the closest zeptosecond.
    ///
    /// + If the factor has a short decimal representation `n / 10^p` (with p ≤ 19), as found by `Duration * f64`,
    ///   the product is computed exactly with that decimal value. Hence, decimal constants such as
    ///   `L_G = 6.969290134e-10` are applied exactly, as defined by the IAU.
    /// + Otherwise (e.g. `L_G / (1 - L_G)`), the product is computed exactly with the binary value of the [f64].
    ///
    /// Saturates to [PreciseDuration::MIN]/[PreciseDuration::MAX], and NaN maps to zero.
    #[must_use]
    pub fn mul_f64(self, q: f64) -> Self {
        if q.is_nan() || self.zeptoseconds == 0 {
            return Self::ZERO;
        }
        match decimal_factor(q) {
            Some((num, p)) => Self {
                zeptoseconds: mul_div_pow10_round(self.zeptoseconds, num, p),
            },
            None => {
                let magnitude = f64_mul_int_round(q, self.zeptoseconds.unsigned_abs());
                Self {
                    zeptoseconds: if self.zeptoseconds < 0 {
                        magnitude.saturating_neg()
                    } else {
                        magnitude
                    },
                }
            }
        }
    }

    /// Returns the absolute value of this duration (saturating).
    #[must_use]
    pub const fn abs(&self) -> Self {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_abs(),
        }
    }

    /// Returns the sign of this duration: -1, 0 or 1.
    #[must_use]
    pub const fn signum(&self) -> i8 {
        self.zeptoseconds.signum() as i8
    }

    /// Returns whether this duration is negative.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.zeptoseconds < 0
    }

    /// Floors this duration to the closest multiple of the provided duration, toward zero
    /// (same semantics as [Duration::floor]).
    #[must_use]
    pub fn floor(&self, duration: Self) -> Self {
        if duration.zeptoseconds == 0 {
            Self::ZERO
        } else {
            Self {
                zeptoseconds: self.zeptoseconds - self.zeptoseconds % duration.zeptoseconds,
            }
        }
    }

    /// Floors then adds the absolute value of the provided duration (same semantics as [Duration::ceil]).
    #[must_use]
    pub fn ceil(&self, duration: Self) -> Self {
        self.floor(duration) + duration.abs()
    }

    /// Rounds this duration to the closest multiple of the provided duration (same semantics as [Duration::round]).
    #[must_use]
    pub fn round(&self, duration: Self) -> Self {
        let floored = self.floor(duration);
        let ceiled = self.ceil(duration);
        if *self - floored < (ceiled - *self).abs() {
            floored
        } else {
            ceiled
        }
    }

    /// Returns the minimum of the two durations.
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Ord::min(self, other)
    }

    /// Returns the maximum of the two durations.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Ord::max(self, other)
    }

    /// Decomposes this duration in its sign, days, hours, minutes, seconds, and the zeptoseconds within that second.
    #[must_use]
    pub const fn decompose(&self) -> (i8, u128, u8, u8, u8, u128) {
        let sign = self.signum();
        let abs = self.zeptoseconds.unsigned_abs();
        let zs_per_s = ZEPTOSECONDS_PER_SECOND as u128;
        let total_s = abs / zs_per_s;
        let subsec = abs % zs_per_s;
        let days = total_s / 86_400;
        let rem = total_s % 86_400;
        (
            sign,
            days,
            (rem / 3_600) as u8,
            ((rem % 3_600) / 60) as u8,
            (rem % 60) as u8,
            subsec,
        )
    }
}

impl From<Duration> for PreciseDuration {
    fn from(duration: Duration) -> Self {
        Self::from_duration(duration)
    }
}

impl From<Unit> for PreciseDuration {
    fn from(unit: Unit) -> Self {
        Self::from_total_zeptoseconds(Self::zeptoseconds_per_unit(unit))
    }
}

impl Add for PreciseDuration {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_add(rhs.zeptoseconds),
        }
    }
}

impl Sub for PreciseDuration {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_sub(rhs.zeptoseconds),
        }
    }
}

impl AddAssign for PreciseDuration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for PreciseDuration {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for PreciseDuration {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_neg(),
        }
    }
}

/// Mixed arithmetic: a nanosecond [Duration] is losslessly promoted.
impl Add<Duration> for PreciseDuration {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        self + Self::from(rhs)
    }
}

impl Sub<Duration> for PreciseDuration {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        self - Self::from(rhs)
    }
}

impl AddAssign<Duration> for PreciseDuration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign<Duration> for PreciseDuration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Add<Unit> for PreciseDuration {
    type Output = Self;
    fn add(self, rhs: Unit) -> Self {
        self + Self::from(rhs)
    }
}

impl Sub<Unit> for PreciseDuration {
    type Output = Self;
    fn sub(self, rhs: Unit) -> Self {
        self - Self::from(rhs)
    }
}

impl Mul<i64> for PreciseDuration {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_mul(rhs as i128),
        }
    }
}

impl Mul<f64> for PreciseDuration {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        self.mul_f64(rhs)
    }
}

impl Div<i64> for PreciseDuration {
    type Output = Self;
    /// Integer division, truncated toward zero. Division by zero saturates.
    fn div(self, rhs: i64) -> Self {
        if rhs == 0 {
            match self.zeptoseconds.cmp(&0) {
                Ordering::Less => Self::MIN,
                Ordering::Equal => Self::ZERO,
                Ordering::Greater => Self::MAX,
            }
        } else {
            Self {
                zeptoseconds: self.zeptoseconds.saturating_div(rhs as i128),
            }
        }
    }
}

impl PartialEq<Duration> for PreciseDuration {
    fn eq(&self, other: &Duration) -> bool {
        *self == Self::from(*other)
    }
}

impl PartialOrd<Duration> for PreciseDuration {
    fn partial_cmp(&self, other: &Duration) -> Option<Ordering> {
        Some(self.cmp(&Self::from(*other)))
    }
}

impl fmt::Display for PreciseDuration {
    /// Prints this duration with automatic selection of the units, down to the zeptosecond,
    /// e.g. `1 day 2 h 3 s 4 ns 5 as 6 zs`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.zeptoseconds == 0 {
            return write!(f, "0 zs");
        }
        let (sign, days, hours, minutes, seconds, subsec) = self.decompose();
        if sign == -1 {
            write!(f, "-")?;
        }
        let ms = subsec / 1_000_000_000_000_000_000;
        let us = (subsec / 1_000_000_000_000_000) % 1_000;
        let ns = (subsec / ZEPTOSECONDS_PER_NANOSECOND as u128) % 1_000;
        let ps = (subsec / ZS_PER_PICOSECOND as u128) % 1_000;
        let fs = (subsec / ZS_PER_FEMTOSECOND as u128) % 1_000;
        let at = (subsec / ZS_PER_ATTOSECOND as u128) % 1_000;
        let zs = subsec % 1_000;

        let values: [u128; 11] = [
            days,
            hours as u128,
            minutes as u128,
            seconds as u128,
            ms,
            us,
            ns,
            ps,
            fs,
            at,
            zs,
        ];
        let units = [
            if days > 1 { "days" } else { "day" },
            "h",
            "min",
            "s",
            "ms",
            "μs",
            "ns",
            "ps",
            "fs",
            "as",
            "zs",
        ];
        let mut insert_space = false;
        for (val, unit) in values.iter().zip(units.iter()) {
            if *val > 0 {
                if insert_space {
                    write!(f, " ")?;
                }
                write!(f, "{val} {unit}")?;
                insert_space = true;
            }
        }
        Ok(())
    }
}

/// Computes `round(value × factor)` exactly (ties away from zero), saturating to the i128 range.
fn f64_mul_int_round(value: f64, factor: u128) -> i128 {
    if value.is_nan() || factor == 0 || value == 0.0 {
        return 0;
    }
    if value.is_infinite() {
        return if value > 0.0 { i128::MAX } else { i128::MIN };
    }
    let bits = value.to_bits();
    let negative = (bits >> 63) != 0;
    let biased_exp = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    // value = mantissa × 2^exp
    let (mantissa, exp) = if biased_exp == 0 {
        (frac, -1074)
    } else {
        (frac | (1u64 << 52), biased_exp - 1075)
    };

    let magnitude = mul_shift_round(mantissa, factor, exp);

    match magnitude {
        Some(m) if negative => {
            if m > (i128::MAX as u128) + 1 {
                i128::MIN
            } else {
                (m as i128).wrapping_neg()
            }
        }
        Some(m) => {
            if m > i128::MAX as u128 {
                i128::MAX
            } else {
                m as i128
            }
        }
        None => {
            if negative {
                i128::MIN
            } else {
                i128::MAX
            }
        }
    }
}

/// Computes `round(m × f × 2^exp)` exactly (ties rounded up), or None if the result does not fit on an u128.
/// The intermediate product `m × f` is up to 181 bits, and is handled as `A × 2^64 + B` with u128 halves.
#[inline]
fn mul_shift_round(m: u64, f: u128, exp: i32) -> Option<u128> {
    let m = m as u128;
    if exp >= 0 {
        // m × f × 2^exp, all bits must fit.
        let prod = m.checked_mul(f)?;
        let shift = exp as u32;
        if shift >= 128 || prod.leading_zeros() < shift {
            if prod == 0 {
                return Some(0);
            }
            return None;
        }
        return Some(prod << shift);
    }
    let s = exp.unsigned_abs();
    // m × f = A × 2^64 + B, with A = m × f_hi < 2^117 and B = m × f_lo < 2^117
    let a = m * (f >> 64);
    let b = m * (f as u64 as u128);
    // = C × 2^64 + b_lo, with C = A + B_hi (no overflow: < 2^118)
    let c = a + (b >> 64);
    let b_lo = b as u64;
    let (truncated, round_bit) = if s >= 64 {
        let k = s - 64; // shift applied to C
        if k >= 128 {
            let rb = k == 128 && (c >> 127) & 1 == 1;
            (0, rb)
        } else if k == 0 {
            (c, (b_lo >> 63) & 1 == 1)
        } else {
            (c >> k, (c >> (k - 1)) & 1 == 1)
        }
    } else {
        // s in [1, 63]: (C << (64 - s)) | (b_lo >> s)
        let up = 64 - s;
        if c.leading_zeros() < up {
            return None;
        }
        (
            (c << up) | ((b_lo >> s) as u128),
            (b_lo >> (s - 1)) & 1 == 1,
        )
    };
    if round_bit {
        truncated.checked_add(1)
    } else {
        Some(truncated)
    }
}

/// Little endian 192 bit unsigned integer, kept as a slow but obviously correct oracle for the tests.
#[cfg(test)]
#[derive(Clone, Copy)]
struct U192([u64; 3]);

#[cfg(test)]
impl U192 {
    fn mul(a: u64, b: u128) -> Self {
        let lo = (a as u128) * (b as u64 as u128);
        let hi = (a as u128) * (b >> 64);
        let l0 = lo as u64;
        let mid = (lo >> 64) + (hi as u64 as u128);
        let l1 = mid as u64;
        let l2 = ((mid >> 64) + (hi >> 64)) as u64;
        Self([l0, l1, l2])
    }

    fn bit(&self, i: u32) -> bool {
        if i >= 192 {
            false
        } else {
            (self.0[(i / 64) as usize] >> (i % 64)) & 1 == 1
        }
    }

    /// Returns the bits [shift, shift + 128) as an u128, assuming shift < 192.
    fn window(&self, shift: u32) -> u128 {
        let mut out: u128 = 0;
        for i in 0..128 {
            if self.bit(shift + i) {
                out |= 1u128 << i;
            }
        }
        out
    }

    fn any_bit_from(&self, from: u32) -> bool {
        (from..192).any(|i| self.bit(i))
    }

    /// Shift right with rounding half away from zero; None if the result does not fit on an u128.
    fn shr_round(&self, shift: u32) -> Option<u128> {
        if shift > 192 {
            return Some(0);
        }
        if self.any_bit_from(shift + 128) {
            return None;
        }
        let truncated = if shift >= 192 { 0 } else { self.window(shift) };
        let round_up = shift > 0 && self.bit(shift - 1);
        if round_up {
            truncated.checked_add(1)
        } else {
            Some(truncated)
        }
    }

    /// Shift left; None if the result does not fit on an u128.
    fn shl_checked(&self, shift: u32) -> Option<u128> {
        if shift >= 128 {
            return if self.any_bit_from(0) { None } else { Some(0) };
        }
        if self.any_bit_from(128 - shift) {
            return None;
        }
        Some(self.window(0) << shift)
    }
}

/// Finds the exact decimal representation `num / 10^p` (p ≤ 19) of a factor, with the same search as `Duration * f64`.
/// Returns None if there is no such short representation, or if it does not fit on an i128.
fn decimal_factor(q: f64) -> Option<(i128, u32)> {
    let mut p: i32 = 0;
    let mut new_val = q;
    while new_val.is_finite() && (new_val.floor() - new_val).abs() >= f64::EPSILON && p < 19 {
        p += 1;
        new_val = q * 10.0_f64.powi(p);
    }
    if !new_val.is_finite()
        || new_val.abs() >= 1.7e38
        || (new_val.floor() - new_val).abs() >= f64::EPSILON
    {
        None
    } else {
        Some((new_val as i128, p as u32))
    }
}

/// Computes `round(x × num / 10^p)` without intermediate overflow (when the result fits), saturating otherwise.
fn mul_div_pow10_round(x: i128, num: i128, p: u32) -> i128 {
    let den = 10_i128.pow(p);
    let q = x.div_euclid(den);
    let r = x.rem_euclid(den); // 0 <= r < 10^19
                               // q × num may saturate, which is the desired behavior for out of range values.
    let high = q.saturating_mul(num);
    // r × num: r < 1e19 and |num| < 1.7e38 / ... may overflow if |num| is large, so use checked math.
    let low = match r.checked_mul(num) {
        Some(rn) => {
            // Round half up; r × num < 10^19 × 10^19 so 2 × rem cannot overflow.
            let base = rn.div_euclid(den);
            let rem = rn.rem_euclid(den);
            base + if rem * 2 >= den { 1 } else { 0 }
        }
        None => {
            // Only reachable for huge factors; fall back on an approximate split.
            let (n_hi, n_lo) = (num / den, num % den);
            r.saturating_mul(n_hi) + (r * n_lo) / den
        }
    };
    high.saturating_add(low)
}

#[cfg(test)]
mod ut_precise_duration {
    use super::*;
    use crate::TimeUnits;

    #[test]
    fn duration_roundtrip() {
        for d in [
            Duration::ZERO,
            Duration::MIN,
            Duration::MAX,
            Duration::MIN_NEGATIVE,
            Duration::MIN_POSITIVE,
            1.days() + 3.nanoseconds(),
            -(1.days() + 3.nanoseconds()),
        ] {
            let p = PreciseDuration::from(d);
            assert_eq!(p.to_duration(), d, "{d}");
            assert_eq!(p.subnanoseconds(), 0);
            assert_eq!(p, d);
        }
    }

    #[test]
    fn floor_to_nanoseconds() {
        let p = PreciseDuration::from_total_zeptoseconds(-1);
        assert_eq!(p.to_duration(), Duration::MIN_NEGATIVE);
        assert_eq!(p.subnanoseconds(), (ZEPTOSECONDS_PER_NANOSECOND - 1) as u64);
        assert_eq!(p.to_duration_rounded(), Duration::ZERO);
        let p = PreciseDuration::from_total_zeptoseconds(1_500_000_000_000);
        assert_eq!(p.to_duration(), 1.nanoseconds());
        assert_eq!(p.to_duration_rounded(), 2.nanoseconds());
    }

    #[test]
    fn from_f64_exact() {
        assert_eq!(
            PreciseDuration::from_seconds(1.0).zeptoseconds,
            ZEPTOSECONDS_PER_SECOND
        );
        assert_eq!(
            PreciseDuration::from_seconds(-2.5).zeptoseconds,
            -5 * ZEPTOSECONDS_PER_SECOND / 2
        );
        // 0.1 is not representable in binary: the exact value of the f64 is
        // 0.1000000000000000055511151231257827021181583404541015625 s
        assert_eq!(
            PreciseDuration::from_seconds(0.1).zeptoseconds,
            100_000_000_000_000_005_551
        );
        // 2^-70 s ≈ 8.47e-22 s rounds to 1 zs
        assert_eq!(
            PreciseDuration::from_seconds(2f64.powi(-70)).zeptoseconds,
            1
        );
        assert_eq!(PreciseDuration::from_seconds(1e-30).zeptoseconds, 0);
        assert_eq!(PreciseDuration::from_seconds(1e300), PreciseDuration::MAX);
        assert_eq!(PreciseDuration::from_seconds(-1e300), PreciseDuration::MIN);
        assert_eq!(
            PreciseDuration::from_seconds(f64::NAN),
            PreciseDuration::ZERO
        );
        assert_eq!(
            PreciseDuration::from_days(1.5),
            PreciseDuration::from(36.hours())
        );
        // Century is the largest unit
        assert_eq!(
            PreciseDuration::from_f64(-3.0, Unit::Century),
            PreciseDuration::from(Duration::from_parts(-3, 0))
        );
    }

    #[test]
    fn to_unit() {
        let p = PreciseDuration::from(1.days()) + PreciseDuration::from_total_zeptoseconds(1);
        assert_eq!(p.to_unit(Unit::Day), 1.0);
        let p = PreciseDuration::from_seconds(-0.5);
        assert_eq!(p.to_seconds(), -0.5);
        assert_eq!(p.to_unit(Unit::Millisecond), -500.0);
    }

    #[test]
    fn mul_f64_decimal_exact() {
        // 1e9 s × L_G must be exactly 0.6969290134 s
        let lg = 6.969_290_134e-10;
        let p = PreciseDuration::from(1_000_000_000.seconds()).mul_f64(lg);
        assert_eq!(p.zeptoseconds, 696_929_013_400_000_000_000);
        // Same as Duration for nanosecond values
        let d = 123_456_789.seconds();
        assert_eq!(PreciseDuration::from(d).mul_f64(lg).to_duration(), d * lg);
        assert_eq!(
            PreciseDuration::from(1.seconds()) * -2.0,
            PreciseDuration::from(-2.seconds())
        );
    }

    #[test]
    fn mul_shift_round_matches_oracle() {
        // Deterministic pseudo random sweep over mantissas, factors and exponents.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..20_000 {
            let m = next() >> 11; // 53 bits
            let f = ((next() as u128) << 64 | next() as u128) >> (next() % 128);
            let exp = (next() % 400) as i32 - 300;
            let prod = U192::mul(m, f);
            let oracle = if exp >= 0 {
                prod.shl_checked(exp as u32)
            } else {
                prod.shr_round((-exp) as u32)
            };
            assert_eq!(mul_shift_round(m, f, exp), oracle, "m={m} f={f} exp={exp}");
        }
    }

    #[test]
    fn display() {
        let p = PreciseDuration::from(1.days() + 3.seconds() + 4.nanoseconds())
            + PreciseDuration::from_total_zeptoseconds(5_006);
        assert_eq!(format!("{p}"), "1 day 3 s 4 ns 5 as 6 zs");
        assert_eq!(format!("{}", -p), "-1 day 3 s 4 ns 5 as 6 zs");
        assert_eq!(format!("{}", PreciseDuration::ZERO), "0 zs");
    }

    #[test]
    fn saturation() {
        assert_eq!(
            PreciseDuration::MAX + PreciseDuration::EPSILON,
            PreciseDuration::MAX
        );
        assert_eq!(
            PreciseDuration::MIN - PreciseDuration::EPSILON,
            PreciseDuration::MIN
        );
        assert_eq!(-PreciseDuration::MIN, PreciseDuration::MAX);
    }

    #[test]
    fn floor_ceil_round() {
        let d = 2.hours() + 3.minutes();
        let p = PreciseDuration::from(d);
        for unit in [
            1.hours(),
            30.minutes(),
            4.hours(),
            1.seconds(),
            1.hours() + 5.minutes(),
        ] {
            let u = PreciseDuration::from(unit);
            assert_eq!(p.floor(u).to_duration(), d.floor(unit));
            assert_eq!(p.ceil(u).to_duration(), d.ceil(unit));
            assert_eq!(p.round(u).to_duration(), d.round(unit));
        }
    }
}
