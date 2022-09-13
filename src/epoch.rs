/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::duration::{Duration, Unit};
use crate::{
    Errors, TimeSystem, DAYS_GPS_TAI_OFFSET, DAYS_PER_YEAR_NLD, ET_EPOCH_S, J1900_OFFSET,
    J2000_TO_J1900_DURATION, MJD_OFFSET, SECONDS_GPS_TAI_OFFSET, SECONDS_GPS_TAI_OFFSET_I64,
    SECONDS_PER_DAY, UNIX_REF_EPOCH,
};
use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(feature = "std")]
use crate::ParsingErrors;

#[cfg(feature = "std")]
use super::regex::Regex;
#[cfg(feature = "std")]
use super::serde::{de, Deserialize, Deserializer};
#[cfg(feature = "std")]
use std::str::FromStr;
#[cfg(feature = "std")]
use std::time::SystemTime;

#[cfg(not(feature = "std"))]
use num_traits::Float;

const TT_OFFSET_MS: i64 = 32_184;
const ET_OFFSET_US: i64 = 32_184_935;

/// NAIF leap second kernel data for M_0 used to calculate the mean anomaly of the heliocentric orbit of the Earth-Moon barycenter.
pub const NAIF_M0: f64 = 6.239996;
/// NAIF leap second kernel data for M_1 used to calculate the mean anomaly of the heliocentric orbit of the Earth-Moon barycenter.
pub const NAIF_M1: f64 = 1.99096871e-7;
/// NAIF leap second kernel data for EB used to calculate the eccentric anomaly of the heliocentric orbit of the Earth-Moon barycenter.
pub const NAIF_EB: f64 = 1.671e-2;
/// NAIF leap second kernel data used to calculate the difference between ET and TAI.
pub const NAIF_K: f64 = 1.657e-3;

/// List of leap seconds from https://www.ietf.org/timezones/data/leap-seconds.list .
/// This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
/// The unannoucned leap seconds come from dat.c in the SOFA library.
const LEAP_SECONDS: [(f64, f64, bool); 42] = [
    (1_893_369_600.0, 1.417818, false), // SOFA: 01 Jan 1960
    (1_924_992_000.0, 1.422818, false), // SOFA: 01 Jan 1961
    (1_943_308_800.0, 1.372818, false), // SOFA: 01 Aug 1961
    (1_956_528_000.0, 1.845858, false), // SOFA: 01 Jan 1962
    (2_014_329_600.0, 1.945858, false), // SOFA: 01 Jan 1963
    (2_019_600_000.0, 3.24013, false),  // SOFA: 01 Jan 1964
    (2_027_462_400.0, 3.34013, false),  // SOFA: 01 Apr 1964
    (2_040_681_600.0, 3.44013, false),  // SOFA: 01 Sep 1964
    (2_051_222_400.0, 3.54013, false),  // SOFA: 01 Jan 1965
    (2_056_320_000.0, 3.64013, false),  // SOFA: 01 Mar 1965
    (2_066_860_800.0, 3.74013, false),  // SOFA: 01 Jul 1965
    (2_072_217_600.0, 3.84013, false),  // SOFA: 01 Sep 1965
    (2_082_758_400.0, 4.31317, false),  // SOFA: 01 Jan 1966
    (2_148_508_800.0, 4.21317, false),  // SOFA: 01 Feb 1968
    (2_272_060_800.0, 10.0, true),      // IERS: 01 Jan 1972
    (2_287_785_600.0, 11.0, true),      // IERS: 01 Jul 1972
    (2_303_683_200.0, 12.0, true),      // IERS: 01 Jan 1973
    (2_335_219_200.0, 13.0, true),      // IERS: 01 Jan 1974
    (2_366_755_200.0, 14.0, true),      // IERS: 01 Jan 1975
    (2_398_291_200.0, 15.0, true),      // IERS: 01 Jan 1976
    (2_429_913_600.0, 16.0, true),      // IERS: 01 Jan 1977
    (2_461_449_600.0, 17.0, true),      // IERS: 01 Jan 1978
    (2_492_985_600.0, 18.0, true),      // IERS: 01 Jan 1979
    (2_524_521_600.0, 19.0, true),      // IERS: 01 Jan 1980
    (2_571_782_400.0, 20.0, true),      // IERS: 01 Jul 1981
    (2_603_318_400.0, 21.0, true),      // IERS: 01 Jul 1982
    (2_634_854_400.0, 22.0, true),      // IERS: 01 Jul 1983
    (2_698_012_800.0, 23.0, true),      // IERS: 01 Jul 1985
    (2_776_982_400.0, 24.0, true),      // IERS: 01 Jan 1988
    (2_840_140_800.0, 25.0, true),      // IERS: 01 Jan 1990
    (2_871_676_800.0, 26.0, true),      // IERS: 01 Jan 1991
    (2_918_937_600.0, 27.0, true),      // IERS: 01 Jul 1992
    (2_950_473_600.0, 28.0, true),      // IERS: 01 Jul 1993
    (2_982_009_600.0, 29.0, true),      // IERS: 01 Jul 1994
    (3_029_443_200.0, 30.0, true),      // IERS: 01 Jan 1996
    (3_076_704_000.0, 31.0, true),      // IERS: 01 Jul 1997
    (3_124_137_600.0, 32.0, true),      // IERS: 01 Jan 1999
    (3_345_062_400.0, 33.0, true),      // IERS: 01 Jan 2006
    (3_439_756_800.0, 34.0, true),      // IERS: 01 Jan 2009
    (3_550_089_600.0, 35.0, true),      // IERS: 01 Jul 2012
    (3_644_697_600.0, 36.0, true),      // IERS: 01 Jul 2015
    (3_692_217_600.0, 37.0, true),      // IERS: 01 Jan 2017
];

/// Years when January had the leap second
const JANUARY_YEARS: [i32; 17] = [
    1972, 1973, 1974, 1975, 1976, 1977, 1978, 1979, 1980, 1988, 1990, 1991, 1996, 1999, 2006, 2009,
    2017,
];

/// Years when July had the leap second
const JULY_YEARS: [i32; 11] = [
    1972, 1981, 1982, 1983, 1985, 1992, 1993, 1994, 1997, 2012, 2015,
];

const USUAL_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Defines an Epoch in TAI (temps atomique international) in seconds past 1900 January 01 at midnight (like the Network Time Protocol).
///
/// Refer to the appropriate functions for initializing this Epoch from different time systems or representations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Epoch(pub(crate) Duration);

impl Sub for Epoch {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        self.0 - other.0
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
        Self(self.0 - duration)
    }
}

impl Add<f64> for Epoch {
    type Output = Self;

    /// WARNING: For speed, there is a possibility to add seconds directly to an Epoch.
    /// Using this is _discouraged_ and should only be used if you have facing bottlenecks with the units.
    fn add(self, seconds: f64) -> Self {
        Self((self.0.in_seconds() + seconds) * Unit::Second)
    }
}

impl Add<Duration> for Epoch {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        Self(self.0 + duration)
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
        Self(self.0 - unit * 1)
    }
}

impl Add<Unit> for Epoch {
    type Output = Self;

    #[allow(clippy::identity_op)]
    fn add(self, unit: Unit) -> Self {
        Self(self.0 + unit * 1)
    }
}

impl AddAssign<Duration> for Epoch {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
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
    /// Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds.
    pub fn leap_seconds_iers(&self) -> i32 {
        match self.leap_seconds(true) {
            Some(v) => v as i32,
            None => 0,
        }
    }

    /// Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds and the SOFA scaling from 1960 to 1972, depending on flag.
    /// Returns None if the epoch is before 1960, year at which UTC was defined.
    ///
    /// # Why does this function return an `Option` when the other returns a value
    /// This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960.
    pub fn leap_seconds(&self, iers_only: bool) -> Option<f64> {
        for (tai_ts, delta_at, announced) in LEAP_SECONDS.iter().rev() {
            if self.0.in_seconds() >= *tai_ts && (!iers_only || *announced) {
                return Some(*delta_at);
            }
        }
        None
    }

    #[must_use]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    pub const fn from_tai_duration(duration: Duration) -> Self {
        Self(duration)
    }

    #[must_use]
    /// Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch.
    pub fn from_tai_parts(centuries: i16, nanoseconds: u64) -> Self {
        Self(Duration::from_parts(centuries, nanoseconds))
    }

    #[must_use]
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    pub fn from_tai_seconds(seconds: f64) -> Self {
        assert!(
            seconds.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self(seconds * Unit::Second)
    }

    #[must_use]
    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    pub fn from_tai_days(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self(days * Unit::Day)
    }

    #[must_use]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    pub fn from_utc_seconds(seconds: f64) -> Self {
        let mut e = Self::from_tai_seconds(seconds);
        // Compute the TAI to UTC offset at this time.
        // We have the time in TAI. But we were given UTC.
        // Hence, we need to _add_ the leap seconds to get the actual TAI time.
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        e.0 += e.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        e
    }

    #[must_use]
    /// Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight
    pub fn from_utc_days(days: f64) -> Self {
        let mut e = Self::from_tai_days(days);
        // Compute the TAI to UTC offset at this time.
        // We have the time in TAI. But we were given UTC.
        // Hence, we need to _add_ the leap seconds to get the actual TAI time.
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        e.0 += e.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        e
    }

    #[must_use]
    pub fn from_mjd_tai(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self((days - J1900_OFFSET) * Unit::Day)
    }

    #[must_use]
    pub fn from_mjd_utc(days: f64) -> Self {
        let mut e = Self::from_mjd_tai(days);
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        e.0 += e.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        e
    }

    #[must_use]
    pub fn from_jde_tai(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self((days - J1900_OFFSET - MJD_OFFSET) * Unit::Day)
    }

    #[must_use]
    pub fn from_jde_utc(days: f64) -> Self {
        let mut e = Self::from_jde_tai(days);
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        e.0 += e.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        e
    }

    #[must_use]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    pub fn from_tt_seconds(seconds: f64) -> Self {
        assert!(
            seconds.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_tt_duration(seconds * Unit::Second)
    }

    #[must_use]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    pub(crate) fn from_tt_duration(duration: Duration) -> Self {
        Self(duration) - Unit::Millisecond * TT_OFFSET_MS
    }

    #[must_use]
    /// Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)
    pub fn from_et_seconds(seconds_since_j2000: f64) -> Epoch {
        Self::from_et_duration(seconds_since_j2000 * Unit::Second)
    }

    #[must_use]
    pub fn from_et_duration(duration_since_j2000: Duration) -> Self {
        // WRT to J2000: offset to apply to TAI to give ET
        let delta_et_tai = Self::delta_et_tai(duration_since_j2000.in_seconds());
        // Offset back to J1900
        Self(
            (duration_since_j2000.in_seconds() - delta_et_tai) * Unit::Second
                + J2000_TO_J1900_DURATION,
        )
    }

    #[must_use]
    /// Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    /// NOTE: This uses the ESA algorithm, which is a notch more complicaste than the SPICE algorithm, but more precise.
    /// In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct.
    pub fn from_tdb_seconds(seconds_j2000: f64) -> Epoch {
        assert!(
            seconds_j2000.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_tdb_duration(seconds_j2000 * Unit::Second)
    }

    #[must_use]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI.
    pub(crate) fn from_tdb_duration(duration_j2k: Duration) -> Epoch {
        let gamma = Self::inner_g(duration_j2k.in_seconds());

        let delta_tdb_tai = gamma * Unit::Second + TT_OFFSET_MS * Unit::Millisecond;

        // Offset back to J1900.
        Self(duration_j2k - delta_tdb_tai + J2000_TO_J1900_DURATION)
    }

    #[must_use]
    /// Initialize from the JDE days
    pub fn from_jde_et(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_jde_tdb(days)
    }

    #[must_use]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days
    pub fn from_jde_tdb(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_jde_tai(days) - Unit::Microsecond * ET_OFFSET_US
    }

    #[must_use]
    /// Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn from_gpst_seconds(seconds: f64) -> Self {
        Self::from_tai_seconds(seconds) + Unit::Second * SECONDS_GPS_TAI_OFFSET
    }

    #[must_use]
    /// Initialize an Epoch from the number of days since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn from_gpst_days(days: f64) -> Self {
        Self::from_tai_days(days) + Unit::Day * DAYS_GPS_TAI_OFFSET
    }

    #[must_use]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use GPS as a time source.
    pub fn from_gpst_nanoseconds(nanoseconds: u64) -> Self {
        Self(Duration {
            centuries: 0,
            nanoseconds,
        }) + Unit::Second * SECONDS_GPS_TAI_OFFSET
    }

    #[must_use]
    /// Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01.
    pub fn from_unix_seconds(seconds: f64) -> Self {
        let utc_seconds = UNIX_REF_EPOCH.as_utc_duration() + seconds * Unit::Second;
        Self::from_utc_seconds(utc_seconds.in_unit(Unit::Second))
    }

    #[must_use]
    /// Initialize an Epoch from the provided UNIX milisecond timestamp since UTC midnight 1970 January 01.
    pub fn from_unix_milliseconds(millisecond: f64) -> Self {
        let utc_seconds = UNIX_REF_EPOCH.as_utc_duration() + millisecond * Unit::Millisecond;
        Self::from_utc_seconds(utc_seconds.in_unit(Unit::Second))
    }

    /// Attempts to build an Epoch from the provided Gregorian date and time in TAI.
    pub fn maybe_from_gregorian_tai(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, Errors> {
        Self::maybe_from_gregorian(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanos,
            TimeSystem::TAI,
        )
    }

    /// Attempts to build an Epoch from the provided Gregorian date and time in the provided time system.
    /// NOTE: If the timesystem is TDB, this function assumes that the SPICE format is used
    #[allow(clippy::too_many_arguments)]
    pub fn maybe_from_gregorian(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        ts: TimeSystem,
    ) -> Result<Self, Errors> {
        if !is_gregorian_valid(year, month, day, hour, minute, second, nanos) {
            return Err(Errors::Carry);
        }

        let mut duration_wrt_1900 = Unit::Day * i64::from(365 * (year - 1900).abs());
        // Now add the seconds for all the years prior to the current year
        for year in 1900..year {
            if is_leap_year(year) {
                duration_wrt_1900 += Unit::Day;
            }
        }
        // Add the seconds for the months prior to the current month
        for month in 0..month - 1 {
            duration_wrt_1900 += Unit::Day * i64::from(USUAL_DAYS_PER_MONTH[(month) as usize]);
        }
        if is_leap_year(year) && month > 2 {
            // NOTE: If on 29th of February, then the day is not finished yet, and therefore
            // the extra seconds are added below as per a normal day.
            duration_wrt_1900 += Unit::Day;
        }
        duration_wrt_1900 += Unit::Day * i64::from(day - 1)
            + Unit::Hour * i64::from(hour)
            + Unit::Minute * i64::from(minute)
            + Unit::Second * i64::from(second)
            + Unit::Nanosecond * i64::from(nanos);
        if second == 60 {
            // Herein lies the whole ambiguity of leap seconds. Two different UTC dates exist at the
            // same number of second afters J1900.0.
            duration_wrt_1900 -= Unit::Second;
        }

        // NOTE: For ET and TDB, we make sure to offset the duration back to J2000 since those functions expect a J2000 input.
        Ok(match ts {
            TimeSystem::TAI => Self(duration_wrt_1900),
            TimeSystem::TT => Self(duration_wrt_1900 - Unit::Millisecond * TT_OFFSET_MS),
            TimeSystem::ET => Self::from_et_duration(duration_wrt_1900 - J2000_TO_J1900_DURATION),
            TimeSystem::TDB => Self::from_tdb_duration(duration_wrt_1900 - J2000_TO_J1900_DURATION),
            TimeSystem::UTC => panic!("use maybe_from_gregorian_utc for UTC time system"),
        })
    }

    #[must_use]
    /// Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_tai if unsure.
    pub fn from_gregorian_tai(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Self {
        Self::maybe_from_gregorian_tai(year, month, day, hour, minute, second, nanos)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from the Gregoerian date at midnight in TAI.
    pub fn from_gregorian_tai_at_midnight(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_tai(year, month, day, 0, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from the Gregorian date at noon in TAI
    pub fn from_gregorian_tai_at_noon(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_tai(year, month, day, 12, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from the Gregorian date and time (without the nanoseconds) in TAI
    pub fn from_gregorian_tai_hms(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Self {
        Self::maybe_from_gregorian_tai(year, month, day, hour, minute, second, 0)
            .expect("invalid Gregorian date")
    }

    /// Attempts to build an Epoch from the provided Gregorian date and time in UTC.
    pub fn maybe_from_gregorian_utc(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, Errors> {
        let mut if_tai =
            Self::maybe_from_gregorian_tai(year, month, day, hour, minute, second, nanos)?;
        // Compute the TAI to UTC offset at this time.
        // We have the time in TAI. But we were given UTC.
        // Hence, we need to _add_ the leap seconds to get the actual TAI time.
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        if_tai.0 += if_tai.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        Ok(if_tai)
    }

    #[must_use]
    /// Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_tai if unsure.
    pub fn from_gregorian_utc(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Self {
        Self::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanos)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from Gregorian date in UTC at midnight
    pub fn from_gregorian_utc_at_midnight(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_utc(year, month, day, 0, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from Gregorian date in UTC at noon
    pub fn from_gregorian_utc_at_noon(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_utc(year, month, day, 12, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from the Gregorian date and time (without the nanoseconds) in UTC
    pub fn from_gregorian_utc_hms(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Self {
        Self::maybe_from_gregorian_utc(year, month, day, hour, minute, second, 0)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Returns the number of TAI seconds since J1900
    pub fn as_tai_seconds(&self) -> f64 {
        self.0.in_seconds()
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in TAI
    pub const fn as_tai_duration(&self) -> Duration {
        self.0
    }

    #[must_use]
    /// Returns the epoch as a floating point value in the provided unit
    pub fn as_tai(&self, unit: Unit) -> f64 {
        self.0.in_unit(unit)
    }

    #[must_use]
    /// Returns the TAI parts of this duration
    pub const fn to_tai_parts(&self) -> (i16, u64) {
        self.0.to_parts()
    }

    #[must_use]
    /// Returns the number of days since J1900 in TAI
    pub fn as_tai_days(&self) -> f64 {
        self.as_tai(Unit::Day)
    }

    #[must_use]
    /// Returns the number of UTC seconds since the TAI epoch
    pub fn as_utc_seconds(&self) -> f64 {
        self.as_utc(Unit::Second)
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UTC
    pub fn as_utc_duration(&self) -> Duration {
        // let cnt = self.get_num_leap_seconds();
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        self.0 - self.leap_seconds(true).unwrap_or(0.0) * Unit::Second
    }

    #[must_use]
    /// Returns the number of UTC seconds since the TAI epoch
    pub fn as_utc(&self, unit: Unit) -> f64 {
        self.as_utc_duration().in_unit(unit)
    }

    #[must_use]
    /// Returns the number of UTC days since the TAI epoch
    pub fn as_utc_days(&self) -> f64 {
        self.as_utc(Unit::Day)
    }

    #[must_use]
    /// `as_mjd_days` creates an Epoch from the provided Modified Julian Date in days as explained
    /// [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
    pub fn as_mjd_tai_days(&self) -> f64 {
        self.as_mjd_tai(Unit::Day)
    }

    #[must_use]
    /// Returns the Modified Julian Date in seconds TAI.
    pub fn as_mjd_tai_seconds(&self) -> f64 {
        self.as_mjd_tai(Unit::Second)
    }

    #[must_use]
    /// Returns this epoch as a duration in the requested units in MJD TAI
    pub fn as_mjd_tai(&self, unit: Unit) -> f64 {
        (self.0 + Unit::Day * J1900_OFFSET).in_unit(unit)
    }

    #[must_use]
    /// Returns the Modified Julian Date in days UTC.
    pub fn as_mjd_utc_days(&self) -> f64 {
        self.as_mjd_utc(Unit::Day)
    }

    #[must_use]
    /// Returns the Modified Julian Date in the provided unit in UTC.
    pub fn as_mjd_utc(&self, unit: Unit) -> f64 {
        (self.as_utc_duration() + Unit::Day * J1900_OFFSET).in_unit(unit)
    }

    #[must_use]
    /// Returns the Modified Julian Date in seconds UTC.
    pub fn as_mjd_utc_seconds(&self) -> f64 {
        self.as_mjd_utc(Unit::Second)
    }

    #[must_use]
    /// Returns the Julian days from epoch 01 Jan -4713, 12:00 (noon)
    /// as explained in "Fundamentals of astrodynamics and applications", Vallado et al.
    /// 4th edition, page 182, and on [Wikipedia](https://en.wikipedia.org/wiki/Julian_day).
    pub fn as_jde_tai_days(&self) -> f64 {
        self.as_jde_tai(Unit::Day)
    }

    #[must_use]
    pub fn as_jde_tai(&self, unit: Unit) -> f64 {
        self.as_jde_tai_duration().in_unit(unit)
    }

    #[must_use]
    pub fn as_jde_tai_duration(&self) -> Duration {
        self.0 + Unit::Day * J1900_OFFSET + Unit::Day * MJD_OFFSET
    }

    #[must_use]
    /// Returns the Julian seconds in TAI.
    pub fn as_jde_tai_seconds(&self) -> f64 {
        self.as_jde_tai(Unit::Second)
    }

    #[must_use]
    /// Returns the Julian days in UTC.
    pub fn as_jde_utc_days(&self) -> f64 {
        self.as_jde_utc_duration().in_unit(Unit::Day)
    }

    #[must_use]
    pub fn as_jde_utc_duration(&self) -> Duration {
        self.as_utc_duration() + Unit::Day * (J1900_OFFSET + MJD_OFFSET)
    }

    #[must_use]
    /// Returns the Julian seconds in UTC.
    pub fn as_jde_utc_seconds(&self) -> f64 {
        self.as_jde_utc_duration().in_seconds()
    }

    #[must_use]
    /// Returns seconds past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_tt_seconds(&self) -> f64 {
        self.as_tt_duration().in_seconds()
        // self.0.in_seconds() + (TT_OFFSET_MS as f64) * 1e-3
    }

    #[must_use]
    pub fn as_tt_duration(&self) -> Duration {
        self.0 + Unit::Millisecond * TT_OFFSET_MS
    }

    #[must_use]
    /// Returns days past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_tt_days(&self) -> f64 {
        self.as_tt_duration().in_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the centuries pased J2000 TT
    pub fn as_tt_centuries_j2k(&self) -> f64 {
        (self.as_tt_duration() - Unit::Second * ET_EPOCH_S).in_unit(Unit::Century)
    }

    #[must_use]
    /// Returns the duration past J2000 TT
    pub fn as_tt_since_j2k(&self) -> Duration {
        self.as_tt_duration() - Unit::Second * ET_EPOCH_S
    }

    #[must_use]
    /// Returns days past Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_jde_tt_days(&self) -> f64 {
        self.as_jde_tt_duration().in_unit(Unit::Day)
    }

    #[must_use]
    pub fn as_jde_tt_duration(&self) -> Duration {
        self.as_tt_duration() + Unit::Day * (J1900_OFFSET + MJD_OFFSET)
    }

    #[must_use]
    /// Returns days past Modified Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_mjd_tt_days(&self) -> f64 {
        self.as_mjd_tt_duration().in_unit(Unit::Day)
    }

    #[must_use]
    pub fn as_mjd_tt_duration(&self) -> Duration {
        self.as_tt_duration() + Unit::Day * J1900_OFFSET
    }

    #[must_use]
    /// Returns seconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn as_gpst_seconds(&self) -> f64 {
        self.as_gpst_duration().in_seconds()
    }

    #[must_use]
    pub fn as_gpst_duration(&self) -> Duration {
        self.as_tai_duration() - Unit::Second * SECONDS_GPS_TAI_OFFSET_I64
    }

    /// Returns nanoseconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// NOTE: This function will return an error if the centuries past GPST time are not zero.
    pub fn as_gpst_nanoseconds(&self) -> Result<u64, Errors> {
        let (centuries, nanoseconds) = self.as_gpst_duration().to_parts();
        if centuries != 0 {
            Err(Errors::Overflow)
        } else {
            Ok(nanoseconds)
        }
    }

    #[must_use]
    /// Returns days past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn as_gpst_days(&self) -> f64 {
        self.as_gpst_duration().in_unit(Unit::Day)
    }

    #[must_use]
    ///Returns the Duration since the UNIX epoch UTC midnight 01 Jan 1970.
    fn as_unix_duration(&self) -> Duration {
        // TAI = UNIX + leap_seconds + UNIX_OFFSET_UTC_SECONDS <=> UNIX = TAI - leap_seconds - UNIX_OFFSET_UTC_SECONDS
        self.0
            - self.leap_seconds(true).unwrap_or(0.0) * Unit::Second
            - UNIX_REF_EPOCH.as_utc_duration()
    }

    #[must_use]
    /// Returns the duration since the UNIX epoch in the provided unit.
    pub fn as_unix(&self, unit: Unit) -> f64 {
        self.as_unix_duration().in_unit(unit)
    }

    #[must_use]
    /// Returns the number seconds since the UNIX epoch defined 01 Jan 1970 midnight UTC.
    pub fn as_unix_seconds(&self) -> f64 {
        self.as_unix(Unit::Second)
    }

    #[must_use]
    /// Returns the number milliseconds since the UNIX epoch defined 01 Jan 1970 midnight UTC.
    pub fn as_unix_milliseconds(&self) -> f64 {
        self.as_unix(Unit::Millisecond)
    }

    #[must_use]
    /// Returns the number days since the UNIX epoch defined 01 Jan 1970 midnight UTC.
    pub fn as_unix_days(&self) -> f64 {
        self.as_unix(Unit::Day)
    }

    #[must_use]
    /// Returns the Ephemeris Time seconds past 2000 JAN 01 midnight, matches NASA/NAIF SPICE.
    pub fn as_et_seconds(&self) -> f64 {
        self.as_et_duration().in_seconds()
    }

    #[must_use]
    /// Returns the Ephemeris Time in duration past 1900 JAN 01 at noon.
    /// **Only** use this if the subsequent computation expect J1900 seconds.
    pub fn as_et_duration_since_j1900(&self) -> Duration {
        self.as_et_duration() + J2000_TO_J1900_DURATION
    }

    #[must_use]
    /// Returns the duration between J2000 and the current epoch as per NAIF SPICE.
    ///
    /// # Warning
    /// The et2utc function of NAIF SPICE will assume that there are 9 leap seconds before 01 JAN 1972,
    /// as this date introduces 10 leap seconds. At the time of writing, this does _not_ seem to be in
    /// line with IERS and the documentation in the leap seconds list.
    ///
    /// In order to match SPICE, the as_et_duration() function will manually get rid of that difference.
    pub fn as_et_duration(&self) -> Duration {
        // Run a Newton Raphston to convert find the correct value of the
        let mut seconds = (self.0 - J2000_TO_J1900_DURATION).in_seconds();
        for _ in 0..5 {
            seconds -= -NAIF_K
                * (NAIF_M0 + NAIF_M1 * seconds + NAIF_EB * (NAIF_M0 + NAIF_M1 * seconds).sin())
                    .sin();
        }

        // At this point, we have a good estimate of the number of seconds of this epoch.
        // Reverse the algorithm:
        let delta_et_tai =
            Self::delta_et_tai(seconds + (TT_OFFSET_MS * Unit::Millisecond).in_seconds());

        // Match SPICE by changing the UTC definition.
        self.0 + delta_et_tai * Unit::Second - J2000_TO_J1900_DURATION
    }

    fn delta_et_tai(seconds: f64) -> f64 {
        // Calculate M, the mean anomaly.4
        let m = NAIF_M0 + seconds * NAIF_M1;
        // Calculate eccentric anomaly
        let e = m + NAIF_EB * m.sin();

        (TT_OFFSET_MS * Unit::Millisecond).in_seconds() + NAIF_K * e.sin()
    }

    #[must_use]
    /// Returns the Dynamics Barycentric Time (TDB) as a high precision Duration since J2000
    ///
    /// ## Algorithm
    /// Given the embedded sine functinos in the equationto compute the difference between TDB and TAI from the number of TDB seconds
    /// past J2000, one cannot solve the revert the operation analytically. Instead, we iterate until the value no longer changes.
    ///
    /// 1. Assume that the TAI duration is in fact the TDB seconds frin J2000.
    /// 2. Offset to J2000 because `Epoch` stores everything in the J1900 but the TDB duration is in J2000.
    /// 3. Compute the offset `g` due to the TDB computation with the current value of the TDB seconds (defined in step 1).
    /// 4. Subtract that offset to the latest TDB seconds and store this as a new candidate for the true TDB seconds value.
    /// 5. Compute the difference between this candidtae and the previous one. If the difference is less than one nanosecond, stop iteration.
    /// 6. Set the new candidate as the TDB seconds since J2000 and loop until step 5 breaks the loop, or we've done five iterations.
    /// 7. At this stage, we have a good approximation of the TDB seconds since J2000.
    /// 8. Reverse the algorithm given that approximation: compute the `g` offset, compute the difference between TDB and TAI, add the TT offset (32.184 s), and offset by the difference between J1900 and J2000.
    pub fn as_tdb_duration(&self) -> Duration {
        // Iterate to convert find the correct value of the
        let mut seconds = (self.0 - J2000_TO_J1900_DURATION).in_seconds();
        let mut delta = 1e8; // Arbitrary large number, greater than first step of Newton Raphson.
        for _ in 0..5 {
            let next = seconds - Self::inner_g(seconds);
            let new_delta = (next - seconds).abs();
            if (new_delta - delta).abs() < 1e-9 {
                break;
            }
            seconds = next; // Loop
            delta = new_delta;
        }

        // At this point, we have a good estimate of the number of seconds of this epoch.
        // Reverse the algorithm:
        let gamma = Self::inner_g(seconds + (TT_OFFSET_MS * Unit::Millisecond).in_seconds());
        let delta_tdb_tai = gamma * Unit::Second + TT_OFFSET_MS * Unit::Millisecond;

        self.0 + delta_tdb_tai - J2000_TO_J1900_DURATION
    }

    #[must_use]
    /// Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)
    pub fn as_tdb_seconds(&self) -> f64 {
        self.as_tdb_duration().in_seconds()
    }

    #[must_use]
    /// Returns the Dynamics Barycentric Time (TDB) as a high precision Duration with reference epoch of 1900 JAN 01 at noon.
    /// **Only** use this if the subsequent computation expect J1900 seconds.
    pub fn as_tdb_duration_since_j1900(&self) -> Duration {
        self.as_tdb_duration() + J2000_TO_J1900_DURATION
    }

    fn inner_g(seconds: f64) -> f64 {
        use core::f64::consts::TAU;
        let g = TAU / 360.0 * 357.528 + 1.990_910_018_065_731e-7 * seconds;
        // Return gamma
        1.658e-3 * (g + 1.67e-2 * g.sin()).sin()
    }

    #[must_use]
    /// Returns the Ephemeris Time JDE past epoch
    pub fn as_jde_et_days(&self) -> f64 {
        self.as_jde_et_duration().in_unit(Unit::Day)
    }

    #[must_use]
    pub fn as_jde_et_duration(&self) -> Duration {
        self.as_et_duration() + Unit::Day * (J1900_OFFSET + MJD_OFFSET) + J2000_TO_J1900_DURATION
    }

    #[must_use]
    pub fn as_jde_et(&self, unit: Unit) -> f64 {
        self.as_jde_et_duration().in_unit(unit)
    }

    #[must_use]
    pub fn as_jde_tdb_duration(&self) -> Duration {
        self.as_tdb_duration() + Unit::Day * (J1900_OFFSET + MJD_OFFSET) + J2000_TO_J1900_DURATION
    }

    #[must_use]
    /// Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)
    pub fn as_jde_tdb_days(&self) -> f64 {
        self.as_jde_tdb_duration().in_unit(Unit::Day)
    }

    #[must_use]
    #[deprecated(note = "Prefer as_tdb_duration", since = "3.4.0")]
    /// Returns the duration since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)
    pub fn as_tdb_duration_since_j2000(&self) -> Duration {
        self.as_tdb_duration()
    }

    #[must_use]
    /// Returns the number of days since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)
    pub fn as_tdb_days_since_j2000(&self) -> f64 {
        self.as_tdb_duration().in_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the number of centuries since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)
    pub fn as_tdb_centuries_since_j2000(&self) -> f64 {
        self.as_tdb_duration().in_unit(Unit::Century)
    }

    #[must_use]
    #[deprecated(note = "Prefer as_et_duration", since = "3.4.0")]
    /// Returns the duration since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn as_et_duration_since_j2000(&self) -> Duration {
        self.as_et_duration()
    }

    #[must_use]
    /// Returns the number of days since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn as_et_days_since_j2000(&self) -> f64 {
        self.as_et_duration().in_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the number of centuries since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn as_et_centuries_since_j2000(&self) -> f64 {
        self.as_et_duration().in_unit(Unit::Century)
    }

    #[must_use]
    /// Converts the Epoch to the Gregorian UTC equivalent as (year, month, day, hour, minute, second).
    /// WARNING: Nanoseconds are lost in this conversion!
    ///
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    ///
    /// let dt = Epoch::from_tai_parts(1, 537582752000000000);
    ///
    /// // With the std feature, you may use FromStr as such
    /// // let dt_str = "2017-01-14T00:31:55 UTC";
    /// // let dt = Epoch::from_gregorian_str(dt_str).unwrap()
    ///
    /// let (y, m, d, h, min, s, _) = dt.as_gregorian_utc();
    /// assert_eq!(y, 2017);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 14);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 31);
    /// assert_eq!(s, 55);
    /// #[cfg(feature = "std")]
    /// assert_eq!("2017-01-14T00:31:55 UTC", dt.as_gregorian_utc_str().to_owned());
    /// ```
    pub fn as_gregorian_utc(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        Self::compute_gregorian(self.as_utc_seconds())
    }

    #[must_use]
    /// Converts the Epoch to the Gregorian TAI equivalent as (year, month, day, hour, minute, second).
    /// WARNING: Nanoseconds are lost in this conversion!
    ///
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    /// let dt = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    /// let (y, m, d, h, min, s, _) = dt.as_gregorian_tai();
    /// assert_eq!(y, 1972);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 1);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 0);
    /// assert_eq!(s, 0);
    /// ```
    pub fn as_gregorian_tai(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        Self::compute_gregorian(self.as_tai_seconds())
    }

    fn compute_gregorian(absolute_seconds_j1900: f64) -> (i32, u8, u8, u8, u8, u8, u32) {
        let (mut year, mut year_fraction) =
            div_rem_f64(absolute_seconds_j1900, DAYS_PER_YEAR_NLD * SECONDS_PER_DAY);
        // TAI is defined at 1900, so a negative time is before 1900 and positive is after 1900.
        year += 1900;
        // Base calculation was on 365 days, so we need to remove one day in seconds per leap year
        // between 1900 and `year`
        for year in 1900..year {
            if is_leap_year(year) {
                year_fraction -= SECONDS_PER_DAY;
            }
        }

        // Get the month from the exact number of seconds between the start of the year and now
        let mut seconds_til_this_month = 0.0;
        let mut month = 1;
        if year_fraction < 0.0 {
            month = 12;
            year -= 1;
        } else {
            loop {
                seconds_til_this_month +=
                    SECONDS_PER_DAY * f64::from(USUAL_DAYS_PER_MONTH[(month - 1) as usize]);
                if is_leap_year(year) && month == 2 {
                    seconds_til_this_month += SECONDS_PER_DAY;
                }
                if seconds_til_this_month > year_fraction {
                    break;
                }
                month += 1;
            }
        }
        let mut days_this_month = USUAL_DAYS_PER_MONTH[(month - 1) as usize];
        if month == 2 && is_leap_year(year) {
            days_this_month += 1;
        }
        // Get the month fraction by the number of seconds in this month from the number of
        // seconds since the start of this month.
        let (_, month_fraction) = div_rem_f64(
            year_fraction - seconds_til_this_month,
            f64::from(days_this_month) * SECONDS_PER_DAY,
        );
        // Get the day by the exact number of seconds in a day
        let (mut day, day_fraction) = div_rem_f64(month_fraction, SECONDS_PER_DAY);
        if day < 0 {
            // Overflow backwards (this happens for end of year calculations)
            month -= 1;
            if month == 0 {
                month = 12;
                year -= 1;
            }
            day = USUAL_DAYS_PER_MONTH[(month - 1) as usize] as i32;
        }
        day += 1; // Otherwise the day count starts at 0
                  // Get the hours by the exact number of seconds in an hour
        let (hours, hours_fraction) = div_rem_f64(day_fraction, 60.0 * 60.0);
        // Get the minutes and seconds by the exact number of seconds in a minute
        let (mins, secs) = div_rem_f64(hours_fraction, 60.0);
        let nanos = (div_rem_f64(secs, 1.0).1 * 1e9) as u32;
        (
            year,
            month as u8,
            day as u8,
            hours as u8,
            mins as u8,
            secs as u8,
            nanos,
        )
    }

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
    /// ```
    pub fn floor(&self, duration: Duration) -> Self {
        Self(self.0.floor(duration))
    }

    /// Ceils this epoch to the closest provided duration
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
    /// ```
    pub fn ceil(&self, duration: Duration) -> Self {
        Self(self.0.ceil(duration))
    }

    /// Rounds this epoch to the closest provided duration
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
        Self(self.0.round(duration))
    }
}

#[cfg(feature = "std")]
impl Epoch {
    /// Converts an ISO8601 Datetime representation without timezone offset to an Epoch.
    /// If no time system is specified, than UTC is assumed.
    /// The `T` which separates the date from the time can be replaced with a single whitespace character (`\W`).
    /// The offset is also optional, cf. the examples below.
    ///
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    /// let dt = Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 0);
    /// assert_eq!(
    ///     dt,
    ///     Epoch::from_gregorian_str("2017-01-14T00:31:55 UTC").unwrap()
    /// );
    /// assert_eq!(
    ///     dt,
    ///     Epoch::from_gregorian_str("2017-01-14T00:31:55.0000 UTC").unwrap()
    /// );
    /// assert_eq!(
    ///     dt,
    ///     Epoch::from_gregorian_str("2017-01-14T00:31:55").unwrap()
    /// );
    /// assert_eq!(
    ///     dt,
    ///     Epoch::from_gregorian_str("2017-01-14 00:31:55").unwrap()
    /// );
    /// // Regression test for #90
    /// assert_eq!(
    ///     Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 811000000),
    ///     Epoch::from_gregorian_str("2017-01-14 00:31:55.811 UTC").unwrap()
    /// );
    /// assert_eq!(
    ///     Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 811200000),
    ///     Epoch::from_gregorian_str("2017-01-14 00:31:55.8112 UTC").unwrap()
    /// );
    /// ```
    pub fn from_gregorian_str(s: &str) -> Result<Self, Errors> {
        let reg: Regex = Regex::new(
            r"^(\d{4})-(\d{2})-(\d{2})(?:T|\W)(\d{2}):(\d{2}):(\d{2})\.?(\d+)?\W?(\w{2,3})?$",
        )
        .unwrap();
        match reg.captures(s) {
            Some(cap) => {
                let nanos = match cap.get(7) {
                    Some(val) => {
                        let val_str = val.as_str();
                        let val = val_str.parse::<u32>().unwrap();
                        if val_str.len() != 9 {
                            val * 10_u32.pow((9 - val_str.len()) as u32)
                        } else {
                            val
                        }
                    }
                    None => 0,
                };

                match cap.get(8) {
                    Some(ts_str) => {
                        let ts = TimeSystem::from_str(ts_str.as_str())?;
                        if ts == TimeSystem::UTC {
                            Self::maybe_from_gregorian_utc(
                                cap[1].to_owned().parse::<i32>()?,
                                cap[2].to_owned().parse::<u8>()?,
                                cap[3].to_owned().parse::<u8>()?,
                                cap[4].to_owned().parse::<u8>()?,
                                cap[5].to_owned().parse::<u8>()?,
                                cap[6].to_owned().parse::<u8>()?,
                                nanos,
                            )
                        } else {
                            Self::maybe_from_gregorian(
                                cap[1].to_owned().parse::<i32>()?,
                                cap[2].to_owned().parse::<u8>()?,
                                cap[3].to_owned().parse::<u8>()?,
                                cap[4].to_owned().parse::<u8>()?,
                                cap[5].to_owned().parse::<u8>()?,
                                cap[6].to_owned().parse::<u8>()?,
                                nanos,
                                ts,
                            )
                        }
                    }
                    None => {
                        // Assume UTC
                        Self::maybe_from_gregorian_utc(
                            cap[1].to_owned().parse::<i32>()?,
                            cap[2].to_owned().parse::<u8>()?,
                            cap[3].to_owned().parse::<u8>()?,
                            cap[4].to_owned().parse::<u8>()?,
                            cap[5].to_owned().parse::<u8>()?,
                            cap[6].to_owned().parse::<u8>()?,
                            nanos,
                        )
                    }
                }
            }
            None => Err(Errors::ParseError(ParsingErrors::ISO8601)),
        }
    }

    #[must_use]
    /// Converts the Epoch to UTC Gregorian in the ISO8601 format.
    pub fn as_gregorian_utc_str(&self) -> String {
        format!("{}", self)
    }

    #[must_use]
    /// Converts the Epoch to TAI Gregorian in the ISO8601 format with " TAI" appended to the string
    pub fn as_gregorian_tai_str(&self) -> String {
        format!("{:x}", self)
    }

    #[must_use]
    /// Converts the Epoch to Gregorian in the provided time system and in the ISO8601 format with the time system appended to the string
    pub fn as_gregorian_str(&self, ts: TimeSystem) -> String {
        let (y, mm, dd, hh, min, s, nanos) = Self::compute_gregorian(match ts {
            TimeSystem::TT => self.as_tt_seconds(),
            TimeSystem::TAI => self.as_tai_seconds(),
            TimeSystem::ET => self.as_et_duration_since_j1900().in_seconds(),
            TimeSystem::TDB => self.as_tdb_duration_since_j1900().in_seconds(),
            TimeSystem::UTC => self.as_utc_seconds(),
        });
        if nanos == 0 {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:?}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{} {:?}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }

    /// Initializes a new Epoch from `now`.
    /// WARNING: This assumes that the system time returns the time in UTC (which is the case on Linux)
    /// Uses [`std::time::SystemTime::now`](https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.now) under the hood
    pub fn now() -> Result<Self, Errors> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(std_duration) => Ok(Self::from_unix_seconds(std_duration.as_secs_f64())),
            Err(_) => Err(Errors::SystemTimeError),
        }
    }
}

#[cfg(feature = "std")]
impl FromStr for Epoch {
    type Err = Errors;

    /// Attempts to convert a string to an Epoch.
    ///
    /// Format identifiers:
    ///  + JD: Julian days
    ///  + MJD: Modified Julian days
    ///  + SEC: Seconds past a given epoch (e.g. SEC 17.2 TAI is 17.2 seconds past TAI Epoch)
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    /// use std::str::FromStr;
    ///
    /// assert!(Epoch::from_str("JD 2452312.500372511 TDB").is_ok());
    /// assert!(Epoch::from_str("JD 2452312.500372511 ET").is_ok());
    /// assert!(Epoch::from_str("JD 2452312.500372511 TAI").is_ok());
    /// assert!(Epoch::from_str("MJD 51544.5 TAI").is_ok());
    /// assert!(Epoch::from_str("SEC 0.5 TAI").is_ok());
    /// assert!(Epoch::from_str("SEC 66312032.18493909 TDB").is_ok());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg: Regex = Regex::new(r"^(\w{2,3})\W?(\d+\.?\d+)\W?(\w{2,3})?$").unwrap();
        // Try to match Gregorian date
        match Self::from_gregorian_str(s) {
            Ok(e) => Ok(e),
            Err(_) => match reg.captures(s) {
                Some(cap) => {
                    let format = cap[1].to_owned().parse::<String>().unwrap();
                    let value = cap[2].to_owned().parse::<f64>().unwrap();
                    let ts = TimeSystem::from_str(&cap[3])?;

                    match format.as_str() {
                        "JD" => match ts {
                            TimeSystem::ET => Ok(Self::from_jde_et(value)),
                            TimeSystem::TAI => Ok(Self::from_jde_tai(value)),
                            TimeSystem::TDB => Ok(Self::from_jde_tdb(value)),
                            TimeSystem::UTC => Ok(Self::from_jde_utc(value)),
                            _ => Err(Errors::ParseError(ParsingErrors::UnsupportedTimeSystem)),
                        },
                        "MJD" => match ts {
                            TimeSystem::TAI => Ok(Self::from_mjd_tai(value)),
                            TimeSystem::UTC => Ok(Self::from_mjd_utc(value)),
                            _ => Err(Errors::ParseError(ParsingErrors::UnsupportedTimeSystem)),
                        },
                        "SEC" => match ts {
                            TimeSystem::TAI => Ok(Self::from_tai_seconds(value)),
                            TimeSystem::ET => Ok(Self::from_et_seconds(value)),
                            TimeSystem::TDB => Ok(Self::from_tdb_seconds(value)),
                            TimeSystem::TT => Ok(Self::from_tt_seconds(value)),
                            TimeSystem::UTC => Ok(Self::from_utc_seconds(value)),
                        },
                        _ => Err(Errors::ParseError(ParsingErrors::UnknownFormat)),
                    }
                }
                None => Err(Errors::ParseError(ParsingErrors::UnknownFormat)),
            },
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Epoch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl fmt::Display for Epoch {
    /// The default format of an epoch is in UTC
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeSystem::UTC;
        let (y, mm, dd, hh, min, s, nanos) = Self::compute_gregorian(self.as_utc_seconds());
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:?}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{} {:?}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::LowerHex for Epoch {
    /// Prints the Epoch in TAI
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeSystem::TAI;
        let (y, mm, dd, hh, min, s, nanos) = Self::compute_gregorian(self.as_tai_seconds());
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:?}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{} {:?}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::UpperHex for Epoch {
    /// Prints the Epoch in TT
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeSystem::TT;
        let (y, mm, dd, hh, min, s, nanos) = Self::compute_gregorian(self.as_tt_seconds());
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:?}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{} {:?}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::LowerExp for Epoch {
    /// Prints the Epoch in TDB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeSystem::TDB;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.as_tdb_duration_since_j1900().in_seconds());
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:?}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{} {:?}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::UpperExp for Epoch {
    /// Prints the Epoch in ET
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeSystem::ET;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.as_et_duration_since_j1900().in_seconds());
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:?}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{} {:?}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::Pointer for Epoch {
    /// Prints the Epoch in UNIX
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_unix_seconds())
    }
}

impl fmt::Octal for Epoch {
    /// Prints the Epoch in GPS
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_gpst_nanoseconds().unwrap())
    }
}

#[must_use]
/// Returns true if the provided Gregorian date is valid. Leap second days may have 60 seconds.
pub fn is_gregorian_valid(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanos: u32,
) -> bool {
    let max_seconds = if (month == 12 || month == 6)
        && day == USUAL_DAYS_PER_MONTH[month as usize - 1]
        && hour == 23
        && minute == 59
        && ((month == 6 && JULY_YEARS.contains(&year))
            || (month == 12 && JANUARY_YEARS.contains(&(year + 1))))
    {
        60
    } else {
        59
    };
    // General incorrect date times
    if month == 0
        || month > 12
        || day == 0
        || day > 31
        || hour > 24
        || minute > 59
        || second > max_seconds
        || f64::from(nanos) > 1e9
    {
        return false;
    }
    if day > USUAL_DAYS_PER_MONTH[month as usize - 1] && (month != 2 || !is_leap_year(year)) {
        // Not in February or not a leap year
        return false;
    }
    true
}

/// `is_leap_year` returns whether the provided year is a leap year or not.
/// Tests for this function are part of the Datetime tests.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn div_rem_f64(me: f64, rhs: f64) -> (i32, f64) {
    ((div_euclid_f64(me, rhs) as i32), rem_euclid_f64(me, rhs))
}

fn div_euclid_f64(lhs: f64, rhs: f64) -> f64 {
    let q = (lhs / rhs).trunc();
    if lhs % rhs < 0.0 {
        return if rhs > 0.0 { q - 1.0 } else { q + 1.0 };
    }
    q
}

fn rem_euclid_f64(lhs: f64, rhs: f64) -> f64 {
    let r = lhs % rhs;
    if r < 0.0 {
        r + rhs.abs()
    } else {
        r
    }
}

#[test]
fn div_rem_f64_test() {
    assert_eq!(div_rem_f64(24.0, 6.0), (4, 0.0));
    assert_eq!(div_rem_f64(25.0, 6.0), (4, 1.0));
    assert_eq!(div_rem_f64(6.0, 6.0), (1, 0.0));
    assert_eq!(div_rem_f64(5.0, 6.0), (0, 5.0));
    assert_eq!(div_rem_f64(3540.0, 3600.0), (0, 3540.0));
    assert_eq!(div_rem_f64(3540.0, 60.0), (59, 0.0));
    assert_eq!(div_rem_f64(24.0, -6.0), (-4, 0.0));
    assert_eq!(div_rem_f64(-24.0, 6.0), (-4, 0.0));
    assert_eq!(div_rem_f64(-24.0, -6.0), (4, 0.0));
}

#[test]
fn test_days_tdb_j2000() {
    let e = Epoch(Duration::from_parts(1, 723038437000000000));
    let days_d = e.as_tdb_days_since_j2000();
    let centuries_t = e.as_tdb_centuries_since_j2000();
    assert!((days_d - 8369.000800729798).abs() < f64::EPSILON);
    assert!((centuries_t - 0.22913075429787266).abs() < f64::EPSILON);
}

#[test]
fn leap_year() {
    assert!(!is_leap_year(2019));
    assert!(!is_leap_year(2001));
    assert!(!is_leap_year(1000));
    // List of leap years from https://kalender-365.de/leap-years.php .
    let leap_years: [i32; 146] = [
        1804, 1808, 1812, 1816, 1820, 1824, 1828, 1832, 1836, 1840, 1844, 1848, 1852, 1856, 1860,
        1864, 1868, 1872, 1876, 1880, 1884, 1888, 1892, 1896, 1904, 1908, 1912, 1916, 1920, 1924,
        1928, 1932, 1936, 1940, 1944, 1948, 1952, 1956, 1960, 1964, 1968, 1972, 1976, 1980, 1984,
        1988, 1992, 1996, 2000, 2004, 2008, 2012, 2016, 2020, 2024, 2028, 2032, 2036, 2040, 2044,
        2048, 2052, 2056, 2060, 2064, 2068, 2072, 2076, 2080, 2084, 2088, 2092, 2096, 2104, 2108,
        2112, 2116, 2120, 2124, 2128, 2132, 2136, 2140, 2144, 2148, 2152, 2156, 2160, 2164, 2168,
        2172, 2176, 2180, 2184, 2188, 2192, 2196, 2204, 2208, 2212, 2216, 2220, 2224, 2228, 2232,
        2236, 2240, 2244, 2248, 2252, 2256, 2260, 2264, 2268, 2272, 2276, 2280, 2284, 2288, 2292,
        2296, 2304, 2308, 2312, 2316, 2320, 2324, 2328, 2332, 2336, 2340, 2344, 2348, 2352, 2356,
        2360, 2364, 2368, 2372, 2376, 2380, 2384, 2388, 2392, 2396, 2400,
    ];
    for year in leap_years.iter() {
        assert!(is_leap_year(*year));
    }
}

#[cfg(feature = "std")]
#[test]
fn deser_test() {
    use serde_derive::Deserialize;
    #[derive(Deserialize)]
    struct _D {
        pub _e: Epoch,
    }

    println!("{}", (1 * Unit::Century + 12 * Unit::Hour).in_seconds());
}
