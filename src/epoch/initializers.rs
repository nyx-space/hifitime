/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2017-onwards Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::str::FromStr;

use snafu::ResultExt;

use crate::{
    efmt::Format, errors::ParseSnafu, Duration, Epoch, HifitimeError, TimeScale, Unit, Weekday,
    ET_OFFSET_US, MJD_J1900, MJD_OFFSET, NANOSECONDS_PER_DAY, UNIX_REF_EPOCH,
};

// Defines the methods that should be classmethods in Python, but must be redefined as per https://github.com/PyO3/pyo3/issues/1003#issuecomment-844433346
impl Epoch {
    #[must_use]
    /// Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch.
    pub const fn from_tai_duration(duration: Duration) -> Self {
        Self {
            duration,
            time_scale: TimeScale::TAI,
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
        Self::from_mjd_in_time_scale(days, TimeScale::TAI)
    }

    pub fn from_mjd_in_time_scale(days: f64, time_scale: TimeScale) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self {
            duration: (days - MJD_J1900) * Unit::Day,
            time_scale,
        }
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
        Self::from_jde_in_time_scale(days, TimeScale::TAI)
    }

    fn from_jde_in_time_scale(days: f64, time_scale: TimeScale) -> Self {
        assert!(
            days.is_finite(),
            "Attempted to initialize Epoch with non finite number"
        );
        Self {
            duration: (days - MJD_J1900 - MJD_OFFSET) * Unit::Day,
            time_scale,
        }
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
        Self::from_duration(Duration::from_parts(0, nanoseconds), TimeScale::GPST)
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
        Self::from_duration(Duration::from_parts(0, nanoseconds), TimeScale::QZSST)
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
        Self::from_duration(Duration::from_parts(0, nanoseconds), TimeScale::GST)
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
        Self::from_duration(Duration::from_parts(0, nanoseconds), TimeScale::BDT)
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
    pub fn from_str_with_format(s_in: &str, format: Format) -> Result<Self, HifitimeError> {
        format.parse(s_in)
    }

    /// Initializes an Epoch from the Format as a string.
    pub fn from_format_str(s_in: &str, format_str: &str) -> Result<Self, HifitimeError> {
        Format::from_str(format_str)
            .with_context(|_| ParseSnafu {
                details: "when using format string",
            })?
            .parse(s_in)
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
