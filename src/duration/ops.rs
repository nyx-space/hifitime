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

use crate::{NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND, NANOSECONDS_PER_SECOND};

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

                Duration::from_total_nanoseconds(total_ns as i128)
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
impl_ops_for_type!(i128);

impl Mul<i128> for Duration {
    type Output = Duration;
    fn mul(self, q: i128) -> Self::Output {
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

    #[allow(clippy::absurd_extreme_comparisons)]
    fn add(self, rhs: Self) -> Duration {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_add(rhs.zeptoseconds),
        }
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_sub(rhs.zeptoseconds),
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

    #[must_use]
    fn neg(self) -> Self::Output {
        Self {
            zeptoseconds: self.zeptoseconds.saturating_neg(),
        }
    }
}
