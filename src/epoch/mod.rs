/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

mod formatting;
mod gregorian;
mod ops;
mod with_funcs;

#[cfg(feature = "std")]
mod leap_seconds_file;
#[cfg(feature = "std")]
mod system_time;

#[cfg(kani)]
mod kani_verif;

#[cfg(feature = "ut1")]
pub mod ut1;

pub mod leap_seconds;

use crate::duration::{Duration, Unit};
use crate::efmt::format::Format;
use crate::errors::{DurationError, ParseSnafu};
use crate::leap_seconds::{LatestLeapSeconds, LeapSecondProvider};
use crate::Weekday;
use crate::{
    EpochError, MonthName, TimeScale, TimeUnits, BDT_REF_EPOCH, ET_EPOCH_S, GPST_REF_EPOCH,
    GST_REF_EPOCH, MJD_J1900, MJD_OFFSET, NANOSECONDS_PER_DAY, QZSST_REF_EPOCH, UNIX_REF_EPOCH,
};
use core::cmp::Eq;
use core::str::FromStr;
pub use gregorian::is_gregorian_valid;
use snafu::ResultExt;

#[cfg(not(kani))]
use crate::ParsingError;

#[cfg(kani)]
use kani::assert;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(not(kani))]
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

pub(crate) const TT_OFFSET_MS: i64 = 32_184;
pub(crate) const ET_OFFSET_US: i64 = 32_184_935;

/// NAIF leap second kernel data for M_0 used to calculate the mean anomaly of the heliocentric orbit of the Earth-Moon barycenter.
pub const NAIF_M0: f64 = 6.239996;
/// NAIF leap second kernel data for M_1 used to calculate the mean anomaly of the heliocentric orbit of the Earth-Moon barycenter.
pub const NAIF_M1: f64 = 1.99096871e-7;
/// NAIF leap second kernel data for EB used to calculate the eccentric anomaly of the heliocentric orbit of the Earth-Moon barycenter.
pub const NAIF_EB: f64 = 1.671e-2;
/// NAIF leap second kernel data used to calculate the difference between ET and TAI.
pub const NAIF_K: f64 = 1.657e-3;

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

// Defines the methods that should be classmethods in Python, but must be redefined as per https://github.com/PyO3/pyo3/issues/1003#issuecomment-844433346
impl Epoch {
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
        Self::from_tai_duration((days - MJD_J1900) * Unit::Day)
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
        Self::from_tai_duration((days - MJD_J1900 - MJD_OFFSET) * Unit::Day)
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

    /// Initializes an Epoch from the provided Format.
    pub fn from_str_with_format(s_in: &str, format: Format) -> Result<Self, EpochError> {
        format.parse(s_in)
    }

    /// Initializes an Epoch from the Format as a string.
    pub fn from_format_str(s_in: &str, format_str: &str) -> Result<Self, EpochError> {
        Format::from_str(format_str)
            .with_context(|_| ParseSnafu {
                details: "when using format string",
            })?
            .parse(s_in)
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

    #[cfg(feature = "std")]
    #[must_use]
    /// The standard ISO format of this epoch (six digits of subseconds) in the _current_ time scale, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options.
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
    fn to_nanoseconds_in_time_scale(&self, time_scale: TimeScale) -> Result<u64, EpochError> {
        let (centuries, nanoseconds) = self.to_duration_in_time_scale(time_scale).to_parts();
        if centuries != 0 {
            Err(EpochError::Duration {
                source: DurationError::Overflow,
            })
        } else {
            Ok(nanoseconds)
        }
    }

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
        (self.to_tai_duration() + Unit::Day * MJD_J1900).to_unit(unit)
    }

    #[must_use]
    /// Returns the Modified Julian Date in days UTC.
    pub fn to_mjd_utc_days(&self) -> f64 {
        self.to_mjd_utc(Unit::Day)
    }

    #[must_use]
    /// Returns the Modified Julian Date in the provided unit in UTC.
    pub fn to_mjd_utc(&self, unit: Unit) -> f64 {
        (self.to_utc_duration() + Unit::Day * MJD_J1900).to_unit(unit)
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
        self.to_tai_duration() + Unit::Day * MJD_J1900 + Unit::Day * MJD_OFFSET
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
        self.to_utc_duration() + Unit::Day * (MJD_J1900 + MJD_OFFSET)
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
        self.to_tt_duration() + Unit::Day * (MJD_J1900 + MJD_OFFSET)
    }

    #[must_use]
    /// Returns days past Modified Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn to_mjd_tt_days(&self) -> f64 {
        self.to_mjd_tt_duration().to_unit(Unit::Day)
    }

    #[must_use]
    pub fn to_mjd_tt_duration(&self) -> Duration {
        self.to_tt_duration() + Unit::Day * MJD_J1900
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
    pub fn to_gpst_nanoseconds(&self) -> Result<u64, EpochError> {
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
    pub fn to_qzsst_nanoseconds(&self) -> Result<u64, EpochError> {
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
    pub fn to_gst_nanoseconds(&self) -> Result<u64, EpochError> {
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
    pub fn to_bdt_nanoseconds(&self) -> Result<u64, EpochError> {
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
    /// Returns the Ephemeris Time JDE past epoch
    pub fn to_jde_et_days(&self) -> f64 {
        self.to_jde_et_duration().to_unit(Unit::Day)
    }

    #[must_use]
    pub fn to_jde_et_duration(&self) -> Duration {
        self.to_et_duration()
            + Unit::Day * (MJD_J1900 + MJD_OFFSET)
            + TimeScale::ET.prime_epoch_offset()
    }

    #[must_use]
    pub fn to_jde_et(&self, unit: Unit) -> f64 {
        self.to_jde_et_duration().to_unit(unit)
    }

    #[must_use]
    pub fn to_jde_tdb_duration(&self) -> Duration {
        self.to_tdb_duration()
            + Unit::Day * (MJD_J1900 + MJD_OFFSET)
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

    pub fn month_name(&self) -> MonthName {
        let month = Self::compute_gregorian(self.duration, self.time_scale).1;
        month.into()
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
}

#[cfg(not(kani))]
impl FromStr for Epoch {
    type Err = EpochError;

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
            Err(EpochError::Parse {
                source: ParsingError::UnknownFormat,
                details: "less than 7 characters",
            })
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
            let ts = TimeScale::from_str(&s[s.len() - 3..]).with_context(|_| ParseSnafu {
                details: "parsing from string",
            })?;
            // Iterate through the string to figure out where the numeric data starts and ends.
            let start_idx = format.len();
            let num_str = s[start_idx..s.len() - ts.formatted_len()].trim();
            let value: f64 = match lexical_core::parse(num_str.as_bytes()) {
                Ok(val) => val,
                Err(_) => {
                    return Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "parsing as JD, MJD, or SEC",
                    })
                }
            };

            match format {
                "JD" => match ts {
                    TimeScale::ET => Ok(Self::from_jde_et(value)),
                    TimeScale::TAI => Ok(Self::from_jde_tai(value)),
                    TimeScale::TDB => Ok(Self::from_jde_tdb(value)),
                    TimeScale::UTC => Ok(Self::from_jde_utc(value)),
                    _ => Err(EpochError::Parse {
                        source: ParsingError::UnsupportedTimeSystem,
                        details: "for Julian Date",
                    }),
                },
                "MJD" => match ts {
                    TimeScale::TAI => Ok(Self::from_mjd_tai(value)),
                    TimeScale::UTC | TimeScale::GPST | TimeScale::BDT | TimeScale::GST => {
                        Ok(Self::from_mjd_in_time_scale(value, ts))
                    }
                    _ => Err(EpochError::Parse {
                        source: ParsingError::UnsupportedTimeSystem,
                        details: "for Modified Julian Date",
                    }),
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
                _ => Err(EpochError::Parse {
                    source: ParsingError::UnknownFormat,
                    details: "suffix not understood",
                }),
            }
        }
    }
}

fn div_rem_f64(me: f64, rhs: f64) -> (i32, f64) {
    ((div_euclid_f64(me, rhs) as i32), rem_euclid_f64(me, rhs))
}

fn div_euclid_f64(lhs: f64, rhs: f64) -> f64 {
    let q = (lhs / rhs).trunc();
    if lhs % rhs < 0.0 {
        if rhs > 0.0 {
            q - 1.0
        } else {
            q + 1.0
        }
    } else {
        q
    }
}

fn rem_euclid_f64(lhs: f64, rhs: f64) -> f64 {
    let r = lhs % rhs;
    if r < 0.0 {
        r + rhs.abs()
    } else {
        r
    }
}

#[cfg(test)]
mod ut_epoch {

    use super::{div_rem_f64, Duration, Epoch};

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
    #[cfg(feature = "serde")]
    fn test_serdes() {
        let e = Epoch::from_gregorian_utc_at_midnight(2020, 01, 01);
        let content = r#""2020-01-01T00:00:00 UTC""#;
        assert_eq!(content, serde_json::to_string(&e).unwrap());
        let parsed: Epoch = serde_json::from_str(content).unwrap();
        assert_eq!(e, parsed);
    }
}
