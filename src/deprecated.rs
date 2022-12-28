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
        self.to_tdb_duration()
    }

    #[must_use]
    #[deprecated(note = "Prefer as_et_duration", since = "3.4.0")]
    /// Returns the duration since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn as_et_duration_since_j2000(&self) -> Duration {
        self.to_et_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_duration(&self) -> Duration {
        self.to_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_duration_in_time_scale(&self, time_scale: TimeScale) -> Duration {
        self.to_duration_in_time_scale(time_scale)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_duration_since_j1900(&self) -> Duration {
        self.to_duration_since_j1900()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_duration_since_j1900_in_time_scale(&self, time_scale: TimeScale) -> Duration {
        self.to_duration_since_j1900_in_time_scale(time_scale)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tai_seconds(&self) -> f64 {
        self.to_tai_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub const fn as_tai_duration(&self) -> Duration {
        self.to_tai_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tai(&self, unit: Unit) -> f64 {
        self.to_tai(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tai_days(&self) -> f64 {
        self.to_tai_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_utc_seconds(&self) -> f64 {
        self.to_utc_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_utc_duration(&self) -> Duration {
        self.to_utc_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_utc(&self, unit: Unit) -> f64 {
        self.to_utc(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_utc_days(&self) -> f64 {
        self.to_utc_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_tai_days(&self) -> f64 {
        self.to_mjd_tai_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_tai_seconds(&self) -> f64 {
        self.to_mjd_tai_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_tai(&self, unit: Unit) -> f64 {
        self.to_mjd_tai(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_utc_days(&self) -> f64 {
        self.to_mjd_utc_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_utc(&self, unit: Unit) -> f64 {
        self.to_mjd_utc(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_utc_seconds(&self) -> f64 {
        self.to_mjd_utc_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tai_days(&self) -> f64 {
        self.to_jde_tai_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tai(&self, unit: Unit) -> f64 {
        self.to_jde_tai(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tai_duration(&self) -> Duration {
        self.to_jde_tai_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tai_seconds(&self) -> f64 {
        self.to_jde_tai_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_utc_days(&self) -> f64 {
        self.to_jde_utc_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_utc_duration(&self) -> Duration {
        self.to_jde_utc_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_utc_seconds(&self) -> f64 {
        self.to_jde_utc_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tt_seconds(&self) -> f64 {
        self.to_tt_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tt_duration(&self) -> Duration {
        self.to_tt_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tt_days(&self) -> f64 {
        self.to_tt_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tt_centuries_j2k(&self) -> f64 {
        self.to_tt_centuries_j2k()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tt_since_j2k(&self) -> Duration {
        self.to_tt_since_j2k()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tt_days(&self) -> f64 {
        self.to_jde_tt_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tt_duration(&self) -> Duration {
        self.to_jde_tt_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_tt_days(&self) -> f64 {
        self.to_mjd_tt_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_mjd_tt_duration(&self) -> Duration {
        self.to_mjd_tt_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gpst_seconds(&self) -> f64 {
        self.to_gpst_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gpst_duration(&self) -> Duration {
        self.to_gpst_duration()
    }

    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gpst_nanoseconds(&self) -> Result<u64, Errors> {
        self.to_gpst_nanoseconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gpst_days(&self) -> f64 {
        self.to_gpst_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_unix(&self, unit: Unit) -> f64 {
        self.to_unix(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_unix_seconds(&self) -> f64 {
        self.to_unix_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_unix_milliseconds(&self) -> f64 {
        self.to_unix_milliseconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_unix_days(&self) -> f64 {
        self.to_unix_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_et_seconds(&self) -> f64 {
        self.to_et_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_et_duration_since_j1900(&self) -> Duration {
        self.to_et_duration_since_j1900()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_et_duration(&self) -> Duration {
        self.to_et_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tdb_duration(&self) -> Duration {
        self.to_tdb_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tdb_seconds(&self) -> f64 {
        self.to_tdb_seconds()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tdb_duration_since_j1900(&self) -> Duration {
        self.to_tdb_duration_since_j1900()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_et_days(&self) -> f64 {
        self.to_jde_et_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_et_duration(&self) -> Duration {
        self.to_jde_et_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_et(&self, unit: Unit) -> f64 {
        self.to_jde_et(unit)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tdb_duration(&self) -> Duration {
        self.to_jde_tdb_duration()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_jde_tdb_days(&self) -> f64 {
        self.to_jde_tdb_days()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tdb_days_since_j2000(&self) -> f64 {
        self.to_tdb_days_since_j2000()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_tdb_centuries_since_j2000(&self) -> f64 {
        self.to_tdb_centuries_since_j2000()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_et_days_since_j2000(&self) -> f64 {
        self.to_et_days_since_j2000()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_et_centuries_since_j2000(&self) -> f64 {
        self.to_et_centuries_since_j2000()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gregorian_utc(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        self.to_gregorian_utc()
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gregorian_tai(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        self.to_gregorian_tai()
    }
}

#[cfg(feature = "std")]
impl Epoch {
    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gregorian_utc_str(&self) -> String {
        format!("{}", self)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gregorian_tai_str(&self) -> String {
        format!("{:x}", self)
    }

    #[must_use]
    #[deprecated(
        note = "Prefix for this function is now `to_` instead of `as_`.",
        since = "3.5.0"
    )]
    pub fn as_gregorian_str(&self, ts: TimeScale) -> String {
        self.to_gregorian_str(ts)
    }
}
