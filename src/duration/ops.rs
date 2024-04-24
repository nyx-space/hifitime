/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the operations on Duration.

use crate::{
    NANOSECONDS_PER_CENTURY, NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND,
    NANOSECONDS_PER_SECOND,
};

use super::{Duration, Freq, Frequencies, TimeUnits, Unit};

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

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

impl_ops_for_type!(f64);
impl_ops_for_type!(i64);

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
        match self.centuries.checked_sub(rhs.centuries) {
            None => {
                // Underflowed, so we've hit the min
                return Self::MIN;
            }
            Some(centuries) => {
                self.centuries = centuries;
            }
        }

        match self.nanoseconds.checked_sub(rhs.nanoseconds) {
            None => {
                // Decrease the number of centuries, and realign
                match self.centuries.checked_sub(1) {
                    Some(centuries) => {
                        self.centuries = centuries;
                        self.nanoseconds += NANOSECONDS_PER_CENTURY - rhs.nanoseconds;
                    }
                    None => {
                        // We're at the min number of centuries already, and we have extra nanos, so we're saturated the duration limit
                        return Self::MIN;
                    }
                };
            }
            Some(nanos) => self.nanoseconds = nanos,
        };

        self.normalize();
        self
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
