/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{Duration, Epoch, TimeScale, Unit, Weekday, NANOSECONDS_PER_DAY};

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

use super::rem_euclid_f64;

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
