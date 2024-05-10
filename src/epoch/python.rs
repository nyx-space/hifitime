/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the implementations that are only built with the pyhon feature.

use crate::{prelude::Format, Duration, Epoch, EpochError, TimeScale};

use core::str::FromStr;

use crate::epoch::leap_seconds_file::LeapSecondsFile;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyType;

#[pymethods]
impl Epoch {
    #[classmethod]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    const fn init_from_tai_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tai_duration(duration)
    }

    #[classmethod]
    /// Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch.
    fn init_from_tai_parts(_cls: &Bound<'_, PyType>, centuries: i16, nanoseconds: u64) -> Self {
        Self::from_tai_parts(centuries, nanoseconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    fn init_from_tai_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tai_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    fn init_from_tai_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_tai_days(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight
    fn init_from_utc_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_utc_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight
    fn init_from_utc_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_utc_days(days)
    }

    #[classmethod]
    /// Initialize an Epoch from given MJD in TAI time scale
    fn init_from_mjd_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_tai(days)
    }

    #[classmethod]
    /// Initialize an Epoch from given MJD in UTC time scale
    fn init_from_mjd_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_mjd_utc(days)
    }

    #[classmethod]
    /// Initialize an Epoch from given JDE in TAI time scale
    fn init_from_jde_tai(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tai(days)
    }

    #[classmethod]
    /// Initialize an Epoch from given JDE in UTC time scale
    fn init_from_jde_utc(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_utc(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    fn init_from_tt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_tt_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    fn init_from_tt_duration(_cls: &Bound<'_, PyType>, duration: Duration) -> Self {
        Self::from_tt_duration(duration)
    }

    #[classmethod]
    /// Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)
    fn init_from_et_seconds(_cls: &Bound<'_, PyType>, seconds_since_j2000: f64) -> Epoch {
        Self::from_et_seconds(seconds_since_j2000)
    }

    #[classmethod]
    /// Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)
    fn init_from_et_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Self {
        Self::from_et_duration(duration_since_j2000)
    }

    #[classmethod]
    /// Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    /// NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    /// In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct.
    fn init_from_tdb_seconds(_cls: &Bound<'_, PyType>, seconds_j2000: f64) -> Epoch {
        Self::from_tdb_seconds(seconds_j2000)
    }

    #[classmethod]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI.
    fn init_from_tdb_duration(_cls: &Bound<'_, PyType>, duration_since_j2000: Duration) -> Epoch {
        Self::from_tdb_duration(duration_since_j2000)
    }

    #[classmethod]
    /// Initialize from the JDE days
    fn init_from_jde_et(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_et(days)
    }

    #[classmethod]
    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days
    fn init_from_jde_tdb(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_jde_tdb(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_gpst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gpst_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of days since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_gpst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gpst_days(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use GPS as a time source.
    fn init_from_gpst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gpst_nanoseconds(nanoseconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_qzsst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_qzsst_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of days since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    fn init_from_qzsst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_qzsst_days(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    /// defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    /// This may be useful for time keeping devices that use QZSS as a time source.
    fn init_from_qzsst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_qzsst_nanoseconds(nanoseconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_gst_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_gst_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of days since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_gst_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_gst_days(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    /// starting on August 21st 1999 Midnight UT,
    /// (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use GST as a time source.
    fn init_from_gst_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_gst_nanoseconds(nanoseconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_bdt_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_bdt_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    fn init_from_bdt_days(_cls: &Bound<'_, PyType>, days: f64) -> Self {
        Self::from_bdt_days(days)
    }

    #[classmethod]
    /// Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    /// defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    /// This may be useful for time keeping devices that use BDT as a time source.
    fn init_from_bdt_nanoseconds(_cls: &Bound<'_, PyType>, nanoseconds: u64) -> Self {
        Self::from_bdt_nanoseconds(nanoseconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01.
    fn init_from_unix_seconds(_cls: &Bound<'_, PyType>, seconds: f64) -> Self {
        Self::from_unix_seconds(seconds)
    }

    #[classmethod]
    /// Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01.
    fn init_from_unix_milliseconds(_cls: &Bound<'_, PyType>, milliseconds: f64) -> Self {
        Self::from_unix_milliseconds(milliseconds)
    }

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
    ) -> Result<Self, EpochError> {
        Self::maybe_from_gregorian_tai(year, month, day, hour, minute, second, nanos)
    }

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
    ) -> Result<Self, EpochError> {
        Self::maybe_from_gregorian(year, month, day, hour, minute, second, nanos, time_scale)
    }

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
    ) -> Result<Self, EpochError> {
        Self::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanos)
    }

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

    #[classmethod]
    /// Equivalent to `datetime.strptime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    fn strptime(_cls: &Bound<'_, PyType>, epoch_str: String, format_str: String) -> PyResult<Self> {
        Self::from_format_str(&epoch_str, &format_str).map_err(|e| PyErr::from(e))
    }

    /// Equivalent to `datetime.strftime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
    fn strftime(&self, format_str: String) -> PyResult<String> {
        use crate::efmt::Formatter;
        let fmt = Format::from_str(&format_str)?;
        Ok(format!("{}", Formatter::new(*self, fmt)))
    }

    /// Equivalent to `datetime.isoformat`, and truncated to 23 chars, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options
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

    fn __getnewargs__(&self) -> Result<(String,), PyErr> {
        Ok((format!("{self:?}"),))
    }

    #[classmethod]
    fn system_now(_cls: &Bound<'_, PyType>) -> Result<Self, EpochError> {
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
}
