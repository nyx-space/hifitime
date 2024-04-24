/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::duration::{Duration, Unit};
use crate::efmt::format::Format;
use crate::leap_seconds::{LatestLeapSeconds, LeapSecondProvider};
use crate::Weekday;
use crate::{
    Errors, MonthName, TimeScale, TimeUnits, BDT_REF_EPOCH, DAYS_PER_YEAR_NLD, ET_EPOCH_S,
    GPST_REF_EPOCH, GST_REF_EPOCH, HIFITIME_REF_YEAR, J1900_OFFSET, J2000_TO_J1900_DURATION,
    MJD_OFFSET, NANOSECONDS_PER_DAY, NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND,
    NANOSECONDS_PER_SECOND_U32, QZSST_REF_EPOCH, UNIX_REF_EPOCH,
};

#[cfg(not(kani))]
use crate::parser::Token;
#[cfg(not(kani))]
use crate::ParsingErrors;

use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

#[cfg(feature = "python")]
use pyo3::types::PyType;

#[cfg(feature = "python")]
use crate::leap_seconds_file::LeapSecondsFile;

#[cfg(not(kani))]
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use core::str::FromStr;

#[cfg(feature = "ut1")]
use crate::ut1::Ut1Provider;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

#[cfg(feature = "std")]
mod system_time;

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

/// Years when January had the leap second
const fn january_years(year: i32) -> bool {
    matches!(
        year,
        1972 | 1973
            | 1974
            | 1975
            | 1976
            | 1977
            | 1978
            | 1979
            | 1980
            | 1988
            | 1990
            | 1991
            | 1996
            | 1999
            | 2006
            | 2009
            | 2017
    )
}

/// Years when July had the leap second
const fn july_years(year: i32) -> bool {
    matches!(
        year,
        1972 | 1981 | 1982 | 1983 | 1985 | 1992 | 1993 | 1994 | 1997 | 2012 | 2015
    )
}

/// Returns the usual days in a given month (zero indexed, i.e. January is month zero and December is month 11)
///
/// # Warning
/// This will return 0 days if the month is invalid.
const fn usual_days_per_month(month: u8) -> u8 {
    match month + 1 {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28,
        _ => 0,
    }
}

/// Calculates the prefix-sum of days counted up to the month start
const CUMULATIVE_DAYS_FOR_MONTH: [u16; 12] = {
    let mut days = [0; 12];
    let mut month = 1;
    while month < 12 {
        days[month] = days[month - 1] + usual_days_per_month(month as u8 - 1) as u16;
        month += 1;
    }
    days
};

/// Defines a nanosecond-precision Epoch.
///
/// Refer to the appropriate functions for initializing this Epoch from different time scales or representations.
#[derive(Copy, Clone, Default, Eq)]
#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "hifitime"))]
pub struct Epoch {
    /// An Epoch is always stored as the duration since the beginning of its time scale
    pub duration: Duration,
    /// Time scale used during the initialization of this Epoch.
    pub time_scale: TimeScale,
}

impl Hash for Epoch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.duration.hash(state);
        self.time_scale.hash(state);
    }
}

#[cfg(not(kani))]
#[cfg(feature = "serde")]
impl Serialize for Epoch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string(); // Assuming `Display` is implemented for `Epoch`
        serializer.serialize_str(&s)
    }
}

#[cfg(not(kani))]
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Epoch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Epoch::from_str(&s).map_err(serde::de::Error::custom)
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

// Defines the methods that should be classmethods in Python, but must be redefined as per https://github.com/PyO3/pyo3/issues/1003#issuecomment-844433346
impl Epoch {
    /// Get the accumulated number of leap seconds up to this Epoch from the provided LeapSecondProvider.
    /// Returns None if the epoch is before 1960, year at which UTC was defined.
    ///
    /// # Why does this function return an `Option` when the other returns a value
    /// This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960.
    pub fn leap_seconds_with<L: LeapSecondProvider>(
        &self,
        iers_only: bool,
        provider: L,
    ) -> Option<f64> {
        for leap_second in provider.rev() {
            if self.to_tai_duration().to_seconds() >= leap_second.timestamp_tai_s
                && (!iers_only || leap_second.announced_by_iers)
            {
                return Some(leap_second.delta_at);
            }
        }
        None
    }

    /// Creates an epoch from given duration expressed in given timescale, i.e. since the given time scale's reference epoch.
    ///
    /// For example, if the duration is 1 day and the time scale is Ephemeris Time, then this will create an epoch of 2000-01-02 at midnight ET. If the duration is 1 day and the time scale is TAI, this will create an epoch of 1900-01-02 at noon, because the TAI reference epoch in Hifitime is chosen to be the J1900 epoch.
    /// In case of ET, TDB Timescales, a duration since J2000 is expected.
    #[must_use]
    pub const fn from_duration(duration: Duration, ts: TimeScale) -> Self {
        Self {
            duration,
            time_scale: ts,
        }
    }

    pub fn to_duration_since_j1900(&self) -> Duration {
        self.to_time_scale(TimeScale::TAI).duration
    }

    #[must_use]
    /// Converts self to another time scale
    ///
    /// As per the [Rust naming convention](https://rust-lang.github.io/api-guidelines/naming.html#ad-hoc-conversions-follow-as_-to_-into_-conventions-c-conv),
    /// this borrows an Epoch and returns an owned Epoch.
    pub fn to_time_scale(&self, ts: TimeScale) -> Self {
        if ts == self.time_scale {
            // Do nothing, just return a copy
            *self
        } else {
            // Now we need to convert from the current time scale into the desired time scale.
            // Let's first compute this epoch from its current time scale into TAI.
            let prime_epoch_offset = match self.time_scale {
                TimeScale::TAI => self.duration,
                TimeScale::TT => self.duration - TT_OFFSET_MS.milliseconds(),
                TimeScale::ET => {
                    // Run a Newton Raphston to convert find the correct value of the
                    let mut seconds_j2000 = self.duration.to_seconds();
                    for _ in 0..5 {
                        seconds_j2000 += -NAIF_K
                            * (NAIF_M0
                                + NAIF_M1 * seconds_j2000
                                + NAIF_EB * (NAIF_M0 + NAIF_M1 * seconds_j2000).sin())
                            .sin();
                    }

                    // At this point, we have a good estimate of the number of seconds of this epoch.
                    // Reverse the algorithm:
                    let delta_et_tai = Self::delta_et_tai(
                        seconds_j2000 - (TT_OFFSET_MS * Unit::Millisecond).to_seconds(),
                    );

                    // Match SPICE by changing the UTC definition.
                    self.duration - delta_et_tai.seconds() + self.time_scale.prime_epoch_offset()
                }
                TimeScale::TDB => {
                    let gamma = Self::inner_g(self.duration.to_seconds());

                    let delta_tdb_tai = gamma * Unit::Second + TT_OFFSET_MS * Unit::Millisecond;

                    // Offset back to J1900.
                    self.duration - delta_tdb_tai + self.time_scale.prime_epoch_offset()
                }
                TimeScale::UTC => {
                    // Assume this is TAI
                    let mut tai_assumption = *self;
                    tai_assumption.time_scale = TimeScale::TAI;
                    self.duration + tai_assumption.leap_seconds(true).unwrap_or(0.0).seconds()
                }
                TimeScale::GPST => self.duration + GPST_REF_EPOCH.to_tai_duration(),
                TimeScale::GST => self.duration + GST_REF_EPOCH.to_tai_duration(),
                TimeScale::BDT => self.duration + BDT_REF_EPOCH.to_tai_duration(),
                TimeScale::QZSST => self.duration + QZSST_REF_EPOCH.to_tai_duration(),
            };

            // Convert to the desired time scale from the TAI duration
            let ts_ref_offset = match ts {
                TimeScale::TAI => prime_epoch_offset,
                TimeScale::TT => prime_epoch_offset + TT_OFFSET_MS.milliseconds(),
                TimeScale::ET => {
                    // Run a Newton Raphston to convert find the correct value of the ... ?!

                    let mut seconds = (prime_epoch_offset - ts.prime_epoch_offset()).to_seconds();
                    for _ in 0..5 {
                        seconds -= -NAIF_K
                            * (NAIF_M0
                                + NAIF_M1 * seconds
                                + NAIF_EB * (NAIF_M0 + NAIF_M1 * seconds).sin())
                            .sin();
                    }

                    // At this point, we have a good estimate of the number of seconds of this epoch.
                    // Reverse the algorithm:
                    let delta_et_tai = Self::delta_et_tai(
                        seconds + (TT_OFFSET_MS * Unit::Millisecond).to_seconds(),
                    );

                    // Match SPICE by changing the UTC definition.
                    prime_epoch_offset + delta_et_tai.seconds() - ts.prime_epoch_offset()
                }
                TimeScale::TDB => {
                    // Iterate to convert find the correct value of the
                    let mut seconds = (prime_epoch_offset - ts.prime_epoch_offset()).to_seconds();
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
                    let gamma =
                        Self::inner_g(seconds + (TT_OFFSET_MS * Unit::Millisecond).to_seconds());
                    let delta_tdb_tai = gamma.seconds() + TT_OFFSET_MS.milliseconds();

                    prime_epoch_offset + delta_tdb_tai - ts.prime_epoch_offset()
                }
                TimeScale::UTC => {
                    // Assume it's TAI
                    let epoch = Self {
                        duration: prime_epoch_offset,
                        time_scale: TimeScale::TAI,
                    };
                    // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
                    prime_epoch_offset - epoch.leap_seconds(true).unwrap_or(0.0).seconds()
                }
                TimeScale::GPST => prime_epoch_offset - GPST_REF_EPOCH.to_tai_duration(),
                TimeScale::GST => prime_epoch_offset - GST_REF_EPOCH.to_tai_duration(),
                TimeScale::BDT => prime_epoch_offset - BDT_REF_EPOCH.to_tai_duration(),
                TimeScale::QZSST => prime_epoch_offset - QZSST_REF_EPOCH.to_tai_duration(),
            };

            Self {
                duration: ts_ref_offset,
                time_scale: ts,
            }
        }
    }

    #[must_use]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    pub const fn from_tai_duration(duration: Duration) -> Self {
        Self {
            duration,
            time_scale: TimeScale::TAI,
        }
    }

    #[must_use]
    /// Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch.
    pub fn from_tai_parts(centuries: i16, nanoseconds: u64) -> Self {
        Self::from_tai_duration(Duration::from_parts(centuries, nanoseconds))
    }

    #[must_use]
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    pub fn from_tai_seconds(seconds: f64) -> Self {
        assert!(
            seconds.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_tai_duration(seconds * Unit::Second)
    }

    #[must_use]
    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    pub fn from_tai_days(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_tai_duration(days * Unit::Day)
    }

    #[must_use]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    pub fn from_utc_duration(duration: Duration) -> Self {
        Self::from_duration(duration, TimeScale::UTC)
    }

    #[must_use]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    pub fn from_utc_seconds(seconds: f64) -> Self {
        Self::from_utc_duration(seconds * Unit::Second)
    }

    #[must_use]
    /// Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight
    pub fn from_utc_days(days: f64) -> Self {
        Self::from_utc_duration(days * Unit::Day)
    }

    #[must_use]
    /// Initialize an Epoch from the provided duration since 1980 January 6 at midnight
    pub fn from_gpst_duration(duration: Duration) -> Self {
        Self::from_duration(duration, TimeScale::GPST)
    }

    #[must_use]
    /// Initialize an Epoch from the provided duration since 1980 January 6 at midnight
    pub fn from_qzsst_duration(duration: Duration) -> Self {
        Self::from_duration(duration, TimeScale::QZSST)
    }

    #[must_use]
    /// Initialize an Epoch from the provided duration since August 21st 1999 midnight
    pub fn from_gst_duration(duration: Duration) -> Self {
        Self::from_duration(duration, TimeScale::GST)
    }

    #[must_use]
    /// Initialize an Epoch from the provided duration since January 1st midnight
    pub fn from_bdt_duration(duration: Duration) -> Self {
        Self::from_duration(duration, TimeScale::BDT)
    }

    #[must_use]
    pub fn from_mjd_tai(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_tai_duration((days - J1900_OFFSET) * Unit::Day)
    }

    fn from_mjd_in_time_scale(days: f64, time_scale: TimeScale) -> Self {
        // always refer to TAI/mjd
        let mut e = Self::from_mjd_tai(days);
        if time_scale.uses_leap_seconds() {
            e.duration += e.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        }
        e.time_scale = time_scale;
        e
    }

    #[must_use]
    pub fn from_mjd_utc(days: f64) -> Self {
        Self::from_mjd_in_time_scale(days, TimeScale::UTC)
    }
    #[must_use]
    pub fn from_mjd_gpst(days: f64) -> Self {
        Self::from_mjd_in_time_scale(days, TimeScale::GPST)
    }
    #[must_use]
    pub fn from_mjd_qzsst(days: f64) -> Self {
        Self::from_mjd_in_time_scale(days, TimeScale::QZSST)
    }
    #[must_use]
    pub fn from_mjd_gst(days: f64) -> Self {
        Self::from_mjd_in_time_scale(days, TimeScale::GST)
    }
    #[must_use]
    pub fn from_mjd_bdt(days: f64) -> Self {
        Self::from_mjd_in_time_scale(days, TimeScale::BDT)
    }

    #[must_use]
    pub fn from_jde_tai(days: f64) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self::from_tai_duration((days - J1900_OFFSET - MJD_OFFSET) * Unit::Day)
    }

    fn from_jde_in_time_scale(days: f64, time_scale: TimeScale) -> Self {
        // always refer to TAI/jde
        let mut e = Self::from_jde_tai(days);
        if time_scale.uses_leap_seconds() {
            e.duration += e.leap_seconds(true).unwrap_or(0.0) * Unit::Second;
        }
        e.time_scale = time_scale;
        e
    }

    #[must_use]
    pub fn from_jde_utc(days: f64) -> Self {
        Self::from_jde_in_time_scale(days, TimeScale::UTC)
    }
    #[must_use]
    pub fn from_jde_gpst(days: f64) -> Self {
        Self::from_jde_in_time_scale(days, TimeScale::GPST)
    }
    #[must_use]
    pub fn from_jde_qzsst(days: f64) -> Self {
        Self::from_jde_in_time_scale(days, TimeScale::QZSST)
    }
    #[must_use]
    pub fn from_jde_gst(days: f64) -> Self {
        Self::from_jde_in_time_scale(days, TimeScale::GST)
    }
    #[must_use]
    pub fn from_jde_bdt(days: f64) -> Self {
        Self::from_jde_in_time_scale(days, TimeScale::BDT)
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
    pub fn from_tt_duration(duration: Duration) -> Self {
        Self::from_duration(duration, TimeScale::TT)
    }

    #[must_use]
    /// Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)
    pub fn from_et_seconds(seconds_since_j2000: f64) -> Epoch {
        Self::from_et_duration(seconds_since_j2000 * Unit::Second)
    }

    /// Initializes an Epoch from the duration between J2000 and the current epoch as per NAIF SPICE.
    ///
    /// # Limitation
    /// This method uses a Newton Raphson iteration to find the appropriate TAI duration. This method is only accuracy to a few nanoseconds.
    /// Hence, when calling `as_et_duration()` and re-initializing it with `from_et_duration` you may have a few nanoseconds of difference (expect less than 10 ns).
    ///
    /// # Warning
    /// The et2utc function of NAIF SPICE will assume that there are 9 leap seconds before 01 JAN 1972,
    /// as this date introduces 10 leap seconds. At the time of writing, this does _not_ seem to be in
    /// line with IERS and the documentation in the leap seconds list.
    ///
    /// In order to match SPICE, the as_et_duration() function will manually get rid of that difference.
    #[must_use]
    pub fn from_et_duration(duration_since_j2000: Duration) -> Self {
        Self::from_duration(duration_since_j2000, TimeScale::ET)
    }

    #[must_use]
    /// Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    /// NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
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
    pub fn from_tdb_duration(duration_since_j2000: Duration) -> Epoch {
        Self::from_duration(duration_since_j2000, TimeScale::TDB)
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
        Self::from_duration(seconds * Unit::Second, TimeScale::GPST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of days since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn from_gpst_days(days: f64) -> Self {
        Self::from_duration(days * Unit::Day, TimeScale::GPST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use GPS as a time source.
    pub fn from_gpst_nanoseconds(nanoseconds: u64) -> Self {
        Self::from_duration(nanoseconds as f64 * Unit::Nanosecond, TimeScale::GPST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn from_qzsst_seconds(seconds: f64) -> Self {
        Self::from_duration(seconds * Unit::Second, TimeScale::QZSST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of days since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn from_qzsst_days(days: f64) -> Self {
        Self::from_duration(days * Unit::Day, TimeScale::QZSST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use QZSS as a time source.
    pub fn from_qzsst_nanoseconds(nanoseconds: u64) -> Self {
        Self::from_duration(
            Duration {
                centuries: 0,
                nanoseconds,
            },
            TimeScale::QZSST,
        )
    }

    #[must_use]
    /// Initialize an Epoch from the number of seconds since the GST Time Epoch,
    /// starting August 21st 1999 midnight (UTC)
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    pub fn from_gst_seconds(seconds: f64) -> Self {
        Self::from_duration(seconds * Unit::Second, TimeScale::GST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of days since the GST Time Epoch,
    /// starting August 21st 1999 midnight (UTC)
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)
    pub fn from_gst_days(days: f64) -> Self {
        Self::from_duration(days * Unit::Day, TimeScale::GST)
    }

    #[must_use]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// starting August 21st 1999 midnight (UTC)
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)
    pub fn from_gst_nanoseconds(nanoseconds: u64) -> Self {
        Self::from_duration(
            Duration {
                centuries: 0,
                nanoseconds,
            },
            TimeScale::GST,
        )
    }

    #[must_use]
    /// Initialize an Epoch from the number of seconds since the BDT Time Epoch,
    /// starting on January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)
    pub fn from_bdt_seconds(seconds: f64) -> Self {
        Self::from_duration(seconds * Unit::Second, TimeScale::BDT)
    }

    #[must_use]
    /// Initialize an Epoch from the number of days since the BDT Time Epoch,
    /// starting on January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)
    pub fn from_bdt_days(days: f64) -> Self {
        Self::from_duration(days * Unit::Day, TimeScale::BDT)
    }

    #[must_use]
    /// Initialize an Epoch from the number of nanoseconds since the BDT Time Epoch,
    /// starting on January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use BDT as a time source.
    pub fn from_bdt_nanoseconds(nanoseconds: u64) -> Self {
        Self::from_duration(
            Duration {
                centuries: 0,
                nanoseconds,
            },
            TimeScale::BDT,
        )
    }

    #[must_use]
    /// Initialize an Epoch from the provided duration since UTC midnight 1970 January 01.
    pub fn from_unix_duration(duration: Duration) -> Self {
        Self::from_utc_duration(UNIX_REF_EPOCH.to_utc_duration() + duration)
    }

    #[must_use]
    /// Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01.
    pub fn from_unix_seconds(seconds: f64) -> Self {
        Self::from_utc_duration(UNIX_REF_EPOCH.to_utc_duration() + seconds * Unit::Second)
    }

    #[must_use]
    /// Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01.
    pub fn from_unix_milliseconds(millisecond: f64) -> Self {
        Self::from_utc_duration(UNIX_REF_EPOCH.to_utc_duration() + millisecond * Unit::Millisecond)
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
            TimeScale::TAI,
        )
    }

    /// Attempts to build an Epoch from the provided Gregorian date and time in the provided time scale.
    /// NOTE: If the time scale is TDB, this function assumes that the SPICE format is used
    #[allow(clippy::too_many_arguments)]
    pub fn maybe_from_gregorian(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        time_scale: TimeScale,
    ) -> Result<Self, Errors> {
        if !is_gregorian_valid(year, month, day, hour, minute, second, nanos) {
            return Err(Errors::Carry);
        }

        let mut duration_wrt_ref = match year.checked_sub(HIFITIME_REF_YEAR) {
            None => return Err(Errors::Overflow),
            Some(years_since_ref) => match years_since_ref.checked_mul(365) {
                None => return Err(Errors::Overflow),
                Some(days) => Unit::Day * i64::from(days),
            },
        } - time_scale.gregorian_epoch_offset();

        // Now add the leap days for all the years prior to the current year
        if year >= HIFITIME_REF_YEAR {
            // Add days
            for year in HIFITIME_REF_YEAR..year {
                if is_leap_year(year) {
                    duration_wrt_ref += Unit::Day;
                }
            }
            // Remove ref hours from duration to correct for the time scale not starting at midnight
            // duration_wrt_ref -= Unit::Hour * time_scale.ref_hour() as i64;
        } else {
            // Remove days
            for year in year..HIFITIME_REF_YEAR {
                if is_leap_year(year) {
                    duration_wrt_ref -= Unit::Day;
                }
            }
            // Add ref hours
            // duration_wrt_ref += Unit::Hour * time_scale.ref_hour() as i64;
        }

        // Add the seconds for the months prior to the current month
        duration_wrt_ref += Unit::Day * i64::from(CUMULATIVE_DAYS_FOR_MONTH[(month - 1) as usize]);

        if is_leap_year(year) && month > 2 {
            // NOTE: If on 29th of February, then the day is not finished yet, and therefore
            // the extra seconds are added below as per a normal day.
            duration_wrt_ref += Unit::Day;
        }
        duration_wrt_ref += Unit::Day * i64::from(day - 1)
            + Unit::Hour * i64::from(hour)
            + Unit::Minute * i64::from(minute)
            + Unit::Second * i64::from(second)
            + Unit::Nanosecond * i64::from(nanos);

        if second == 60 {
            // Herein lies the whole ambiguity of leap seconds. Two different UTC dates exist at the
            // same number of second after J1900.0.
            duration_wrt_ref -= Unit::Second;
        }

        Ok(Self {
            duration: duration_wrt_ref,
            time_scale,
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
    /// Initialize from the Gregorian date at midnight in TAI.
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
        Self::maybe_from_gregorian(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanos,
            TimeScale::UTC,
        )
    }

    #[must_use]
    /// Builds an Epoch from the provided Gregorian date and time in UTC. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_utc if unsure.
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

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    /// Builds an Epoch from the provided Gregorian date and time in the provided time scale. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian if unsure.
    pub fn from_gregorian(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        time_scale: TimeScale,
    ) -> Self {
        Self::maybe_from_gregorian(year, month, day, hour, minute, second, nanos, time_scale)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from Gregorian date in UTC at midnight
    pub fn from_gregorian_at_midnight(
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Self {
        Self::maybe_from_gregorian(year, month, day, 0, 0, 0, 0, time_scale)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from Gregorian date in UTC at noon
    pub fn from_gregorian_at_noon(year: i32, month: u8, day: u8, time_scale: TimeScale) -> Self {
        Self::maybe_from_gregorian(year, month, day, 12, 0, 0, 0, time_scale)
            .expect("invalid Gregorian date")
    }

    #[must_use]
    /// Initialize from the Gregorian date and time (without the nanoseconds) in UTC
    pub fn from_gregorian_hms(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        time_scale: TimeScale,
    ) -> Self {
        Self::maybe_from_gregorian(year, month, day, hour, minute, second, 0, time_scale)
            .expect("invalid Gregorian date")
    }

    /// Converts a Gregorian date time in ISO8601 or RFC3339 format into an Epoch, accounting for the time zone designator and the time scale.
    ///
    /// # Definition
    /// 1. Time Zone Designator: this is either a `Z` (lower or upper case) to specify UTC, or an offset in hours and minutes off of UTC, such as `+01:00` for UTC plus one hour and zero minutes.
    /// 2. Time system (or time "scale"): UTC, TT, TAI, TDB, ET, etc.
    ///
    /// Converts an ISO8601 or RFC3339 datetime representation to an Epoch.
    /// If no time scale is specified, then UTC is assumed.
    /// A time scale may be specified _in addition_ to the format unless
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
    /// // Example from https://www.w3.org/TR/NOTE-datetime
    /// assert_eq!(
    ///     Epoch::from_gregorian_utc_hms(1994, 11, 5, 13, 15, 30),
    ///     Epoch::from_gregorian_str("1994-11-05T13:15:30Z").unwrap()
    /// );
    /// assert_eq!(
    ///     Epoch::from_gregorian_utc_hms(1994, 11, 5, 13, 15, 30),
    ///     Epoch::from_gregorian_str("1994-11-05T08:15:30-05:00").unwrap()
    /// );
    /// ```
    #[cfg(not(kani))]
    pub fn from_gregorian_str(s_in: &str) -> Result<Self, Errors> {
        // All of the integers in a date: year, month, day, hour, minute, second, subsecond, offset hours, offset minutes
        let mut decomposed = [0_i32; 9];
        // The parsed time scale, defaults to UTC
        let mut ts = TimeScale::UTC;
        // The offset sign, defaults to positive.
        let mut offset_sign = 1;

        // Previous index of interest in the string
        let mut prev_idx = 0;
        let mut cur_token = Token::Year;

        let s = s_in.trim();

        for (idx, char) in s.chars().enumerate() {
            if !char.is_numeric() || idx == s.len() - 1 {
                if cur_token == Token::Timescale {
                    // Then we match the timescale directly.
                    if idx != s.len() - 1 {
                        // We have some remaining characters, so let's parse those in the only formats we know.
                        ts = TimeScale::from_str(s[idx..].trim())?;
                    }
                    break;
                }
                let prev_token = cur_token;

                let pos = cur_token.gregorian_position().unwrap();

                let end_idx = if idx != s.len() - 1 || !char.is_numeric() {
                    // Only advance the token if we aren't at the end of the string
                    cur_token.advance_with(char)?;
                    idx
                } else {
                    idx + 1
                };

                if prev_idx > end_idx {
                    return Err(Errors::ParseError(ParsingErrors::ISO8601));
                }

                match lexical_core::parse(s[prev_idx..end_idx].as_bytes()) {
                    Ok(val) => {
                        // Check that this valid is OK for the token we're reading it as.
                        prev_token.value_ok(val)?;
                        // If these are the subseconds, we must convert them to nanoseconds
                        if prev_token == Token::Subsecond {
                            if end_idx - prev_idx != 9 {
                                decomposed[pos] =
                                    val * 10_i32.pow((9 - (end_idx - prev_idx)) as u32);
                            } else {
                                decomposed[pos] = val;
                            }
                        } else {
                            decomposed[pos] = val
                        }
                    }
                    Err(_) => return Err(Errors::ParseError(ParsingErrors::ISO8601)),
                }
                prev_idx = idx + 1;
                // If we are about to parse an hours offset, we need to set the sign now.
                if cur_token == Token::OffsetHours {
                    if &s[idx..idx + 1] == "-" {
                        offset_sign = -1;
                    }
                    prev_idx += 1;
                }
            }
        }

        let tz = if offset_sign > 0 {
            // We oppose the sign in the string to undo the offset
            -(i64::from(decomposed[7]) * Unit::Hour + i64::from(decomposed[8]) * Unit::Minute)
        } else {
            i64::from(decomposed[7]) * Unit::Hour + i64::from(decomposed[8]) * Unit::Minute
        };

        let epoch = Self::maybe_from_gregorian(
            decomposed[0],
            decomposed[1].try_into().unwrap(),
            decomposed[2].try_into().unwrap(),
            decomposed[3].try_into().unwrap(),
            decomposed[4].try_into().unwrap(),
            decomposed[5].try_into().unwrap(),
            decomposed[6].try_into().unwrap(),
            ts,
        )?;

        Ok(epoch + tz)
    }

    /// Initializes an Epoch from the provided Format.
    pub fn from_str_with_format(s_in: &str, format: Format) -> Result<Self, Errors> {
        format.parse(s_in)
    }

    /// Initializes an Epoch from the Format as a string.
    pub fn from_format_str(s_in: &str, format_str: &str) -> Result<Self, Errors> {
        Format::from_str(format_str)
            .map_err(Errors::ParseError)?
            .parse(s_in)
    }

    #[cfg(feature = "ut1")]
    #[must_use]
    /// Initialize an Epoch from the provided UT1 duration since 1900 January 01 at midnight
    ///
    /// # Warning
    /// The time scale of this Epoch will be set to TAI! This is to ensure that no additional computations will change the duration since it's stored in TAI.
    /// However, this also means that calling `to_duration()` on this Epoch will return the TAI duration and not the UT1 duration!
    pub fn from_ut1_duration(duration: Duration, provider: Ut1Provider) -> Self {
        let mut e = Self::from_tai_duration(duration);
        // Compute the TAI to UT1 offset at this time.
        // We have the time in TAI. But we were given UT1.
        // The offset is provided as offset = TAI - UT1 <=> TAI = UT1 + offset
        e.duration += e.ut1_offset(provider).unwrap_or(Duration::ZERO);
        e.time_scale = TimeScale::TAI;
        e
    }

    fn delta_et_tai(seconds: f64) -> f64 {
        // Calculate M, the mean anomaly.4
        let m = NAIF_M0 + seconds * NAIF_M1;
        // Calculate eccentric anomaly
        let e = m + NAIF_EB * m.sin();

        (TT_OFFSET_MS * Unit::Millisecond).to_seconds() + NAIF_K * e.sin()
    }

    fn inner_g(seconds: f64) -> f64 {
        use core::f64::consts::TAU;
        let g = TAU / 360.0 * 357.528 + 1.990_910_018_065_731e-7 * seconds;
        // Return gamma
        1.658e-3 * (g + 1.67e-2 * g.sin()).sin()
    }

    pub(crate) fn compute_gregorian(
        duration: Duration,
        ts: TimeScale,
    ) -> (i32, u8, u8, u8, u8, u8, u32) {
        let (sign, days, mut hours, minutes, seconds, milliseconds, microseconds, nanos) =
            (duration + ts.gregorian_epoch_offset()).decompose();

        let days_f64 = if sign < 0 {
            -(days as f64)
        } else {
            days as f64
        };

        let (mut year, mut days_in_year) = div_rem_f64(days_f64, DAYS_PER_YEAR_NLD);
        year += HIFITIME_REF_YEAR;

        // Base calculation was on 365 days, so we need to remove one day in seconds per leap year
        // between 1900 and `year`
        if year >= HIFITIME_REF_YEAR {
            for year in HIFITIME_REF_YEAR..year {
                if is_leap_year(year) {
                    days_in_year -= 1.0;
                }
            }
        } else {
            for year in year..HIFITIME_REF_YEAR {
                if is_leap_year(year) {
                    days_in_year += 1.0;
                }
            }
        }

        // Get the month from the exact number of seconds between the start of the year and now
        let mut month = 1;
        let mut day;

        let mut days_so_far = 0.0;
        loop {
            let mut days_next_month = usual_days_per_month(month - 1) as f64;
            if month == 2 && is_leap_year(year) {
                days_next_month += 1.0;
            }

            if days_so_far + days_next_month > days_in_year || month == 12 {
                // We've found the month and can calculate the days
                day = if sign >= 0 {
                    days_in_year - days_so_far + 1.0
                } else {
                    days_in_year - days_so_far
                };
                break;
            }

            // Otherwise, count up the number of days this year so far and keep track of the month.
            days_so_far += days_next_month;
            month += 1;
        }

        if hours >= 24 {
            hours -= 24;
            if year >= HIFITIME_REF_YEAR {
                day += 1.0;
            } else {
                day -= 1.0;
            }
        }

        if day <= 0.0 || days_in_year < 0.0 {
            // We've overflowed backward
            month = 12;
            year -= 1;
            // NOTE: Leap year is already accounted for in the TAI duration when counting backward.
            day = if days_in_year < 0.0 {
                days_in_year + usual_days_per_month(11) as f64 + 1.0
            } else {
                usual_days_per_month(11) as f64
            };
        }

        if sign < 0 {
            let time = Duration::compose(
                sign,
                0,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanos,
            );

            // Last check on the validity of the Gregorian date

            if time == Duration::ZERO || month == 12 && day == 32.0 {
                // We've underflowed since we're before 1900.
                year += 1;
                month = 1;
                day = 1.0;
            }

            let (_, _, hours, minutes, seconds, milliseconds, microseconds, nanos) =
                (24 * Unit::Hour + time).decompose();

            (
                year,
                month,
                day as u8,
                hours as u8,
                minutes as u8,
                seconds as u8,
                (nanos
                    + microseconds * NANOSECONDS_PER_MICROSECOND
                    + milliseconds * NANOSECONDS_PER_MILLISECOND) as u32,
            )
        } else {
            (
                year,
                month,
                day as u8,
                hours as u8,
                minutes as u8,
                seconds as u8,
                (nanos
                    + microseconds * NANOSECONDS_PER_MICROSECOND
                    + milliseconds * NANOSECONDS_PER_MILLISECOND) as u32,
            )
        }
    }

    /// Builds an Epoch from given `week`: elapsed weeks counter into the desired Time scale, and the amount of nanoseconds within that week.
    /// For example, this is how GPS vehicles describe a GPST epoch.
    ///
    /// Note that this constructor relies on 128 bit integer math and may be slow on embedded devices.
    #[must_use]
    pub fn from_time_of_week(week: u32, nanoseconds: u64, time_scale: TimeScale) -> Self {
        let mut nanos = i128::from(nanoseconds);
        nanos += i128::from(week) * Weekday::DAYS_PER_WEEK_I128 * i128::from(NANOSECONDS_PER_DAY);
        let duration = Duration::from_total_nanoseconds(nanos);
        Self::from_duration(duration, time_scale)
    }

    #[must_use]
    /// Builds a UTC Epoch from given `week`: elapsed weeks counter and "ns" amount of nanoseconds since closest Sunday Midnight.
    pub fn from_time_of_week_utc(week: u32, nanoseconds: u64) -> Self {
        Self::from_time_of_week(week, nanoseconds, TimeScale::UTC)
    }

    #[must_use]
    /// Builds an Epoch from the provided year, days in the year, and a time scale.
    ///
    /// # Limitations
    /// In the TDB or ET time scales, there may be an error of up to 750 nanoseconds when initializing an Epoch this way.
    /// This is because we first initialize the epoch in Gregorian scale and then apply the TDB/ET offset, but that offset actually depends on the precise time.
    ///
    /// # Day couting behavior
    ///
    /// The day counter starts at 01, in other words, 01 January is day 1 of the counter, as per the GPS specificiations.
    ///
    pub fn from_day_of_year(year: i32, days: f64, time_scale: TimeScale) -> Self {
        let start_of_year = Self::from_gregorian(year, 1, 1, 0, 0, 0, 0, time_scale);
        start_of_year + (days - 1.0) * Unit::Day
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Epoch {
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
        self.leap_seconds_with(iers_only, LatestLeapSeconds::default())
    }

    #[cfg(feature = "ut1")]
    /// Get the accumulated offset between this epoch and UT1, assuming that the provider includes all data.
    pub fn ut1_offset(&self, provider: Ut1Provider) -> Option<Duration> {
        for delta_tai_ut1 in provider.rev() {
            if self > &delta_tai_ut1.epoch {
                return Some(delta_tai_ut1.delta_tai_minus_ut1);
            }
        }
        None
    }

    /// Get the accumulated number of leap seconds up to this Epoch from the provided LeapSecondProvider.
    /// Returns None if the epoch is before 1960, year at which UTC was defined.
    ///
    /// # Why does this function return an `Option` when the other returns a value
    /// This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960.
    #[cfg(feature = "python")]
    pub fn leap_seconds_with_file(
        &self,
        iers_only: bool,
        provider: LeapSecondsFile,
    ) -> Option<f64> {
        self.leap_seconds_with(iers_only, provider)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    const fn init_from_tai_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tai_duration(duration)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch.
    fn init_from_tai_parts(_cls: &Bound<'_, PyType>, centuries: i16, nanoseconds: u64) -> Self {
        Self::from_tai_parts(centuries, nanoseconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    fn init_from_tai_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tai_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    fn init_from_tai_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_tai_days(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    fn init_from_utc_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_utc_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight
    fn init_from_utc_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_utc_days(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from given MJD in TAI time scale
    fn init_from_mjd_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_tai(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from given MJD in UTC time scale
    fn init_from_mjd_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_utc(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from given JDE in TAI time scale
    fn init_from_jde_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tai(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from given JDE in UTC time scale
    fn init_from_jde_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_utc(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    fn init_from_tt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tt_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    fn init_from_tt_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tt_duration(duration)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)
    fn init_from_et_seconds(_cls: &Bound<'_, PyType>, seconds_since_j2000: f64) -> Epoch {
        Self::from_et_seconds(seconds_since_j2000)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)
    fn init_from_et_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Self {
        Self::from_et_duration(duration_since_j2000)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    /// NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    /// In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct.
    fn init_from_tdb_seconds(_cls: &Bound<'_, PyType>, seconds_j2000: f64) -> Epoch {
        Self::from_tdb_seconds(seconds_j2000)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI.
    fn init_from_tdb_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Epoch {
        Self::from_tdb_duration(duration_since_j2000)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from the JDE days
    fn init_from_jde_et(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_et(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days
    fn init_from_jde_tdb(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tdb(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_gpst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gpst_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of days since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_gpst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gpst_days(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use GPS as a time source.
    fn init_from_gpst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gpst_nanoseconds(nanoseconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_qzsst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_qzsst_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of days since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_qzsst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_qzsst_days(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use QZSS as a time source.
    fn init_from_qzsst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_qzsst_nanoseconds(nanoseconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_gst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gst_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of days since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_gst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gst_days(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use GST as a time source.
    fn init_from_gst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gst_nanoseconds(nanoseconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_bdt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_bdt_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_bdt_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_bdt_days(days)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use BDT as a time source.
    fn init_from_bdt_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_bdt_nanoseconds(nanoseconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01.
    fn init_from_unix_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_unix_seconds(seconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01.
    fn init_from_unix_milliseconds(_cls: &Bound<'_, PyType>, milliseconds: f64) -> Self {
        Self::from_unix_milliseconds(milliseconds)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    fn init_from_gregorian(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        time_scale: TimeScale,
    ) -> Self {
        Self::from_gregorian(year, month, day, hour, minute, second, nanos, time_scale)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    fn init_from_gregorian_at_noon(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Self {
        Self::from_gregorian_at_noon(year, month, day, time_scale)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    fn init_from_gregorian_at_midnight(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Self {
        Self::from_gregorian_at_midnight(year, month, day, time_scale)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Attempts to build an Epoch from the provided Gregorian date and time in TAI.
    fn maybe_init_from_gregorian_tai(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, Errors> {
        Self::maybe_from_gregorian_tai(year, month, day, hour, minute, second, nanos)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Attempts to build an Epoch from the provided Gregorian date and time in the provided time scale.
    /// NOTE: If the time scale is TDB, this function assumes that the SPICE format is used
    #[allow(clippy::too_many_arguments)]
    fn maybe_init_from_gregorian(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        time_scale: TimeScale,
    ) -> Result<Self, Errors> {
        Self::maybe_from_gregorian(year, month, day, hour, minute, second, nanos, time_scale)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_tai if unsure.
    fn init_from_gregorian_tai(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Self {
        Self::from_gregorian_tai(year, month, day, hour, minute, second, nanos)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from the Gregorian date at midnight in TAI.
    fn init_from_gregorian_tai_at_midnight(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
    ) -> Self {
        Self::from_gregorian_tai_at_midnight(year, month, day)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from the Gregorian date at noon in TAI
    fn init_from_gregorian_tai_at_noon(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
    ) -> Self {
        Self::from_gregorian_tai_at_noon(year, month, day)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from the Gregorian date and time (without the nanoseconds) in TAI
    fn init_from_gregorian_tai_hms(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Self {
        Self::from_gregorian_tai_hms(year, month, day, hour, minute, second)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Attempts to build an Epoch from the provided Gregorian date and time in UTC.
    fn maybe_init_from_gregorian_utc(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, Errors> {
        Self::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanos)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_tai if unsure.
    fn init_from_gregorian_utc(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Self {
        Self::from_gregorian_utc(year, month, day, hour, minute, second, nanos)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from Gregorian date in UTC at midnight
    fn init_from_gregorian_utc_at_midnight(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
    ) -> Self {
        Self::from_gregorian_utc_at_midnight(year, month, day)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from Gregorian date in UTC at noon
    fn init_from_gregorian_utc_at_noon(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
    ) -> Self {
        Self::from_gregorian_utc_at_noon(year, month, day)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Initialize from the Gregorian date and time (without the nanoseconds) in UTC
    fn init_from_gregorian_utc_hms(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Self {
        Self::from_gregorian_utc_hms(year, month, day, hour, minute, second)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    /// Equivalent to `datetime.strptime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    fn strptime(_cls: &Bound<'_, PyType>, epoch_str: String, format_str: String) -> PyResult<Self> {
        Self::from_format_str(&epoch_str, &format_str).map_err(|e| PyErr::from(e))
    }

    #[cfg(feature = "python")]
    /// Equivalent to `datetime.strftime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    fn strftime(&self, format_str: String) -> PyResult<String> {
        use crate::efmt::Formatter;
        let fmt = Format::from_str(&format_str)
            .map_err(Errors::ParseError)
            .map_err(|e| PyErr::from(e))?;
        Ok(format!("{}", Formatter::new(*self, fmt)))
    }

    #[cfg(feature = "python")]
    /// Equivalent to `datetime.isoformat`, and truncated to 23 chars, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    fn isoformat(&self) -> PyResult<String> {
        Ok(self.to_isoformat())
    }

    #[cfg(feature = "std")]
    #[must_use]
    /// The standard ISO format of this epoch (six digits of subseconds), refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    pub fn to_isoformat(&self) -> String {
        use crate::efmt::consts::ISO8601_STD;
        use crate::efmt::Formatter;
        format!("{}", Formatter::new(*self, ISO8601_STD))[..26].to_string()
    }

    #[must_use]
    /// Returns this epoch with respect to the provided time scale.
    /// This is needed to correctly perform duration conversions in dynamical time scales (e.g. TDB).
    pub fn to_duration_in_time_scale(&self, ts: TimeScale) -> Duration {
        self.to_time_scale(ts).duration
    }

    /// Attempts to return the number of nanoseconds since the reference epoch of the provided time scale.
    /// This will return an overflow error if more than one century has past since the reference epoch in the provided time scale.
    /// If this is _not_ an issue, you should use `epoch.to_duration_in_time_scale().to_parts()` to retrieve both the centuries and the nanoseconds
    /// in that century.
    #[allow(clippy::wrong_self_convention)]
    fn to_nanoseconds_in_time_scale(&self, time_scale: TimeScale) -> Result<u64, Errors> {
        let (centuries, nanoseconds) = self.to_duration_in_time_scale(time_scale).to_parts();
        if centuries != 0 {
            Err(Errors::Overflow)
        } else {
            Ok(nanoseconds)
        }
    }

    /*
        /// Makes a copy of self and sets the duration and time scale appropriately given the new duration
        #[must_use]
        pub fn set(&self, new_duration: Duration) -> Self {
            match self.time_scale {
                TimeScale::TAI => Self::from_tai_duration(new_duration),
                TimeScale::TT => Self::from_tt_duration(new_duration),
                TimeScale::ET => Self::from_et_duration(new_duration),
                TimeScale::TDB => Self::from_tdb_duration(new_duration),
                TimeScale::UTC => Self::from_utc_duration(new_duration),
                // GPST and QZSST share the same properties
                TimeScale::GPST | TimeScale::QZSST => Self::from_gpst_duration(new_duration),
                TimeScale::GST => Self::from_gst_duration(new_duration),
                TimeScale::BDT => Self::from_bdt_duration(new_duration),
            }
        }
    */
    #[must_use]
    /// Returns the number of TAI seconds since J1900
    pub fn to_tai_seconds(&self) -> f64 {
        self.to_tai_duration().to_seconds()
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in TAI
    pub fn to_tai_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::TAI).duration
    }

    #[must_use]
    /// Returns the epoch as a floating point value in the provided unit
    pub fn to_tai(&self, unit: Unit) -> f64 {
        self.to_tai_duration().to_unit(unit)
    }

    #[must_use]
    /// Returns the TAI parts of this duration
    pub fn to_tai_parts(&self) -> (i16, u64) {
        self.to_tai_duration().to_parts()
    }

    #[must_use]
    /// Returns the number of days since J1900 in TAI
    pub fn to_tai_days(&self) -> f64 {
        self.to_tai(Unit::Day)
    }

    #[must_use]
    /// Returns the number of UTC seconds since the TAI epoch
    pub fn to_utc_seconds(&self) -> f64 {
        self.to_utc(Unit::Second)
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UTC
    pub fn to_utc_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::UTC).duration
    }

    #[must_use]
    /// Returns the number of UTC seconds since the TAI epoch
    pub fn to_utc(&self, unit: Unit) -> f64 {
        self.to_utc_duration().to_unit(unit)
    }

    #[must_use]
    /// Returns the number of UTC days since the TAI epoch
    pub fn to_utc_days(&self) -> f64 {
        self.to_utc(Unit::Day)
    }

    #[must_use]
    /// `as_mjd_days` creates an Epoch from the provided Modified Julian Date in days as explained
    /// [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
    pub fn to_mjd_tai_days(&self) -> f64 {
        self.to_mjd_tai(Unit::Day)
    }

    #[must_use]
    /// Returns the Modified Julian Date in seconds TAI.
    pub fn to_mjd_tai_seconds(&self) -> f64 {
        self.to_mjd_tai(Unit::Second)
    }

    #[must_use]
    /// Returns this epoch as a duration in the requested units in MJD TAI
    pub fn to_mjd_tai(&self, unit: Unit) -> f64 {
        (self.to_tai_duration() + Unit::Day * J1900_OFFSET).to_unit(unit)
    }

    #[must_use]
    /// Returns the Modified Julian Date in days UTC.
    pub fn to_mjd_utc_days(&self) -> f64 {
        self.to_mjd_utc(Unit::Day)
    }

    #[must_use]
    /// Returns the Modified Julian Date in the provided unit in UTC.
    pub fn to_mjd_utc(&self, unit: Unit) -> f64 {
        (self.to_utc_duration() + Unit::Day * J1900_OFFSET).to_unit(unit)
    }

    #[must_use]
    /// Returns the Modified Julian Date in seconds UTC.
    pub fn to_mjd_utc_seconds(&self) -> f64 {
        self.to_mjd_utc(Unit::Second)
    }

    #[must_use]
    /// Returns the Julian days from epoch 01 Jan -4713, 12:00 (noon)
    /// as explained in "Fundamentals of astrodynamics and applications", Vallado et al.
    /// 4th edition, page 182, and on [Wikipedia](https://en.wikipedia.org/wiki/Julian_day).
    pub fn to_jde_tai_days(&self) -> f64 {
        self.to_jde_tai(Unit::Day)
    }

    #[must_use]
    /// Returns the Julian Days from epoch 01 Jan -4713 12:00 (noon) in desired Duration::Unit
    pub fn to_jde_tai(&self, unit: Unit) -> f64 {
        self.to_jde_tai_duration().to_unit(unit)
    }

    #[must_use]
    /// Returns the Julian Days from epoch 01 Jan -4713 12:00 (noon) as a Duration
    pub fn to_jde_tai_duration(&self) -> Duration {
        self.to_tai_duration() + Unit::Day * J1900_OFFSET + Unit::Day * MJD_OFFSET
    }

    #[must_use]
    /// Returns the Julian seconds in TAI.
    pub fn to_jde_tai_seconds(&self) -> f64 {
        self.to_jde_tai(Unit::Second)
    }

    #[must_use]
    /// Returns the Julian days in UTC.
    pub fn to_jde_utc_days(&self) -> f64 {
        self.to_jde_utc_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the Julian days in UTC as a `Duration`
    pub fn to_jde_utc_duration(&self) -> Duration {
        self.to_utc_duration() + Unit::Day * (J1900_OFFSET + MJD_OFFSET)
    }

    #[must_use]
    /// Returns the Julian Days in UTC seconds.
    pub fn to_jde_utc_seconds(&self) -> f64 {
        self.to_jde_utc_duration().to_seconds()
    }

    #[must_use]
    /// Returns seconds past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn to_tt_seconds(&self) -> f64 {
        self.to_tt_duration().to_seconds()
    }

    #[must_use]
    /// Returns `Duration` past TAI epoch in Terrestrial Time (TT).
    pub fn to_tt_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::TT).duration
    }

    #[must_use]
    /// Returns days past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn to_tt_days(&self) -> f64 {
        self.to_tt_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the centuries passed J2000 TT
    pub fn to_tt_centuries_j2k(&self) -> f64 {
        (self.to_tt_duration() - Unit::Second * ET_EPOCH_S).to_unit(Unit::Century)
    }

    #[must_use]
    /// Returns the duration past J2000 TT
    pub fn to_tt_since_j2k(&self) -> Duration {
        self.to_tt_duration() - Unit::Second * ET_EPOCH_S
    }

    #[must_use]
    /// Returns days past Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn to_jde_tt_days(&self) -> f64 {
        self.to_jde_tt_duration().to_unit(Unit::Day)
    }

    #[must_use]
    pub fn to_jde_tt_duration(&self) -> Duration {
        self.to_tt_duration() + Unit::Day * (J1900_OFFSET + MJD_OFFSET)
    }

    #[must_use]
    /// Returns days past Modified Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn to_mjd_tt_days(&self) -> f64 {
        self.to_mjd_tt_duration().to_unit(Unit::Day)
    }

    #[must_use]
    pub fn to_mjd_tt_duration(&self) -> Duration {
        self.to_tt_duration() + Unit::Day * J1900_OFFSET
    }

    #[must_use]
    /// Returns seconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn to_gpst_seconds(&self) -> f64 {
        self.to_gpst_duration().to_seconds()
    }

    #[must_use]
    /// Returns `Duration` past GPS time Epoch.
    pub fn to_gpst_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::GPST).duration
    }

    /// Returns nanoseconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// NOTE: This function will return an error if the centuries past GPST time are not zero.
    pub fn to_gpst_nanoseconds(&self) -> Result<u64, Errors> {
        self.to_nanoseconds_in_time_scale(TimeScale::GPST)
    }

    #[must_use]
    /// Returns days past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn to_gpst_days(&self) -> f64 {
        self.to_gpst_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns seconds past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn to_qzsst_seconds(&self) -> f64 {
        self.to_qzsst_duration().to_seconds()
    }

    #[must_use]
    /// Returns `Duration` past QZSS time Epoch.
    pub fn to_qzsst_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::QZSST).duration
    }

    /// Returns nanoseconds past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// NOTE: This function will return an error if the centuries past QZSST time are not zero.
    pub fn to_qzsst_nanoseconds(&self) -> Result<u64, Errors> {
        self.to_nanoseconds_in_time_scale(TimeScale::QZSST)
    }

    #[must_use]
    /// Returns days past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    pub fn to_qzsst_days(&self) -> f64 {
        self.to_gpst_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns seconds past GST (Galileo) Time Epoch
    pub fn to_gst_seconds(&self) -> f64 {
        self.to_gst_duration().to_seconds()
    }

    #[must_use]
    /// Returns `Duration` past GST (Galileo) time Epoch.
    pub fn to_gst_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::GST).duration
    }

    /// Returns nanoseconds past GST (Galileo) Time Epoch, starting on August 21st 1999 Midnight UT
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// NOTE: This function will return an error if the centuries past GST time are not zero.
    pub fn to_gst_nanoseconds(&self) -> Result<u64, Errors> {
        self.to_nanoseconds_in_time_scale(TimeScale::GST)
    }

    #[must_use]
    /// Returns days past GST (Galileo) Time Epoch,
    /// starting on August 21st 1999 Midnight UT
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    pub fn to_gst_days(&self) -> f64 {
        self.to_gst_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns seconds past BDT (BeiDou) Time Epoch
    pub fn to_bdt_seconds(&self) -> f64 {
        self.to_bdt_duration().to_seconds()
    }

    #[must_use]
    /// Returns `Duration` past BDT (BeiDou) time Epoch.
    pub fn to_bdt_duration(&self) -> Duration {
        self.to_tai_duration() - BDT_REF_EPOCH.to_tai_duration()
    }

    #[must_use]
    /// Returns days past BDT (BeiDou) Time Epoch, defined as Jan 01 2006 UTC
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    pub fn to_bdt_days(&self) -> f64 {
        self.to_bdt_duration().to_unit(Unit::Day)
    }

    /// Returns nanoseconds past BDT (BeiDou) Time Epoch, defined as Jan 01 2006 UTC
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// NOTE: This function will return an error if the centuries past GST time are not zero.
    pub fn to_bdt_nanoseconds(&self) -> Result<u64, Errors> {
        self.to_nanoseconds_in_time_scale(TimeScale::BDT)
    }

    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    /// Returns the Duration since the UNIX epoch UTC midnight 01 Jan 1970.
    fn to_unix_duration(&self) -> Duration {
        self.to_duration_in_time_scale(TimeScale::UTC) - UNIX_REF_EPOCH.to_utc_duration()
    }

    #[must_use]
    /// Returns the duration since the UNIX epoch in the provided unit.
    pub fn to_unix(&self, unit: Unit) -> f64 {
        self.to_unix_duration().to_unit(unit)
    }

    #[must_use]
    /// Returns the number seconds since the UNIX epoch defined 01 Jan 1970 midnight UTC.
    pub fn to_unix_seconds(&self) -> f64 {
        self.to_unix(Unit::Second)
    }

    #[must_use]
    /// Returns the number milliseconds since the UNIX epoch defined 01 Jan 1970 midnight UTC.
    pub fn to_unix_milliseconds(&self) -> f64 {
        self.to_unix(Unit::Millisecond)
    }

    #[must_use]
    /// Returns the number days since the UNIX epoch defined 01 Jan 1970 midnight UTC.
    pub fn to_unix_days(&self) -> f64 {
        self.to_unix(Unit::Day)
    }

    #[must_use]
    /// Returns the Ephemeris Time seconds past 2000 JAN 01 midnight, matches NASA/NAIF SPICE.
    pub fn to_et_seconds(&self) -> f64 {
        self.to_et_duration().to_seconds()
    }

    #[must_use]
    /// Returns the Ephemeris Time in duration past 1900 JAN 01 at noon.
    /// **Only** use this if the subsequent computation expect J1900 seconds.
    pub fn to_et_duration_since_j1900(&self) -> Duration {
        self.to_et_duration() + J2000_TO_J1900_DURATION
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
    pub fn to_et_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::ET).duration
    }

    #[must_use]
    /// Returns the Dynamics Barycentric Time (TDB) as a high precision Duration since J2000
    ///
    /// ## Algorithm
    /// Given the embedded sine functions in the equation to compute the difference between TDB and TAI from the number of TDB seconds
    /// past J2000, one cannot solve the revert the operation analytically. Instead, we iterate until the value no longer changes.
    ///
    /// 1. Assume that the TAI duration is in fact the TDB seconds from J2000.
    /// 2. Offset to J2000 because `Epoch` stores everything in the J1900 but the TDB duration is in J2000.
    /// 3. Compute the offset `g` due to the TDB computation with the current value of the TDB seconds (defined in step 1).
    /// 4. Subtract that offset to the latest TDB seconds and store this as a new candidate for the true TDB seconds value.
    /// 5. Compute the difference between this candidate and the previous one. If the difference is less than one nanosecond, stop iteration.
    /// 6. Set the new candidate as the TDB seconds since J2000 and loop until step 5 breaks the loop, or we've done five iterations.
    /// 7. At this stage, we have a good approximation of the TDB seconds since J2000.
    /// 8. Reverse the algorithm given that approximation: compute the `g` offset, compute the difference between TDB and TAI, add the TT offset (32.184 s), and offset by the difference between J1900 and J2000.
    pub fn to_tdb_duration(&self) -> Duration {
        self.to_time_scale(TimeScale::TDB).duration
    }

    #[must_use]
    /// Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)
    pub fn to_tdb_seconds(&self) -> f64 {
        self.to_tdb_duration().to_seconds()
    }

    #[must_use]
    /// Returns the Dynamics Barycentric Time (TDB) as a high precision Duration with reference epoch of 1900 JAN 01 at noon.
    /// **Only** use this if the subsequent computation expect J1900 seconds.
    pub fn to_tdb_duration_since_j1900(&self) -> Duration {
        self.to_tdb_duration() + J2000_TO_J1900_DURATION
    }

    #[must_use]
    /// Returns the Ephemeris Time JDE past epoch
    pub fn to_jde_et_days(&self) -> f64 {
        self.to_jde_et_duration().to_unit(Unit::Day)
    }

    #[must_use]
    pub fn to_jde_et_duration(&self) -> Duration {
        self.to_et_duration()
            + Unit::Day * (J1900_OFFSET + MJD_OFFSET)
            + TimeScale::ET.prime_epoch_offset()
    }

    #[must_use]
    pub fn to_jde_et(&self, unit: Unit) -> f64 {
        self.to_jde_et_duration().to_unit(unit)
    }

    #[must_use]
    pub fn to_jde_tdb_duration(&self) -> Duration {
        self.to_tdb_duration()
            + Unit::Day * (J1900_OFFSET + MJD_OFFSET)
            + TimeScale::TDB.prime_epoch_offset()
    }

    #[must_use]
    /// Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)
    pub fn to_jde_tdb_days(&self) -> f64 {
        self.to_jde_tdb_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the number of days since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)
    pub fn to_tdb_days_since_j2000(&self) -> f64 {
        self.to_tdb_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the number of centuries since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)
    pub fn to_tdb_centuries_since_j2000(&self) -> f64 {
        self.to_tdb_duration().to_unit(Unit::Century)
    }

    #[must_use]
    /// Returns the number of days since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn to_et_days_since_j2000(&self) -> f64 {
        self.to_et_duration().to_unit(Unit::Day)
    }

    #[must_use]
    /// Returns the number of centuries since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)
    pub fn to_et_centuries_since_j2000(&self) -> f64 {
        self.to_et_duration().to_unit(Unit::Century)
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
    /// let (y, m, d, h, min, s, _) = dt.to_gregorian_utc();
    /// assert_eq!(y, 2017);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 14);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 31);
    /// assert_eq!(s, 55);
    /// #[cfg(feature = "std")]
    /// assert_eq!("2017-01-14T00:31:55 UTC", format!("{dt}"));
    /// ```
    pub fn to_gregorian_utc(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        let ts = TimeScale::UTC;
        Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts)
    }

    #[must_use]
    /// Converts the Epoch to the Gregorian TAI equivalent as (year, month, day, hour, minute, second).
    /// WARNING: Nanoseconds are lost in this conversion!
    ///
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    /// let dt = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    /// let (y, m, d, h, min, s, _) = dt.to_gregorian_tai();
    /// assert_eq!(y, 1972);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 1);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 0);
    /// assert_eq!(s, 0);
    /// ```
    pub fn to_gregorian_tai(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        let ts = TimeScale::TAI;
        Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts)
    }

    #[cfg(feature = "ut1")]
    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UT1
    pub fn to_ut1_duration(&self, provider: Ut1Provider) -> Duration {
        // TAI = UT1 + offset <=> UTC = TAI - offset
        self.to_tai_duration() - self.ut1_offset(provider).unwrap_or(Duration::ZERO)
    }

    #[cfg(feature = "ut1")]
    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UT1
    pub fn to_ut1(&self, provider: Ut1Provider) -> Self {
        Self::from_tai_duration(self.to_ut1_duration(provider))
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

    #[must_use]
    /// Returns the duration since the start of the year
    pub fn duration_in_year(&self) -> Duration {
        let start_of_year = Self::from_gregorian(self.year(), 1, 1, 0, 0, 0, 0, self.time_scale);
        self.duration - start_of_year.duration
    }

    #[must_use]
    /// Returns the number of days since the start of the year.
    pub fn day_of_year(&self) -> f64 {
        self.duration_in_year().to_unit(Unit::Day) + 1.0
    }

    #[must_use]
    /// Returns the number of Gregorian years of this epoch in the current time scale.
    pub fn year(&self) -> i32 {
        Self::compute_gregorian(self.duration, self.time_scale).0
    }

    #[must_use]
    /// Returns the year and the days in the year so far (days of year).
    pub fn year_days_of_year(&self) -> (i32, f64) {
        (self.year(), self.day_of_year())
    }

    /// Returns the hours of the Gregorian representation  of this epoch in the time scale it was initialized in.
    pub fn hours(&self) -> u64 {
        self.duration.decompose().2
    }

    /// Returns the minutes of the Gregorian representation  of this epoch in the time scale it was initialized in.
    pub fn minutes(&self) -> u64 {
        self.duration.decompose().3
    }

    /// Returns the seconds of the Gregorian representation  of this epoch in the time scale it was initialized in.
    pub fn seconds(&self) -> u64 {
        self.duration.decompose().4
    }

    /// Returns the milliseconds of the Gregorian representation  of this epoch in the time scale it was initialized in.
    pub fn milliseconds(&self) -> u64 {
        self.duration.decompose().5
    }

    /// Returns the microseconds of the Gregorian representation  of this epoch in the time scale it was initialized in.
    pub fn microseconds(&self) -> u64 {
        self.duration.decompose().6
    }

    /// Returns the nanoseconds of the Gregorian representation  of this epoch in the time scale it was initialized in.
    pub fn nanoseconds(&self) -> u64 {
        self.duration.decompose().7
    }

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

    pub fn month_name(&self) -> MonthName {
        let month = Self::compute_gregorian(self.duration, self.time_scale).1;
        month.into()
    }

    // Python helpers

    #[cfg(feature = "python")]
    #[new]
    fn new_py(string_repr: String) -> PyResult<Self> {
        match Self::from_str(&string_repr) {
            Ok(d) => Ok(d),
            Err(e) => Err(PyErr::from(e)),
        }
    }

    #[cfg(feature = "python")]
    fn __getnewargs__(&self) -> Result<(String,), PyErr> {
        Ok((format!("{self:?}"),))
    }

    #[cfg(feature = "python")]
    #[classmethod]
    fn system_now(_cls: &Bound<'_, PyType>) -> Result<Self, Errors> {
        Self::now()
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
    }

    #[cfg(feature = "python")]
    fn __add__(&self, duration: Duration) -> Self {
        *self + duration
    }

    #[cfg(feature = "python")]
    fn __sub__(&self, duration: Duration) -> Self {
        *self - duration
    }

    #[cfg(feature = "python")]
    fn timedelta(&self, other: Self) -> Duration {
        *self - other
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Lt => *self < other,
            CompareOp::Le => *self <= other,
            CompareOp::Eq => *self == other,
            CompareOp::Ne => *self != other,
            CompareOp::Gt => *self > other,
            CompareOp::Ge => *self >= other,
        }
    }

    #[cfg(feature = "std")]
    #[must_use]
    /// Converts the Epoch to Gregorian in the provided time scale and in the ISO8601 format with the time scale appended to the string
    pub fn to_gregorian_str(&self, time_scale: TimeScale) -> String {
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.duration, self.time_scale);

        if nanos == 0 {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, time_scale
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, time_scale
            )
        }
    }

    #[cfg(feature = "std")]
    /// Returns this epoch in UTC in the RFC3339 format
    pub fn to_rfc3339(&self) -> String {
        let ts = TimeScale::UTC;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
                y, mm, dd, hh, min, s
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}+00:00",
                y, mm, dd, hh, min, s, nanos
            )
        }
    }

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
}

#[cfg(not(kani))]
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
    /// use core::str::FromStr;
    ///
    /// assert!(Epoch::from_str("JD 2452312.500372511 TDB").is_ok());
    /// assert!(Epoch::from_str("JD 2452312.500372511 ET").is_ok());
    /// assert!(Epoch::from_str("JD 2452312.500372511 TAI").is_ok());
    /// assert!(Epoch::from_str("MJD 51544.5 TAI").is_ok());
    /// assert!(Epoch::from_str("SEC 0.5 TAI").is_ok());
    /// assert!(Epoch::from_str("SEC 66312032.18493909 TDB").is_ok());
    /// ```
    fn from_str(s_in: &str) -> Result<Self, Self::Err> {
        let s = s_in.trim();

        if s.len() < 7 {
            // We need at least seven characters for a valid epoch
            Err(Errors::ParseError(ParsingErrors::UnknownFormat))
        } else {
            let format = if &s[..2] == "JD" {
                "JD"
            } else if &s[..3] == "MJD" {
                "MJD"
            } else if &s[..3] == "SEC" {
                "SEC"
            } else {
                // Not a valid format, hopefully it's a Gregorian date.
                return Self::from_gregorian_str(s_in);
            };

            // This is a valid numerical format.
            // Parse the time scale from the last three characters (TS trims white spaces).
            let ts = TimeScale::from_str(&s[s.len() - 3..])?;
            // Iterate through the string to figure out where the numeric data starts and ends.
            let start_idx = format.len();
            let num_str = s[start_idx..s.len() - ts.formatted_len()].trim();
            let value: f64 = match lexical_core::parse(num_str.as_bytes()) {
                Ok(val) => val,
                Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
            };

            match format {
                "JD" => match ts {
                    TimeScale::ET => Ok(Self::from_jde_et(value)),
                    TimeScale::TAI => Ok(Self::from_jde_tai(value)),
                    TimeScale::TDB => Ok(Self::from_jde_tdb(value)),
                    TimeScale::UTC => Ok(Self::from_jde_utc(value)),
                    _ => Err(Errors::ParseError(ParsingErrors::UnsupportedTimeSystem)),
                },
                "MJD" => match ts {
                    TimeScale::TAI => Ok(Self::from_mjd_tai(value)),
                    TimeScale::UTC | TimeScale::GPST | TimeScale::BDT | TimeScale::GST => {
                        Ok(Self::from_mjd_in_time_scale(value, ts))
                    }
                    _ => Err(Errors::ParseError(ParsingErrors::UnsupportedTimeSystem)),
                },
                "SEC" => match ts {
                    TimeScale::TAI => Ok(Self::from_tai_seconds(value)),
                    TimeScale::ET => Ok(Self::from_et_seconds(value)),
                    TimeScale::TDB => Ok(Self::from_tdb_seconds(value)),
                    TimeScale::TT => Ok(Self::from_tt_seconds(value)),
                    ts => {
                        let secs = value * Unit::Second;
                        Ok(Self::from_duration(secs, ts))
                    }
                },
                _ => Err(Errors::ParseError(ParsingErrors::UnknownFormat)),
            }
        }
    }
}

impl fmt::Debug for Epoch {
    /// Print this epoch in Gregorian in the time scale used at initialization
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.duration, self.time_scale);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, self.time_scale
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, self.time_scale
            )
        }
    }
}

impl fmt::Display for Epoch {
    /// The default format of an epoch is in UTC
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::UTC;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::LowerHex for Epoch {
    /// Prints the Epoch in TAI
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::TAI;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::UpperHex for Epoch {
    /// Prints the Epoch in TT
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::TT;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::LowerExp for Epoch {
    /// Prints the Epoch in TDB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::TDB;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::UpperExp for Epoch {
    /// Prints the Epoch in ET
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::ET;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::Pointer for Epoch {
    /// Prints the Epoch in UNIX
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_unix_seconds())
    }
}

impl fmt::Octal for Epoch {
    /// Prints the Epoch in GPS
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_gpst_nanoseconds().unwrap())
    }
}

#[must_use]
/// Returns true if the provided Gregorian date is valid. Leap second days may have 60 seconds.
pub const fn is_gregorian_valid(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanos: u32,
) -> bool {
    let max_seconds = if (month == 12 || month == 6)
        && day == usual_days_per_month(month - 1)
        && hour == 23
        && minute == 59
        && ((month == 6 && july_years(year)) || (month == 12 && january_years(year + 1)))
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
        || nanos > NANOSECONDS_PER_SECOND_U32
    {
        return false;
    }
    if day > usual_days_per_month(month - 1) && (month != 2 || !is_leap_year(year)) {
        // Not in February or not a leap year
        return false;
    }
    true
}

/// `is_leap_year` returns whether the provided year is a leap year or not.
/// Tests for this function are part of the Datetime tests.
const fn is_leap_year(year: i32) -> bool {
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
fn test_days_et_j2000() {
    /*
    WARNING: THIS ASSUMES THE UTC EPOCH in SPICE!
    Verification via SPICE: load naif0012.txt (contains leap seconds until 2017-JAN-1)
    In [6]: sp.str2et("2022-11-30 12:00:00")
    Out[6]: 723081669.183061
    In [7]: from hifitime import *
    In [8]: Unit.Second*723081669.183061
    Out[8]: 8369 days 1 min 9 s 183 ms 60 μs 992 ns @ 0x7fcd1559ef80
    In [9]: (Unit.Second*723081669.183061).to_unit(Unit.Day)
    Out[9]: 8369.000800729873
    In [10]: (Unit.Second*723081669.183061).to_unit(Unit.Century)
    Out[10]: 0.2291307542978747

     */
    let e = Epoch::from_tai_duration(Duration::from_parts(1, 723038437000000000));
    let days_d = e.to_et_days_since_j2000();
    let centuries_t = e.to_et_centuries_since_j2000();
    assert!((days_d - 8369.000800729873).abs() < f64::EPSILON);
    assert!((centuries_t - 0.2291307542978747).abs() < f64::EPSILON);
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

#[test]
fn cumulative_days_for_month() {
    assert_eq!(
        CUMULATIVE_DAYS_FOR_MONTH,
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]
    )
}

#[test]
#[cfg(feature = "serde")]
fn test_serdes() {
    let e = Epoch::from_gregorian_utc(2020, 01, 01, 0, 0, 0, 0);
    let content = r#""2020-01-01T00:00:00 UTC""#;
    assert_eq!(content, serde_json::to_string(&e).unwrap());
    let parsed: Epoch = serde_json::from_str(content).unwrap();
    assert_eq!(e, parsed);
}

#[cfg(kani)]
#[kani::proof]
fn formal_epoch_reciprocity_tai() {
    let duration: Duration = kani::any();

    // TAI
    let time_scale: TimeScale = TimeScale::TAI;
    let epoch: Epoch = Epoch::from_duration(duration, time_scale);
    assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_tai_seconds(seconds);
    }

    let days: f64 = kani::any();
    if days.is_finite() {
        let _ = Epoch::from_tai_days(days);
    }
}

#[cfg(kani)]
#[kani::proof]
fn formal_epoch_reciprocity_tt() {
    let duration: Duration = kani::any();

    // TT -- Check valid within bounds of (MIN + TT Offset) and (MAX - TT Offset)
    if duration > Duration::MIN + TT_OFFSET_MS * Unit::Millisecond
        && duration < Duration::MAX - TT_OFFSET_MS * Unit::Millisecond
    {
        let time_scale: TimeScale = TimeScale::TT;
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_tt_seconds(seconds);
    }
    // No TT Days initializer
}

// Skip ET, kani chokes on the Newton Raphson loop.

// Skip TDB
// #[cfg(kani)]
// #[kani::proof]
#[test]
fn formal_epoch_reciprocity_tdb() {
    // let duration: Duration = kani::any();
    let duration = Duration::from_parts(19510, 3155759999999997938);

    // TDB
    let ts_offset = TimeScale::TDB.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        // We guard TDB from durations that are would hit the MIN or the MAX.
        // TDB is centered on J2000 but the Epoch is on J1900. So on initialization, we offset by one century and twelve hours.
        // If the duration is too close to the Duration bounds, then the TDB initialization and retrieval will fail (because the bounds will have been hit).

        let time_scale: TimeScale = TimeScale::TDB;
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        let out_duration = epoch.to_duration_in_time_scale(time_scale);
        assert_eq!((out_duration - duration).to_seconds(), 0.0);
        assert_eq!(out_duration.centuries, duration.centuries);
        assert_eq!(out_duration.nanoseconds, duration.nanoseconds);
        let error = (out_duration.nanoseconds - duration.nanoseconds) as f64;
        assert!(error.abs() < 500_000.0, "error: {}", error);
    }
}

// Skip UTC, kani chokes on the leap seconds counting.

#[cfg(kani)]
#[kani::proof]
#[test]
fn formal_epoch_reciprocity_gpst() {
    let duration: Duration = kani::any();

    // GPST
    let time_scale: TimeScale = TimeScale::GPST;
    let ts_offset = TimeScale::GPST.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_gpst_seconds(seconds);
    }

    let _ = Epoch::from_gpst_nanoseconds(kani::any());
}

#[cfg(kani)]
#[kani::proof]
fn formal_epoch_reciprocity_gst() {
    let duration: Duration = kani::any();

    // GST
    let time_scale: TimeScale = TimeScale::GST;
    let ts_offset = TimeScale::GST.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_gst_seconds(seconds);
    }

    let days: f64 = kani::any();
    if days.is_finite() {
        let _ = Epoch::from_gst_days(days);
    }

    let _ = Epoch::from_gst_nanoseconds(kani::any());
}

#[cfg(kani)]
#[kani::proof]
fn formal_epoch_reciprocity_bdt() {
    let duration: Duration = kani::any();

    // BDT
    let time_scale: TimeScale = TimeScale::BDT;
    let ts_offset = TimeScale::BDT.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_bdt_seconds(seconds);
    }

    let days: f64 = kani::any();
    if days.is_finite() {
        let _ = Epoch::from_bdt_days(days);
    }

    let _ = Epoch::from_bdt_nanoseconds(kani::any());
}

#[cfg(kani)]
#[kani::proof]
fn formal_epoch_julian() {
    let days: f64 = kani::any();

    if days.is_finite() {
        // The initializers will fail on subnormal days.
        let _ = Epoch::from_mjd_bdt(days);
        let _ = Epoch::from_mjd_gpst(days);
        let _ = Epoch::from_mjd_gst(days);
        let _ = Epoch::from_mjd_tai(days);
        let _ = Epoch::from_jde_bdt(days);
        let _ = Epoch::from_jde_gpst(days);
        let _ = Epoch::from_jde_gst(days);
        let _ = Epoch::from_jde_tai(days);
        let _ = Epoch::from_jde_et(days);
        let _ = Epoch::from_jde_tai(days);
    }
}
