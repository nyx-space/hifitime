/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::prelude::*;

#[deprecated(since = "3.5.0", note = "TimeSystem has been renamed to TimeScale")]
pub type TimeSystem = TimeScale;

impl Duration {
    #[must_use]
    #[deprecated(note = "Prefer to_seconds()", since = "3.5.0")]
    pub fn in_seconds(&self) -> f64 {
        self.to_seconds()
    }

    /// Returns the value of this duration in the requested unit.
    #[must_use]
    #[deprecated(note = "Prefer to_unit()", since = "3.5.0")]
    pub fn in_unit(&self, unit: Unit) -> f64 {
        self.to_unit(unit)
    }
}

impl Epoch {
    #[must_use]
    /// Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds.
    /// For the leap seconds _and_ the scaling in "prehistoric" times from 1960 to 1972, use `leap_seconds()`.
    #[deprecated(note = "Prefer leap_seconds_iers or leap_seconds", since = "3.4.0")]
    pub fn get_num_leap_seconds(&self) -> i32 {
        self.leap_seconds_iers()
    }

    #[must_use]
    #[deprecated(note = "Prefer as_tdb_duration", since = "3.4.0")]
    /// Returns the duration since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)
    pub fn as_tdb_duration_since_j2000(&self) -> Duration {
        self.as_tdb_duration()
    }

    #[must_use]
    #[deprecated(note = "Prefer as_et_duration", since = "3.4.0")]
    /// Returns the duration since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn as_et_duration_since_j2000(&self) -> Duration {
        self.as_et_duration()
    }
}
