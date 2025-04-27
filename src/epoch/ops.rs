/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use core::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{
    errors::HifitimeError, Duration, Epoch, Polynomial, TimeScale, Unit, Weekday,
    NANOSECONDS_PER_DAY,
};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

use super::rem_euclid_f64;

#[cfg_attr(feature = "python", pymethods)]
impl Epoch {
    /// Returns the minimum of the two epochs.
    ///
    /// ```
    /// use hifitime::Epoch;
    ///
    /// let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
    /// let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);
    ///
    /// assert_eq!(e0, e1.min(e0));
    /// assert_eq!(e0, e0.min(e1));
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

    /// Returns the maximum of the two epochs.
    ///
    /// ```
    /// use hifitime::Epoch;
    ///
    /// let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
    /// let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);
    ///
    /// assert_eq!(e1, e1.max(e0));
    /// assert_eq!(e1, e0.max(e1));
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

    #[must_use]
    /// Floors this epoch to the closest provided duration
    ///
    /// # Example
    /// ```
    /// use hifitime::{Epoch, TimeUnits};
    ///
    /// let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    /// assert_eq!(
    ///     e.floor(1.hours()),
    ///     Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 0, 0)
    /// );
    ///
    /// let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
    /// assert_eq!(
    ///     e.floor(3.minutes()),
    ///     Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 42, 0)
    /// );
    /// ```
    pub fn floor(&self, duration: Duration) -> Self {
        Self::from_duration(self.duration.floor(duration), self.time_scale)
    }

    #[must_use]
    /// Ceils this epoch to the closest provided duration in the TAI time scale
    ///
    /// # Example
    /// ```
    /// use hifitime::{Epoch, TimeUnits};
    ///
    /// let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    /// assert_eq!(
    ///     e.ceil(1.hours()),
    ///     Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    /// );
    ///
    /// // 45 minutes is a multiple of 3 minutes, hence this result
    /// let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
    /// assert_eq!(
    ///     e.ceil(3.minutes()),
    ///     Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 45, 0)
    /// );
    /// ```
    pub fn ceil(&self, duration: Duration) -> Self {
        Self::from_duration(self.duration.ceil(duration), self.time_scale)
    }

    #[must_use]
    /// Rounds this epoch to the closest provided duration in TAI
    ///
    /// # Example
    /// ```
    /// use hifitime::{Epoch, TimeUnits};
    ///
    /// let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    /// assert_eq!(
    ///     e.round(1.hours()),
    ///     Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    /// );
    /// ```
    pub fn round(&self, duration: Duration) -> Self {
        Self::from_duration(self.duration.round(duration), self.time_scale)
    }

    #[must_use]
    /// Converts this epoch into the time of week, represented as a rolling week counter into that time scale
    /// and the number of nanoseconds elapsed in current week (since closest Sunday midnight).
    /// This is usually how GNSS receivers describe a timestamp.
    pub fn to_time_of_week(&self) -> (u32, u64) {
        let total_nanoseconds = self.duration.total_nanoseconds();
        let weeks = total_nanoseconds / NANOSECONDS_PER_DAY as i128 / Weekday::DAYS_PER_WEEK_I128;
        // elapsed nanoseconds in current week:
        //   remove previously determined nb of weeks
        //   get residual nanoseconds
        let nanoseconds =
            total_nanoseconds - weeks * NANOSECONDS_PER_DAY as i128 * Weekday::DAYS_PER_WEEK_I128;
        (weeks as u32, nanoseconds as u64)
    }

    /// Converts this [Epoch] into targeted [TimeScale] using provided [Polynomial].
    ///
    /// ## Input
    /// - forward: whether this is forward or backward conversion.
    /// For example, using GPST-UTC [Polynomial]
    ///   - GPST->UTC is the forward conversion
    ///   - UTC->GPST is the backward conversion
    /// - reference_epoch: any reference [Epoch] for the provided [Polynomial].  
    /// While we support any time difference, it should remain short in pratice
    /// (a day at most, for precise applications).
    /// - polynomial: that must be valid for this reference [Epoch], used in the equation
    /// a0 + a1*dt + a2*dt² = GPST-UTC for example.
    /// - target: targetted [TimeScale] we will transition to.
    ///
    /// Example:
    /// ```
    /// use hifitime::{Epoch, TimeScale, Polynomial, Unit};
    ///
    /// // random GPST Epoch for forward conversion to UTC
    /// let t_gpst = Epoch::from_gregorian(2020, 01, 01, 0, 0, 0, 0, TimeScale::GPST);
    ///
    /// // Let's say we know the GPST-UTC polynomials for that day,
    /// // They allow precise forward transition from GPST to UTC,
    /// // and precise backward transition from UTC to GPST.
    /// let gpst_utc_polynomials = Polynomial::from_constant_offset_nanoseconds(1.0);
    ///
    /// // This is the reference [Epoch] attached to the publication of these polynomials.
    /// // You should use polynomials that remain valid and were provided recently (usually one day at most).
    /// // Example: polynomials were published 1 hour ago.
    /// let gpst_reference = t_gpst - 1.0 * Unit::Hour;
    ///
    /// // Forward conversion (to UTC) GPST - a0 + a1 *dt + a2*dt² = UTC
    /// let t_utc = t_gpst.precise_timescale_conversion(true, gpst_reference, gpst_utc_polynomials, TimeScale::UTC)
    ///     .unwrap();
    ///
    /// // Verify we did transition to UTC
    /// assert_eq!(t_utc.time_scale, TimeScale::UTC);
    ///
    /// // Verify the resulting [Epoch] is the coarse GPST->UTC transition + fine correction
    /// let reversed = t_utc.to_time_scale(TimeScale::GPST) + 1.0 * Unit::Nanosecond;
    /// assert_eq!(reversed, t_gpst);
    ///
    /// // Apply the backward transition, from t_utc back to t_gpst.
    /// // The timescale conversion works both ways: (from UTC) GPST = UTC + a0 + a1 *dt + a2*dt²
    /// let backwards = t_utc.precise_timescale_conversion(false, gpst_reference, gpst_utc_polynomials, TimeScale::GPST)
    ///     .unwrap();
    ///
    /// assert_eq!(backwards, t_gpst);
    ///
    /// // It is important to understand that your reference point does not have to be in the past.
    /// // The only logic that should prevail is to always minimize interpolation gap.
    /// // In other words, if you can access future interpolation information that would minimize the data gap, they should prevail.
    /// // Example: +30' in the future.
    /// let gpst_reference = t_gpst + 30.0 * Unit::Minute;
    ///
    /// // Forward conversion (to UTC) but using polynomials that were released 1 hour after t_gpst
    /// let t_utc = t_gpst.precise_timescale_conversion(true, gpst_reference, gpst_utc_polynomials, TimeScale::UTC)
    ///     .unwrap();
    ///
    /// // Verifications
    /// assert_eq!(t_utc.time_scale, TimeScale::UTC);
    ///
    /// let reversed = t_utc.to_time_scale(TimeScale::GPST) + 1.0 * Unit::Nanosecond;
    /// assert_eq!(reversed, t_gpst);
    /// ```
    pub fn precise_timescale_conversion(
        &self,
        forward: bool,
        reference_epoch: Self,
        polynomial: Polynomial,
        target: TimeScale,
    ) -> Result<Self, HifitimeError> {
        if self.time_scale == target {
            // Incorrect operation.
            return Err(HifitimeError::SystemTimeError);
        }

        let reference_epoch = reference_epoch.to_time_scale(self.time_scale);

        // supports any interpolation gap. But applications should remain within
        // current week (to the very least..)
        let dt = *self - reference_epoch;
        let correction = polynomial.correction_duration(dt);

        // coarse conversion
        let converted = self.to_time_scale(target);

        // fine correction
        if forward {
            // GPST-UTC = a0+a1+a2
            //      UTC = GPST -a0-a1-a2
            Ok(converted - correction)
        } else {
            // GPST-UTC = a0+a1+a2
            // GPST     = a0+a1+a2 + UTC
            Ok(converted + correction)
        }
    }

    #[must_use]
    /// Returns the weekday in provided time scale **ASSUMING** that the reference epoch of that time scale is a Monday.
    /// You _probably_ do not want to use this. You probably either want `weekday()` or `weekday_utc()`.
    /// Several time scales do _not_ have a reference day that's on a Monday, e.g. BDT.
    pub fn weekday_in_time_scale(&self, time_scale: TimeScale) -> Weekday {
        (rem_euclid_f64(
            self.to_duration_in_time_scale(time_scale)
                .to_unit(Unit::Day),
            Weekday::DAYS_PER_WEEK,
        )
        .floor() as u8)
            .into()
    }

    #[must_use]
    /// Returns weekday (uses the TAI representation for this calculation).
    pub fn weekday(&self) -> Weekday {
        // J1900 was a Monday so we just have to modulo the number of days by the number of days per week.
        // The function call will be optimized away.
        self.weekday_in_time_scale(TimeScale::TAI)
    }

    #[must_use]
    /// Returns weekday in UTC timescale
    pub fn weekday_utc(&self) -> Weekday {
        self.weekday_in_time_scale(TimeScale::UTC)
    }

    #[must_use]
    /// Returns the next weekday.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    /// assert_eq!(epoch.weekday_utc(), Weekday::Saturday);
    /// assert_eq!(epoch.next(Weekday::Sunday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 3));
    /// assert_eq!(epoch.next(Weekday::Monday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 4));
    /// assert_eq!(epoch.next(Weekday::Tuesday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 5));
    /// assert_eq!(epoch.next(Weekday::Wednesday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 6));
    /// assert_eq!(epoch.next(Weekday::Thursday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 7));
    /// assert_eq!(epoch.next(Weekday::Friday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 8));
    /// assert_eq!(epoch.next(Weekday::Saturday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 9));
    /// ```
    pub fn next(&self, weekday: Weekday) -> Self {
        let delta_days = self.weekday() - weekday;
        if delta_days == Duration::ZERO {
            *self + 7 * Unit::Day
        } else {
            *self + delta_days
        }
    }

    #[must_use]
    pub fn next_weekday_at_midnight(&self, weekday: Weekday) -> Self {
        self.next(weekday).with_hms_strict(0, 0, 0)
    }

    #[must_use]
    pub fn next_weekday_at_noon(&self, weekday: Weekday) -> Self {
        self.next(weekday).with_hms_strict(12, 0, 0)
    }

    #[must_use]
    /// Returns the next weekday.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    /// assert_eq!(epoch.previous(Weekday::Friday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 1));
    /// assert_eq!(epoch.previous(Weekday::Thursday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 31));
    /// assert_eq!(epoch.previous(Weekday::Wednesday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 30));
    /// assert_eq!(epoch.previous(Weekday::Tuesday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 29));
    /// assert_eq!(epoch.previous(Weekday::Monday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 28));
    /// assert_eq!(epoch.previous(Weekday::Sunday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 27));
    /// assert_eq!(epoch.previous(Weekday::Saturday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 26));
    /// ```
    pub fn previous(&self, weekday: Weekday) -> Self {
        let delta_days = weekday - self.weekday();
        if delta_days == Duration::ZERO {
            *self - 7 * Unit::Day
        } else {
            *self - delta_days
        }
    }

    #[must_use]
    pub fn previous_weekday_at_midnight(&self, weekday: Weekday) -> Self {
        self.previous(weekday).with_hms_strict(0, 0, 0)
    }

    #[must_use]
    pub fn previous_weekday_at_noon(&self, weekday: Weekday) -> Self {
        self.previous(weekday).with_hms_strict(12, 0, 0)
    }
}

impl Sub for Epoch {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        self.duration - other.to_time_scale(self.time_scale).duration
    }
}

impl SubAssign<Duration> for Epoch {
    fn sub_assign(&mut self, duration: Duration) {
        *self = *self - duration;
    }
}

impl Sub<Duration> for Epoch {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        Self {
            duration: self.duration - duration,
            time_scale: self.time_scale,
        }
    }
}

/// WARNING: For speed, there is a possibility to add seconds directly to an Epoch. These will be added in the time scale the Epoch was initialized in.
/// Using this is _discouraged_ and should only be used if you have facing bottlenecks with the units.
impl Add<f64> for Epoch {
    type Output = Self;

    fn add(self, seconds: f64) -> Self {
        Self {
            duration: self.duration + seconds * Unit::Second,
            time_scale: self.time_scale,
        }
    }
}

impl Add<Duration> for Epoch {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        Self {
            duration: self.duration + duration,
            time_scale: self.time_scale,
        }
    }
}

impl AddAssign<Unit> for Epoch {
    #[allow(clippy::identity_op)]
    fn add_assign(&mut self, unit: Unit) {
        *self = *self + unit * 1;
    }
}

impl SubAssign<Unit> for Epoch {
    #[allow(clippy::identity_op)]
    fn sub_assign(&mut self, unit: Unit) {
        *self = *self - unit * 1;
    }
}

impl Sub<Unit> for Epoch {
    type Output = Self;

    #[allow(clippy::identity_op)]
    fn sub(self, unit: Unit) -> Self {
        Self {
            duration: self.duration - unit * 1,
            time_scale: self.time_scale,
        }
    }
}

impl Add<Unit> for Epoch {
    type Output = Self;

    #[allow(clippy::identity_op)]
    fn add(self, unit: Unit) -> Self {
        Self {
            duration: self.duration + unit * 1,
            time_scale: self.time_scale,
        }
    }
}

impl AddAssign<Duration> for Epoch {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

/// Equality only checks the duration since J1900 match in TAI, because this is how all of the epochs are referenced.
impl PartialEq for Epoch {
    fn eq(&self, other: &Self) -> bool {
        if self.time_scale == other.time_scale {
            self.duration == other.duration
        } else {
            // If one of the two time scales does not include leap seconds,
            // we always convert the time scale with leap seconds into the
            // time scale that does NOT have leap seconds.
            if self.time_scale.uses_leap_seconds() != other.time_scale.uses_leap_seconds() {
                if self.time_scale.uses_leap_seconds() {
                    self.to_time_scale(other.time_scale).duration == other.duration
                } else {
                    self.duration == other.to_time_scale(self.time_scale).duration
                }
            } else {
                // Otherwise it does not matter
                self.duration == other.to_time_scale(self.time_scale).duration
            }
        }
    }
}

impl PartialOrd for Epoch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.duration
                .cmp(&other.to_time_scale(self.time_scale).duration),
        )
    }
}

impl Ord for Epoch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.duration
            .cmp(&other.to_time_scale(self.time_scale).duration)
    }
}

impl Hash for Epoch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.duration.hash(state);
        self.time_scale.hash(state);
    }
}
