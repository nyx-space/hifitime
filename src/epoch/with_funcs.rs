/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{parts::DurationParts, Epoch};

impl Epoch {
    /// Returns a copy of self where the time is set to the provided hours, minutes, seconds
    /// Invalid number of hours, minutes, and seconds will overflow into their higher unit.
    /// Warning: this does _not_ set the subdivisions of second to zero.
    pub fn with_hms(&self, hours: u64, minutes: u64, seconds: u64) -> Self {
        let mut parts = self.duration.decompose();
        parts.hours = hours.into();
        parts.minutes = minutes.into();
        parts.seconds = seconds.into();
        Self::from_duration(parts.into(), self.time_scale)
    }

    /// Returns a copy of self where the hours, minutes, seconds is set to the time of the provided epoch but the
    /// sub-second parts are kept from the current epoch.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc(2022, 12, 1, 10, 11, 12, 13);
    /// let other_utc = Epoch::from_gregorian_utc(2024, 12, 1, 20, 21, 22, 23);
    /// let other = other_utc.to_time_scale(TimeScale::TDB);
    ///
    /// assert_eq!(
    ///     epoch.with_hms_from(other),
    ///     Epoch::from_gregorian_utc(2022, 12, 1, 20, 21, 22, 13)
    /// );
    /// ```
    pub fn with_hms_from(&self, other: Self) -> Self {
        let mut parts = self.duration.decompose();
        // Shadow other with the provided other epoch but in the correct time scale.
        let other = other.to_time_scale(self.time_scale);
        parts.hours = other.hours().into();
        parts.minutes = other.minutes().into();
        parts.seconds = other.seconds().into();
        Self::from_duration(parts.into(), self.time_scale)
    }

    /// Returns a copy of self where all of the time components (hours, minutes, seconds, and sub-seconds) are set to the time of the provided epoch.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc(2022, 12, 1, 10, 11, 12, 13);
    /// let other_utc = Epoch::from_gregorian_utc(2024, 12, 1, 20, 21, 22, 23);
    /// // If the other Epoch is in another time scale, it does not matter, it will be converted to the correct time scale.
    /// let other = other_utc.to_time_scale(TimeScale::TDB);
    ///
    /// assert_eq!(
    ///     epoch.with_time_from(other),
    ///     Epoch::from_gregorian_utc(2022, 12, 1, 20, 21, 22, 23)
    /// );
    /// ```
    pub fn with_time_from(&self, other: Self) -> Self {
        // Grab days from self
        let parts = self.duration.decompose();

        // Grab everything else from other
        let mut others_parts = other.to_duration_in_time_scale(self.time_scale).decompose();
        others_parts.sign = parts.sign;
        others_parts.days = parts.days;

        Self::from_duration(others_parts.into(), self.time_scale)
    }

    /// Returns a copy of self where the time is set to the provided hours, minutes, seconds
    /// Invalid number of hours, minutes, and seconds will overflow into their higher unit.
    /// Warning: this will set the subdivisions of seconds to zero.
    pub fn with_hms_strict(&self, hours: u64, minutes: u64, seconds: u64) -> Self {
        let parts = self.duration.decompose();
        let new_duration = DurationParts::builder()
            .sign(parts.sign)
            .days(parts.days)
            .hours(hours.into())
            .minutes(minutes.into())
            .seconds(seconds.into())
            .build();
        Self::from_duration(new_duration.into(), self.time_scale)
    }

    /// Returns a copy of self where the time is set to the time of the other epoch but the subseconds are set to zero.
    ///
    /// ```
    /// use hifitime::prelude::*;
    ///
    /// let epoch = Epoch::from_gregorian_utc(2022, 12, 1, 10, 11, 12, 13);
    /// let other_utc = Epoch::from_gregorian_utc(2024, 12, 1, 20, 21, 22, 23);
    /// let other = other_utc.to_time_scale(TimeScale::TDB);
    ///
    /// assert_eq!(
    ///     epoch.with_hms_strict_from(other),
    ///     Epoch::from_gregorian_utc(2022, 12, 1, 20, 21, 22, 0)
    /// );
    /// ```
    pub fn with_hms_strict_from(&self, other: Self) -> Self {
        let parts = self.duration.decompose();
        let other = other.to_time_scale(self.time_scale);
        let new_duration = DurationParts::builder()
            .sign(parts.sign)
            .days(parts.days)
            .hours(other.hours().into())
            .minutes(other.minutes().into())
            .seconds(other.seconds().into())
            .build();
        Self::from_duration(new_duration.into(), self.time_scale)
    }
}
