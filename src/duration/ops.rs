/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the operations on Duration.

use crate::{
    NANOSECONDS_PER_CENTURY, NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND,
    NANOSECONDS_PER_SECOND,
};

use super::{Duration, Freq, Frequencies, TimeUnits, Unit};

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

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

impl_ops_for_type!(f64);
impl_ops_for_type!(i64);

impl Mul<i64> for Duration {
    type Output = Duration;

    /// Scales this duration by `q`, saturating at [`Duration::MIN`] /
    /// [`Duration::MAX`].
    fn mul(self, q: i64) -> Self::Output {
        Duration::from_total_nanoseconds(self.total_nanoseconds().saturating_mul(i128::from(q)))
    }
}

// Shared by Duration's Mul<f64> and Div<f64> below.

/// Decomposes a finite, nonzero f64's magnitude into `mantissa * 2^exponent`
/// (`mantissa` fits in 53 bits). Sign is not included; use `q.is_sign_negative()`.
#[inline]
fn decompose_f64(q: f64) -> (u64, i32) {
    let bits = q.to_bits();
    let raw_exponent = ((bits >> 52) & 0x7FF) as i32;
    let raw_mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
    if raw_exponent == 0 {
        (raw_mantissa, 1 - 1023 - 52) // subnormal: no implicit leading bit
    } else {
        (raw_mantissa | (1 << 52), raw_exponent - 1023 - 52) // normal
    }
}

/// Builds a `Duration` from an unsigned nanosecond magnitude and a sign,
/// saturating if it doesn't fit.
#[inline]
fn duration_from_magnitude(magnitude: u128, negative: bool) -> Duration {
    if magnitude < NANOSECONDS_PER_CENTURY as u128 {
        let ns = magnitude as u64;
        return if !negative {
            Duration::from_parts(0, ns)
        } else if ns == 0 {
            Duration::ZERO
        } else {
            Duration::from_parts(-1, NANOSECONDS_PER_CENTURY - ns)
        };
    }
    if magnitude > i128::MAX as u128 {
        return if negative {
            Duration::MIN
        } else {
            Duration::MAX
        };
    }
    let signed = magnitude as i128;
    Duration::from_total_nanoseconds(if negative { -signed } else { signed })
}

/// `a << shift` as `(hi, lo)`, i.e. `hi * 2^128 + lo`. `shift` must be < 256.
#[inline]
fn widening_shl(a: u128, shift: u32) -> (u128, u128) {
    debug_assert!(shift < 256, "shift past 256 bits");
    if shift == 0 {
        (0, a)
    } else if shift < 128 {
        (a >> (128 - shift), a << shift)
    } else {
        (a << (shift - 128), 0)
    }
}

/// Divides `hi * 2^128 + lo` by `divisor`, truncating. Long division 64 bits at
/// a time, so each step's working value fits in a `u128`.
///
/// The quotient must fit in a `u128`, i.e. `hi < divisor`; the accumulator shift
/// drops high bits silently otherwise.
#[inline]
fn divide_wide(hi: u128, lo: u128, divisor: u64) -> u128 {
    debug_assert!(divisor != 0, "divide by zero");
    debug_assert!(hi < divisor as u128, "quotient exceeds u128");
    let divisor = divisor as u128;
    let mut remainder = 0u128;
    let mut quotient = 0u128;
    for word in [(hi >> 64) as u64, hi as u64, (lo >> 64) as u64, lo as u64] {
        let chunk = (remainder << 64) | word as u128;
        quotient = (quotient << 64) | (chunk / divisor);
        remainder = chunk % divisor;
    }
    quotient
}

impl Mul<f64> for Duration {
    type Output = Duration;

    /// Scales this duration by `q`.
    ///
    /// `q` is decomposed into its exact `mantissa * 2^exponent` form and applied
    /// in wide integer arithmetic, so the scaling is exact. Only the result is
    /// truncated toward zero to whole nanoseconds, then saturated at
    /// [`Duration::MIN`] / [`Duration::MAX`].
    ///
    /// # Non-finite `q`
    ///
    /// Never panics, unlike [`Duration::from_seconds`] and the other `f64`
    /// constructors.
    ///
    /// | `q` | result |
    /// |---|---|
    /// | `±∞` | [`Duration::MAX`] / [`Duration::MIN`] by sign |
    /// | `±0.0` | [`Duration::ZERO`] |
    /// | `NaN` | [`Duration::ZERO`] |
    ///
    /// A [`Duration`] cannot represent NaN, and a NaN has no meaningful sign to
    /// saturate towards. `Duration::ZERO` times an infinity is `0 × ∞`, also
    /// [`Duration::ZERO`].
    #[inline]
    fn mul(self, q: f64) -> Self::Output {
        if q.is_nan() {
            return Duration::ZERO;
        }

        let numerator = self.total_nanoseconds();
        if numerator == 0 || q == 0.0 {
            return Duration::ZERO;
        }

        let result_negative = (numerator < 0) != q.is_sign_negative();
        let saturate = || {
            if result_negative {
                Duration::MIN
            } else {
                Duration::MAX
            }
        };

        if q.is_infinite() {
            return saturate();
        }

        let (mantissa, exponent) = decompose_f64(q);
        let a = numerator.unsigned_abs();

        // Widening multiply of a * mantissa, as `hi * 2^128 + lo`.
        let a_lo = a as u64 as u128;
        let a_hi = a >> 64;
        let m = mantissa as u128;
        let p_lo = a_lo * m; // < 2^117, always fits
        let p_hi = a_hi.saturating_mul(m);
        let (lo, carry) = p_lo.overflowing_add((p_hi as u64 as u128) << 64);
        let hi = (p_hi >> 64) + u128::from(carry);

        let magnitude = if exponent < 0 {
            let shift = (-exponent) as u32;
            if shift >= 256 {
                0
            } else if shift >= 128 {
                hi >> (shift - 128)
            } else if hi >= (1u128 << shift) {
                return saturate();
            } else {
                (hi << (128 - shift)) | (lo >> shift)
            }
        } else {
            let shift = exponent as u32;
            if hi != 0 || shift >= 128 || lo > (u128::MAX >> shift) {
                return saturate();
            }
            lo << shift
        };

        duration_from_magnitude(magnitude, result_negative)
    }
}

impl Div<f64> for Duration {
    type Output = Duration;

    /// Divides this duration by `q`.
    ///
    /// Same exact `mantissa * 2^exponent` decomposition as
    /// [`Mul<f64>`](Duration::mul), dividing by the mantissa rather than
    /// multiplying. The result is truncated toward zero to whole nanoseconds,
    /// then saturated at [`Duration::MIN`] / [`Duration::MAX`].
    ///
    /// # Non-finite `q`
    ///
    /// Never panics; dividing by zero saturates rather than trapping.
    ///
    /// | `q` | result |
    /// |---|---|
    /// | `±∞` | [`Duration::ZERO`] |
    /// | `±0.0` | [`Duration::MAX`] / [`Duration::MIN`] by sign |
    /// | `NaN` | [`Duration::ZERO`] |
    ///
    /// `Duration::ZERO` divided by zero is `0 / 0`, also [`Duration::ZERO`].
    #[inline]
    fn div(self, q: f64) -> Self::Output {
        // Dividing by an infinity tends to zero; a NaN has no representation.
        if !q.is_finite() {
            return Duration::ZERO;
        }

        let numerator = self.total_nanoseconds();
        if numerator == 0 {
            return Duration::ZERO;
        }

        let result_negative = (numerator < 0) != q.is_sign_negative();

        if q == 0.0 {
            return if result_negative {
                Duration::MIN
            } else {
                Duration::MAX
            };
        }

        let (divisor, exponent) = decompose_f64(q);
        let a = numerator.unsigned_abs();

        let magnitude = if exponent >= 0 {
            let shift = exponent as u32;
            (if shift >= 128 { 0 } else { a >> shift }) / (divisor as u128)
        } else {
            let shift = (-exponent) as u32;
            let bits_needed = (u128::BITS - a.leading_zeros()) + shift;
            let divisor_bits = u64::BITS - divisor.leading_zeros();
            if bits_needed > divisor_bits + 126 {
                return if result_negative {
                    Duration::MIN
                } else {
                    Duration::MAX
                };
            }
            let (hi, lo) = widening_shl(a, shift);
            if hi == 0 {
                lo / (divisor as u128)
            } else {
                divide_wide(hi, lo, divisor)
            }
        };

        duration_from_magnitude(magnitude, result_negative)
    }
}

impl Div<i64> for Duration {
    type Output = Duration;

    /// Divides this duration by `q`, truncating toward zero.
    ///
    /// Dividing by zero saturates at [`Duration::MAX`] / [`Duration::MIN`] by
    /// this duration's sign, and `Duration::ZERO / 0` is [`Duration::ZERO`].
    fn div(self, q: i64) -> Self::Output {
        let numerator = self.total_nanoseconds();
        if q == 0 {
            return match numerator.signum() {
                1 => Duration::MAX,
                -1 => Duration::MIN,
                _ => Duration::ZERO,
            };
        }
        Duration::from_total_nanoseconds(numerator.saturating_div(i128::from(q)))
    }
}

impl MulAssign<f64> for Duration {
    fn mul_assign(&mut self, q: f64) {
        *self = *self * q;
    }
}

impl DivAssign<f64> for Duration {
    fn div_assign(&mut self, q: f64) {
        *self = *self / q;
    }
}

impl MulAssign<i64> for Duration {
    fn mul_assign(&mut self, q: i64) {
        *self = *self * q;
    }
}

impl DivAssign<i64> for Duration {
    fn div_assign(&mut self, q: i64) {
        *self = *self / q;
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
    #[allow(clippy::absurd_extreme_comparisons)]
    fn add(mut self, mut rhs: Self) -> Duration {
        // Ensure that the durations are normalized to avoid extra logic to handle under/overflows
        self.normalize();
        rhs.normalize();

        // Check that the addition fits in an i16
        match self.centuries.checked_add(rhs.centuries) {
            None => {
                // Overflowed, so we've hit the bound.
                if self.centuries < 0 {
                    // We've hit the negative bound, so return MIN.
                    return Self::MIN;
                } else {
                    // We've hit the positive bound, so return MAX.
                    return Self::MAX;
                }
            }
            Some(centuries) => {
                self.centuries = centuries;
            }
        }

        if self.centuries == Self::MIN.centuries && self.nanoseconds < Self::MIN.nanoseconds {
            // Then we do the operation backward
            match self
                .nanoseconds
                .checked_sub(NANOSECONDS_PER_CENTURY - rhs.nanoseconds)
            {
                Some(nanos) => self.nanoseconds = nanos,
                None => {
                    self.centuries += 1; // Safe because we're at the MIN
                    self.nanoseconds = rhs.nanoseconds
                }
            }
        } else {
            match self.nanoseconds.checked_add(rhs.nanoseconds) {
                Some(nanoseconds) => self.nanoseconds = nanoseconds,
                None => {
                    // Rare case where somehow the input data was not normalized. So let's normalize it and call add again.
                    let mut rhs = rhs;
                    rhs.normalize();

                    match self.centuries.checked_add(rhs.centuries) {
                        None => return Self::MAX,
                        Some(centuries) => self.centuries = centuries,
                    };
                    // Now it will fit!
                    self.nanoseconds += rhs.nanoseconds;
                }
            }
        }

        self.normalize();
        self
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
    fn sub(mut self, mut rhs: Self) -> Self {
        // Ensure that the durations are normalized to avoid extra logic to handle under/overflows
        self.normalize();
        rhs.normalize();

        // Widen the century subtraction so that its direction is still known
        // when the result exceeds the Duration bounds.
        let mut centuries = i32::from(self.centuries) - i32::from(rhs.centuries);
        let mut nanoseconds = match self.nanoseconds.checked_sub(rhs.nanoseconds) {
            Some(nanoseconds) => nanoseconds,
            None => {
                centuries -= 1;
                NANOSECONDS_PER_CENTURY - (rhs.nanoseconds - self.nanoseconds)
            }
        };

        // MAX is the only normalized value whose nanoseconds equal a full
        // century. Carry it before checking the widened century bounds.
        if nanoseconds == NANOSECONDS_PER_CENTURY {
            centuries += 1;
            nanoseconds = 0;
        }

        if centuries > i32::from(i16::MAX) {
            Self::MAX
        } else if centuries < i32::from(i16::MIN) {
            Self::MIN
        } else {
            Self {
                centuries: centuries as i16,
                nanoseconds,
            }
        }
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

impl Neg for Duration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self == Self::MIN {
            Self::MAX
        } else if self == Self::MAX {
            Self::MIN
        } else {
            let centuries = -i32::from(self.centuries) - 1;
            let nanoseconds = NANOSECONDS_PER_CENTURY - self.nanoseconds;
            Self::from_parts(
                i16::try_from(centuries).expect("negated duration centuries must fit in i16"),
                nanoseconds,
            )
        }
    }
}
