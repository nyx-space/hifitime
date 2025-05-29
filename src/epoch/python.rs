/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the implementations that are only built with the pyhon feature.

use crate::{prelude::Format, Duration, Epoch, HifitimeError, TimeScale};

use core::str::FromStr;

use crate::epoch::leap_seconds_file::LeapSecondsFile;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDateAccess, PyDateTime, PyTimeAccess, PyType, PyTzInfoAccess};

#[pymethods]
impl Epoch {
    #[classmethod]
    #[pyo3(name = "from_tai_duration")]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    /// :type duration: Duration
    /// :rtype: Epoch
    fn from_tai_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tai_duration(duration)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tai_duration` instead")]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    /// :type duration: Duration
    /// :rtype: Epoch
    const fn init_from_tai_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tai_duration(_cls, duration)
    }

    #[classmethod]
    #[pyo3(name = "from_tai_parts")]
    /// Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch.
    /// :type centuries: int
    /// :type nanoseconds: int
    /// :rtype: Epoch
    fn from_tai_parts(_cls: &Bound<'_, PyType>, centuries: i16, nanoseconds: u64) -> Self {
        Self::from_tai_parts(centuries, nanoseconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tai_parts` instead")]
    /// Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch.
    /// :type centuries: int
    /// :type nanoseconds: int
    /// :rtype: Epoch
    fn init_from_tai_parts(_cls: &Bound<'_, PyType>, centuries: i16, nanoseconds: u64) -> Self {
        Self::from_tai_parts(_cls, centuries, nanoseconds)
    }

    #[classmethod]
    #[pyo3(name = "from_tai_seconds")]
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_tai_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tai_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tai_seconds` instead")]
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_tai_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tai_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_tai_days")]
    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    /// :type days: float
    /// :rtype: Epoch
    fn from_tai_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_tai_days(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tai_days` instead")]
    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_tai_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_tai_days(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_utc_seconds")]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_utc_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_utc_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_utc_seconds` instead")]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_utc_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_utc_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_utc_days")]
    /// Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight
    /// :type days: float
    /// :rtype: Epoch
    fn from_utc_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_utc_days(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_utc_days` instead")]
    /// Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_utc_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_utc_days(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_mjd_tai")]
    /// Initialize an Epoch from given MJD in TAI time scale
    /// :type days: float
    /// :rtype: Epoch
    fn from_mjd_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_tai(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_mjd_tai` instead")]
    /// Initialize an Epoch from given MJD in TAI time scale
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_mjd_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_tai(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_mjd_utc")]
    /// Initialize an Epoch from given MJD in UTC time scale
    /// :type days: float
    /// :rtype: Epoch
    fn from_mjd_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_utc(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_mjd_utc` instead")]
    /// Initialize an Epoch from given MJD in UTC time scale
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_mjd_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_utc(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_jde_tai")]
    /// Initialize an Epoch from given JDE in TAI time scale
    /// :type days: float
    /// :rtype: Epoch
    fn from_jde_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tai(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_jde_tai` instead")]
    /// Initialize an Epoch from given JDE in TAI time scale
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_jde_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tai(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_jde_utc")]
    /// Initialize an Epoch from given JDE in UTC time scale
    /// :type days: float
    /// :rtype: Epoch
    fn from_jde_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_utc(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_jde_utc` instead")]
    /// Initialize an Epoch from given JDE in UTC time scale
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_jde_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_utc(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_tt_seconds")]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_tt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tt_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tt_seconds` instead")]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_tt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tt_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_tt_duration")]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    /// :type duration: Duration
    /// :rtype: Epoch
    fn from_tt_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tt_duration(duration)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tt_duration` instead")]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    /// :type duration: Duration
    /// :rtype: Epoch
    fn init_from_tt_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tt_duration(_cls, duration)
    }

    #[classmethod]
    #[pyo3(name = "from_et_seconds")]
    /// Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)
    /// :type seconds_since_j2000: float
    /// :rtype: Epoch
    fn from_et_seconds(_cls: &Bound<'_, PyType>, seconds_since_j2000: f64) -> Epoch {
        Self::from_et_seconds(seconds_since_j2000)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_et_seconds` instead")]
    /// Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)
    /// :type seconds_since_j2000: float
    /// :rtype: Epoch
    fn init_from_et_seconds(_cls: &Bound<'_, PyType>, seconds_since_j2000: f64) -> Epoch {
        Self::from_et_seconds(_cls, seconds_since_j2000)
    }

    #[classmethod]
    #[pyo3(name = "from_et_duration")]
    /// Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)
    /// :type duration_since_j2000: Duration
    /// :rtype: Epoch
    fn from_et_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Self {
        Self::from_et_duration(duration_since_j2000)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_et_duration` instead")]
    /// Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)
    /// :type duration_since_j2000: Duration
    /// :rtype: Epoch
    fn init_from_et_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Self {
        Self::from_et_duration(_cls, duration_since_j2000)
    }

    #[classmethod]
    #[pyo3(name = "from_tdb_seconds")]
    /// Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    /// NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    /// In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct.
    /// :type seconds_j2000: float
    /// :rtype: Epoch
    fn from_tdb_seconds(_cls: &Bound<'_, PyType>, seconds_j2000: f64) -> Epoch {
        Self::from_tdb_seconds(seconds_j2000)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tdb_seconds` instead")]
    /// Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    /// NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    /// In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct.
    /// :type seconds_j2000: float
    /// :rtype: Epoch
    fn init_from_tdb_seconds(_cls: &Bound<'_, PyType>, seconds_j2000: f64) -> Epoch {
        Self::from_tdb_seconds(_cls, seconds_j2000)
    }

    #[classmethod]
    #[pyo3(name = "from_tdb_duration")]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI.
    ///  :type duration_since_j2000: Duration
    /// :rtype: Epoch
    fn from_tdb_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Epoch {
        Self::from_tdb_duration(duration_since_j2000)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_tdb_duration` instead")]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI.
    ///  :type duration_since_j2000: Duration
    /// :rtype: Epoch
    fn init_from_tdb_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Epoch {
        Self::from_tdb_duration(_cls, duration_since_j2000)
    }

    #[classmethod]
    #[pyo3(name = "from_jde_et")]
    /// Initialize from the JDE days
    /// :type days: float
    /// :rtype: Epoch
    fn from_jde_et(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_et(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_jde_et` instead")]
    /// Initialize from the JDE days
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_jde_et(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_et(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_jde_tdb")]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days
    /// :type days: float
    /// :rtype: Epoch
    fn from_jde_tdb(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tdb(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_jde_tdb` instead")]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_jde_tdb(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tdb(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_gpst_seconds")]
    /// Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_gpst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gpst_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gpst_seconds` instead")]
    /// Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_gpst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gpst_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_gpst_days")]
    /// Initialize an Epoch from the number of days since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type days: float
    /// :rtype: Epoch
    fn from_gpst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gpst_days(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gpst_days` instead")]
    /// Initialize an Epoch from the number of days since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_gpst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gpst_days(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_gpst_nanoseconds")]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use GPS as a time source.
    /// :type nanoseconds: float
    /// :rtype: Epoch
    fn from_gpst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gpst_nanoseconds(nanoseconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gpst_nanoseconds` instead")]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use GPS as a time source.
    /// :type nanoseconds: float
    /// :rtype: Epoch
    fn init_from_gpst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gpst_nanoseconds(_cls, nanoseconds)
    }

    #[classmethod]
    #[pyo3(name = "from_qzsst_seconds")]
    /// Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_qzsst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_qzsst_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_qzsst_seconds` instead")]
    /// Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_qzsst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_qzsst_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_qzsst_days")]
    /// Initialize an Epoch from the number of days since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type days: float
    /// :rtype: Epoch
    fn from_qzsst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_qzsst_days(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_qzsst_days` instead")]
    /// Initialize an Epoch from the number of days since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_qzsst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_qzsst_days(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_qzsst_nanoseconds")]
    /// Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use QZSS as a time source.
    /// :type nanoseconds: int
    /// :rtype: Epoch
    fn from_qzsst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_qzsst_nanoseconds(nanoseconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_qzsst_nanoseconds` instead")]
    /// Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use QZSS as a time source.
    /// :type nanoseconds: int
    /// :rtype: Epoch
    fn init_from_qzsst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_qzsst_nanoseconds(_cls, nanoseconds)
    }

    #[classmethod]
    #[pyo3(name = "from_gst_seconds")]
    /// Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_gst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gst_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gst_seconds` instead")]
    /// Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_gst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gst_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_gst_days")]
    /// Initialize an Epoch from the number of days since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type days: float
    /// :rtype: Epoch
    fn from_gst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gst_days(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gst_days` instead")]
    /// Initialize an Epoch from the number of days since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_gst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gst_days(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_gst_nanoseconds")]
    /// Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use GST as a time source.
    /// :type nanoseconds: float
    /// :rtype: Epoch
    fn from_gst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gst_nanoseconds(nanoseconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gst_nanoseconds` instead")]
    /// Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use GST as a time source.
    /// :type nanoseconds: float
    /// :rtype: Epoch
    fn init_from_gst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gst_nanoseconds(_cls, nanoseconds)
    }

    #[classmethod]
    #[pyo3(name = "from_bdt_seconds")]
    /// Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_bdt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_bdt_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_bdt_seconds` instead")]
    /// Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_bdt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_bdt_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_bdt_days")]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type days: float
    /// :rtype: Epoch
    fn from_bdt_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_bdt_days(days)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_bdt_days` instead")]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// :type days: float
    /// :rtype: Epoch
    fn init_from_bdt_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_bdt_days(_cls, days)
    }

    #[classmethod]
    #[pyo3(name = "from_bdt_nanoseconds")]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use BDT as a time source.
    /// :type nanoseconds: float
    /// :rtype: Epoch
    fn from_bdt_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_bdt_nanoseconds(nanoseconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_bdt_nanoseconds` instead")]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use BDT as a time source.
    /// :type nanoseconds: float
    /// :rtype: Epoch
    fn init_from_bdt_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_bdt_nanoseconds(_cls, nanoseconds)
    }

    #[classmethod]
    #[pyo3(name = "from_unix_seconds")]
    /// Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01.
    /// :type seconds: float
    /// :rtype: Epoch
    fn from_unix_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_unix_seconds(seconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_unix_seconds` instead")]
    /// Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01.
    /// :type seconds: float
    /// :rtype: Epoch
    fn init_from_unix_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_unix_seconds(_cls, seconds)
    }

    #[classmethod]
    #[pyo3(name = "from_unix_milliseconds")]
    /// Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01.
    /// :type milliseconds: float
    /// :rtype: Epoch
    fn from_unix_milliseconds(_cls: &Bound<'_, PyType>, milliseconds: f64) -> Self {
        Self::from_unix_milliseconds(milliseconds)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_unix_milliseconds` instead")]
    /// Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01.
    /// :type milliseconds: float
    /// :rtype: Epoch
    fn init_from_unix_milliseconds(_cls: &Bound<'_, PyType>, milliseconds: f64) -> Self {
        Self::from_unix_milliseconds(_cls, milliseconds)
    }

    #[classmethod]
    #[pyo3(name = "from_gregorian")]
    /// Initialize from the Gregorian parts
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type hour: int
    /// :type minute: int
    /// :type second: int
    /// :type nanos: int
    /// :type time_scale: TimeScale
    /// :rtype: Epoch
    fn from_gregorian(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        time_scale: TimeScale,
    ) -> Result<Self, HifitimeError> {
        Self::maybe_from_gregorian(year, month, day, hour, minute, second, nanos, time_scale)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gregorian` instead")]
    /// Initialize from the Gregorian parts
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type hour: int
    /// :type minute: int
    /// :type second: int
    /// :type nanos: int
    /// :type time_scale: TimeScale
    /// :rtype: Epoch
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
    ) -> Result<Self, HifitimeError> {
        Self::from_gregorian(_cls, year, month, day, hour, minute, second, nanos, time_scale)
    }

    #[classmethod]
    #[pyo3(name = "from_gregorian_at_noon")]
    /// Initialize from the Gregorian parts, time set to noon
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type time_scale: TimeScale
    /// :rtype: Epoch
    fn from_gregorian_at_noon(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Result<Self, HifitimeError> {
        Self::maybe_from_gregorian(year, month, day, 12, 0, 0, 0, time_scale)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gregorian_at_noon` instead")]
    /// Initialize from the Gregorian parts, time set to noon
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type time_scale: TimeScale
    /// :rtype: Epoch
    fn init_from_gregorian_at_noon(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Result<Self, HifitimeError> {
        Self::from_gregorian_at_noon(_cls, year, month, day, time_scale)
    }

    #[classmethod]
    #[pyo3(name = "from_gregorian_at_midnight")]
    /// Initialize from the Gregorian parts, time set to midnight
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type time_scale: TimeScale
    /// :rtype: Epoch
    fn from_gregorian_at_midnight(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Result<Self, HifitimeError> {
        Self::maybe_from_gregorian(year, month, day, 0, 0, 0, 0, time_scale)
    }
    
    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gregorian_at_midnight` instead")]
    /// Initialize from the Gregorian parts, time set to midnight
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type time_scale: TimeScale
    /// :rtype: Epoch
    fn init_from_gregorian_at_midnight(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        time_scale: TimeScale,
    ) -> Result<Self, HifitimeError> {
        Self::from_gregorian_at_midnight(_cls, year, month, day, time_scale)
    }

    #[classmethod]
    #[pyo3(name = "from_gregorian_utc")]
    /// Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_tai if unsure.
    ///
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type hour: int
    /// :type minute: int
    /// :type second: int
    /// :type nanos: int
    /// :rtype: Epoch
    fn from_gregorian_utc(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, HifitimeError> {
        Self::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanos)
    }

    #[classmethod]
    #[deprecated(since = "4.2.0", note = "Use `from_gregorian_utc` instead")]
    /// Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    /// Use maybe_from_gregorian_tai if unsure.
    ///
    /// :type year: int
    /// :type month: int
    /// :type day: int
    /// :type hour: int
    /// :type minute: int
    /// :type second: int
    /// :type nanos: int
    /// :rtype: Epoch
    fn init_from_gregorian_utc(
        _cls: &Bound<'_, PyType>,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, HifitimeError> {
        Self::from_gregorian_utc(_cls, year, month, day, hour, minute, second, nanos)
    }

    #[classmethod]
    /// Equivalent to `datetime.strptime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    /// :type epoch_str: str
    /// :type format_str: str
    /// :rtype: Epoch
    fn strptime(_cls: &Bound<'_, PyType>, epoch_str: String, format_str: String) -> PyResult<Self> {
        Self::from_format_str(&epoch_str, &format_str).map_err(|e| PyErr::from(e))
    }

    /// Equivalent to `datetime.strftime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    /// :type format_str: str
    /// :rtype: str
    fn strftime(&self, format_str: String) -> PyResult<String> {
        use crate::efmt::Formatter;
        let fmt = Format::from_str(&format_str)?;
        Ok(format!("{}", Formatter::new(*self, fmt)))
    }

    /// Equivalent to `datetime.isoformat`, and truncated to 23 chars, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    /// :rtype: str
    fn isoformat(&self) -> PyResult<String> {
        Ok(self.to_isoformat())
    }

    #[new]
    fn new_py(string_repr: String) -> PyResult<Self> {
        match Self::from_str(&string_repr) {
            Ok(d) => Ok(d),
            Err(e) => Err(PyErr::from(e)),
        }
    }

    /// :rtype: Duration
    #[getter]
    fn get_duration(&self) -> PyResult<Duration> {
        Ok(self.duration)
    }

    /// :rtype: TimeScale
    #[getter]
    fn get_time_scale(&self) -> PyResult<TimeScale> {
        Ok(self.time_scale)
    }

    /// Get the accumulated number of leap seconds up to this Epoch from the provided LeapSecondProvider.
    /// Returns None if the epoch is before 1960, year at which UTC was defined.
    ///
    /// # Why does this function return an `Option` when the other returns a value
    /// This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960.
    ///
    /// :type iers_only: bool
    /// :type provider: LeapSecondsFile
    /// :rtype: float
    #[cfg(feature = "python")]
    pub fn leap_seconds_with_file(
        &self,
        iers_only: bool,
        provider: LeapSecondsFile,
    ) -> Option<f64> {
        self.leap_seconds_with(iers_only, provider)
    }

    fn __getnewargs__(&self) -> Result<(String,), PyErr> {
        Ok((format!("{self:?}"),))
    }

    #[classmethod]
    /// Returns the computer clock in UTC
    ///
    /// :rtype: Epoch
    fn system_now(_cls: &Bound<'_, PyType>) -> Result<Self, HifitimeError> {
        Self::now()
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
    }

    fn __add__(&self, duration: Duration) -> Self {
        *self + duration
    }

    fn __sub__(&self, duration: Duration) -> Self {
        *self - duration
    }

    /// Differences between two epochs
    /// :type other: Duration
    /// :rtype: Duration
    fn timedelta(&self, other: Self) -> Duration {
        *self - other
    }

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

    /// Returns a Python datetime object from this Epoch (truncating the nanoseconds away)
    fn todatetime<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDateTime>, PyErr> {
        let (y, mm, dd, hh, min, s, nanos) =
            Epoch::compute_gregorian(self.duration, TimeScale::UTC);

        let datetime = PyDateTime::new(py, y, mm, dd, hh, min, s, nanos / 1_000, None)?;

        Ok(datetime)
    }

    /// Builds an Epoch in UTC from the provided datetime after timezone correction if any is present.
    #[classmethod]
    fn fromdatetime(
        _cls: &Bound<'_, PyType>,
        dt: &Bound<'_, PyAny>,
    ) -> Result<Self, HifitimeError> {
        let dt = dt
            .downcast::<PyDateTime>()
            .map_err(|e| HifitimeError::PythonError {
                reason: e.to_string(),
            })?;

        // If the user tries to convert a timezone aware datetime into a naive one,
        // we return a hard error. We could silently remove tzinfo, or assume local timezone
        // and do a conversion, but better leave this decision to the user of the library.
        let has_tzinfo = dt.get_tzinfo().is_some();
        if has_tzinfo {
            return Err(HifitimeError::PythonError {
                reason: "expected a datetime without tzinfo, call my_datetime.replace(tzinfo=None)"
                    .to_string(),
            });
        }

        Epoch::maybe_from_gregorian_utc(
            dt.get_year(),
            dt.get_month().into(),
            dt.get_day().into(),
            dt.get_hour().into(),
            dt.get_minute().into(),
            dt.get_second().into(),
            dt.get_microsecond() * 1_000,
        )
    }
}
