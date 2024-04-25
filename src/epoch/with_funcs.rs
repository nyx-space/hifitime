/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{Duration, Epoch};

impl Epoch {
    /// Returns a copy of self where the time is set to the provided hours, minutes, seconds
    /// Invalid number of hours, minutes, and seconds will overflow into their higher unit.
    /// Warning: this does _not_ set the subdivisions of second to zero.
    pub fn with_hms(&self, hours: u64, minutes: u64, seconds: u64) -> Self {
        let (sign, days, _, _, _, milliseconds, microseconds, nanoseconds) =
            self.duration.decompose();
        Self::from_duration(
            Duration::compose(
                sign,
                days,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
            self.time_scale,
        )
    }

    /// Returns a copy of self where the hours, minutes, seconds is set to the time of the provided epoch but the
    /// sub-second parts are kept from the current epoch.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    /// let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    /// let other = other_utc.to_time_scale(TimeScale::TDB);
    ///
    /// assert_eq!(
    ///     epoch.with_hms_from(other),
    ///     Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 13)
    /// );
    /// ```
    pub fn with_hms_from(&self, other: Self) -> Self {
        let (sign, days, _, _, _, milliseconds, microseconds, nanoseconds) =
            self.duration.decompose();
        // Shadow other with the provided other epoch but in the correct time scale.
        let other = other.to_time_scale(self.time_scale);
        Self::from_duration(
            Duration::compose(
                sign,
                days,
                other.hours(),
                other.minutes(),
                other.seconds(),
                milliseconds,
                microseconds,
                nanoseconds,
            ),
            self.time_scale,
        )
    }

    /// Returns a copy of self where all of the time components (hours, minutes, seconds, and sub-seconds) are set to the time of the provided epoch.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    /// let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    /// // If the other Epoch is in another time scale, it does not matter, it will be converted to the correct time scale.
    /// let other = other_utc.to_time_scale(TimeScale::TDB);
    ///
    /// assert_eq!(
    ///     epoch.with_time_from(other),
    ///     Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 23)
    /// );
    /// ```
    pub fn with_time_from(&self, other: Self) -> Self {
        // Grab days from self
        let (sign, days, _, _, _, _, _, _) = self.duration.decompose();

        // Grab everything else from other
        let (_, _, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) =
            other.to_duration_in_time_scale(self.time_scale).decompose();

        Self::from_duration(
            Duration::compose(
                sign,
                days,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
            self.time_scale,
        )
    }

    /// Returns a copy of self where the time is set to the provided hours, minutes, seconds
    /// Invalid number of hours, minutes, and seconds will overflow into their higher unit.
    /// Warning: this will set the subdivisions of seconds to zero.
    pub fn with_hms_strict(&self, hours: u64, minutes: u64, seconds: u64) -> Self {
        let (sign, days, _, _, _, _, _, _) = self.duration.decompose();
        Self::from_duration(
            Duration::compose(sign, days, hours, minutes, seconds, 0, 0, 0),
            self.time_scale,
        )
    }

    /// Returns a copy of self where the time is set to the time of the other epoch but the subseconds are set to zero.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    /// let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    /// let other = other_utc.to_time_scale(TimeScale::TDB);
    ///
    /// assert_eq!(
    ///     epoch.with_hms_strict_from(other),
    ///     Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 0)
    /// );
    /// ```
    pub fn with_hms_strict_from(&self, other: Self) -> Self {
        let (sign, days, _, _, _, _, _, _) = self.duration.decompose();
        let other = other.to_time_scale(self.time_scale);
        Self::from_duration(
            Duration::compose(
                sign,
                days,
                other.hours(),
                other.minutes(),
                other.seconds(),
                0,
                0,
                0,
            ),
            self.time_scale,
        )
    }
}
