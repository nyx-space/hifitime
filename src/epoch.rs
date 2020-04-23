extern crate lazy_static;
extern crate regex;

use self::lazy_static::lazy_static;
use self::regex::Regex;
use crate::{
    Errors, TimeSystem, DAYS_PER_CENTURY, ET_EPOCH_S, J1900_OFFSET, MJD_OFFSET, SECONDS_PER_DAY,
};
use std::ops::{Add, Sub};
use std::str::FromStr;

/// From https://www.ietf.org/timezones/data/leap-seconds.list .
const LEAP_SECONDS: [f64; 28] = [
    2_272_060_800.0, //	10	# 1 Jan 1972
    2_287_785_600.0, //	11	# 1 Jul 1972
    2_303_683_200.0, //	12	# 1 Jan 1973
    2_335_219_200.0, //	13	# 1 Jan 1974
    2_366_755_200.0, //	14	# 1 Jan 1975
    2_398_291_200.0, //	15	# 1 Jan 1976
    2_429_913_600.0, //	16	# 1 Jan 1977
    2_461_449_600.0, //	17	# 1 Jan 1978
    2_492_985_600.0, //	18	# 1 Jan 1979
    2_524_521_600.0, //	19	# 1 Jan 1980
    2_571_782_400.0, //	20	# 1 Jul 1981
    2_603_318_400.0, //	21	# 1 Jul 1982
    2_634_854_400.0, //	22	# 1 Jul 1983
    2_698_012_800.0, //	23	# 1 Jul 1985
    2_776_982_400.0, //	24	# 1 Jan 1988
    2_840_140_800.0, //	25	# 1 Jan 1990
    2_871_676_800.0, //	26	# 1 Jan 1991
    2_918_937_600.0, //	27	# 1 Jul 1992
    2_950_473_600.0, //	28	# 1 Jul 1993
    2_982_009_600.0, //	29	# 1 Jul 1994
    3_029_443_200.0, //	30	# 1 Jan 1996
    3_076_704_000.0, //	31	# 1 Jul 1997
    3_124_137_600.0, //	32	# 1 Jan 1999
    3_345_062_400.0, //	33	# 1 Jan 2006
    3_439_756_800.0, //	34	# 1 Jan 2009
    3_550_089_600.0, //	35	# 1 Jul 2012
    3_644_697_600.0, //	36	# 1 Jul 2015
    3_692_217_600.0, //	37	# 1 Jan 2017
];

const JANUARY_YEARS: [i32; 17] = [
    1972, 1973, 1974, 1975, 1976, 1977, 1978, 1979, 1980, 1988, 1990, 1991, 1996, 1999, 2006, 2009,
    2017,
];

const JULY_YEARS: [i32; 11] = [
    1972, 1981, 1982, 1983, 1985, 1992, 1993, 1994, 1997, 2012, 2015,
];

const USUAL_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const USUAL_DAYS_PER_YEAR: f64 = 365.0;

/// Defines an Epoch in TAI (temps atomique international) in seconds past 1900 January 01 at midnight (like the Network Time Protocol).
///
/// Refer to the appropriate functions for initializing this Epoch from different time systems or representations.
// pub struct Epoch {
//     pub secs_past_1900: f64,
// }
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Epoch(f64);

impl Sub for Epoch {
    type Output = f64;

    fn sub(self, other: Self) -> f64 {
        self.0 - other.0
    }
}

impl Add<f64> for Epoch {
    type Output = Self;

    fn add(self, seconds: f64) -> Self {
        Self {
            0: self.0 + seconds,
        }
    }
}

impl Sub<f64> for Epoch {
    type Output = Self;

    fn sub(self, seconds: f64) -> Self {
        Self {
            0: self.0 - seconds,
        }
    }
}

impl Epoch {
    /// Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight
    pub fn from_tai_seconds(seconds: f64) -> Self {
        Self { 0: seconds }
    }

    /// Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight
    pub fn from_tai_days(days: f64) -> Self {
        Self {
            0: days * SECONDS_PER_DAY,
        }
    }

    pub fn from_mjd_tai(days: f64) -> Self {
        Self {
            0: (days - J1900_OFFSET) * SECONDS_PER_DAY,
        }
    }

    pub fn from_jde_tai(days: f64) -> Self {
        Self {
            0: (days - J1900_OFFSET - MJD_OFFSET) * SECONDS_PER_DAY,
        }
    }

    /// Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)
    pub fn from_tt_seconds(seconds: f64) -> Self {
        Self {
            0: seconds - 32.184,
        }
    }

    pub fn from_et_seconds(seconds: f64) -> Epoch {
        Self {
            0: seconds - 32.184_935 + ET_EPOCH_S,
        }
    }

    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI
    pub fn from_tdb_seconds(seconds: f64) -> Epoch {
        use std::f64::consts::PI;
        let tt_seconds = seconds - 32.184;
        let tt_centuries_j2k = (tt_seconds - ET_EPOCH_S) / (SECONDS_PER_DAY * DAYS_PER_CENTURY);
        let g_rad = 2.0 * PI * (357.528 + 35_999.050 * tt_centuries_j2k) / 360.0;

        Self {
            0: tt_seconds + ET_EPOCH_S - 0.001_658 * (g_rad + 0.0167 * g_rad.sin()).sin(),
        }
    }

    pub fn from_jde_et(days: f64) -> Self {
        Self::from_jde_tdb(days)
    }

    /// Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days
    pub fn from_jde_tdb(days: f64) -> Self {
        let mut rtn = Self::from_jde_tai(days);
        rtn.0 -= 32.184_935;
        rtn
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

        let mut seconds_wrt_1900: f64 =
            f64::from((year - 1900).abs()) * SECONDS_PER_DAY * USUAL_DAYS_PER_YEAR;

        // Now add the seconds for all the years prior to the current year
        for year in 1900..year {
            if is_leap_year(year) {
                seconds_wrt_1900 += SECONDS_PER_DAY;
            }
        }
        // Add the seconds for the months prior to the current month
        for month in 0..month - 1 {
            seconds_wrt_1900 += SECONDS_PER_DAY * f64::from(USUAL_DAYS_PER_MONTH[(month) as usize]);
        }
        if is_leap_year(year) && month > 2 {
            // NOTE: If on 29th of February, then the day is not finished yet, and therefore
            // the extra seconds are added below as per a normal day.
            seconds_wrt_1900 += SECONDS_PER_DAY;
        }
        seconds_wrt_1900 += f64::from(day - 1) * SECONDS_PER_DAY
            + f64::from(hour) * 3600.0
            + f64::from(minute) * 60.0
            + f64::from(second);
        if second == 60 {
            // Herein lies the whole ambiguity of leap seconds. Two different UTC dates exist at the
            // same number of second afters J1900.0.
            seconds_wrt_1900 -= 1.0;
        }

        Ok(match ts {
            TimeSystem::TAI => Epoch::from_tai_seconds(seconds_wrt_1900),
            TimeSystem::TT => Epoch::from_tt_seconds(seconds_wrt_1900),
            TimeSystem::ET => Epoch::from_et_seconds(seconds_wrt_1900),
            TimeSystem::TDB => Epoch::from_tdb_seconds(seconds_wrt_1900),
            TimeSystem::UTC => panic!("use maybe_from_gregorian_utc for UTC time system"),
        })
    }

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

    pub fn from_gregorian_tai_at_midnight(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_tai(year, month, day, 0, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

    pub fn from_gregorian_tai_at_noon(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_tai(year, month, day, 12, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

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
        let mut cnt = 0;
        for tai_ts in LEAP_SECONDS.iter() {
            if &if_tai.0 >= tai_ts {
                if cnt == 0 {
                    cnt = 10;
                } else {
                    cnt += 1;
                }
            } else {
                break; // No more leap seconds to process
            }
        }
        // We have the time in TAI. But we were given UTC.
        // Hence, we need to _add_ the leap seconds to get the actual TAI time.
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        if_tai.0 += f64::from(cnt);
        Ok(if_tai)
    }

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

    pub fn from_gregorian_utc_at_midnight(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_utc(year, month, day, 0, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

    pub fn from_gregorian_utc_at_noon(year: i32, month: u8, day: u8) -> Self {
        Self::maybe_from_gregorian_utc(year, month, day, 12, 0, 0, 0)
            .expect("invalid Gregorian date")
    }

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

    pub fn as_tai_seconds(self) -> f64 {
        self.0
    }

    pub fn as_tai_days(self) -> f64 {
        self.0 / SECONDS_PER_DAY
    }

    /// Returns the number of UTC seconds since the TAI epoch
    pub fn as_utc_seconds(self) -> f64 {
        let mut cnt = 0;
        for tai_ts in LEAP_SECONDS.iter() {
            if &self.0 >= tai_ts {
                if cnt == 0 {
                    cnt = 10;
                } else {
                    cnt += 1;
                }
            } else {
                break; // No more leap seconds to process
            }
        }
        // TAI = UTC + leap_seconds <=> UTC = TAI - leap_seconds
        self.0 - f64::from(cnt)
    }

    pub fn as_utc_days(self) -> f64 {
        self.as_utc_seconds() / SECONDS_PER_DAY
    }

    /// `as_mjd_days` creates an Epoch from the provided Modified Julian Date in days as explained
    /// [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
    pub fn as_mjd_tai_days(self) -> f64 {
        self.as_tai_days() + J1900_OFFSET
    }

    /// Returns the Modified Julian Date in days UTC.
    pub fn as_mjd_utc_days(self) -> f64 {
        self.as_utc_days() + J1900_OFFSET
    }

    /// Returns the Modified Julian Date in seconds TAI.
    pub fn as_mjd_tai_seconds(self) -> f64 {
        self.as_mjd_tai_days() * SECONDS_PER_DAY
    }

    /// Returns the Modified Julian Date in seconds UTC.
    pub fn as_mjd_utc_seconds(self) -> f64 {
        self.as_mjd_utc_days() * SECONDS_PER_DAY
    }

    /// Returns the Julian days from epoch 01 Jan -4713, 12:00 (noon)
    /// as explained in "Fundamentals of astrodynamics and applications", Vallado et al.
    /// 4th edition, page 182, and on [Wikipedia](https://en.wikipedia.org/wiki/Julian_day).
    pub fn as_jde_tai_days(self) -> f64 {
        self.as_mjd_tai_days() + MJD_OFFSET
    }

    /// Returns the Julian seconds in TAI.
    pub fn as_jde_tai_seconds(self) -> f64 {
        self.as_jde_tai_days() * SECONDS_PER_DAY
    }

    /// Returns the Julian days in UTC.
    pub fn as_jde_utc_days(self) -> f64 {
        self.as_mjd_utc_days() + MJD_OFFSET
    }

    /// Returns the Julian seconds in UTC.
    pub fn as_jde_utc_seconds(self) -> f64 {
        self.as_jde_utc_days() * SECONDS_PER_DAY
    }

    /// Returns seconds past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_tt_seconds(self) -> f64 {
        self.as_tai_seconds() + 32.184
    }

    /// Returns days past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_tt_days(self) -> f64 {
        self.as_tai_days() + 32.184 / SECONDS_PER_DAY
    }

    /// Returns the centuries pased J2000 TT
    pub fn as_tt_centuries_j2k(self) -> f64 {
        (self.as_tt_seconds() - ET_EPOCH_S) / (SECONDS_PER_DAY * DAYS_PER_CENTURY)
    }

    /// Returns days past Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_jde_tt_days(self) -> f64 {
        self.as_jde_tai_days() + 32.184 / SECONDS_PER_DAY
    }

    /// Returns days past Modified Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    pub fn as_mjd_tt_days(self) -> f64 {
        self.as_mjd_tai_days() + 32.184 / SECONDS_PER_DAY
    }

    /// Returns seconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29).
    pub fn as_gpst_seconds(self) -> f64 {
        self.as_tai_seconds() - 19.0
    }

    /// Returns days past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29).
    pub fn as_gpst_days(self) -> f64 {
        self.as_gpst_seconds() / SECONDS_PER_DAY
    }

    /// Returns the Ephemeris Time seconds past epoch
    pub fn as_et_seconds(self) -> f64 {
        self.as_tt_seconds() - ET_EPOCH_S + 0.000_935
    }

    /// Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB)
    pub fn as_tdb_seconds(self) -> f64 {
        use std::f64::consts::PI;
        let g_rad = 2.0 * PI * (357.528 + 35_999.050 * self.as_tt_centuries_j2k()) / 360.0;
        self.as_tt_seconds() - ET_EPOCH_S + 0.001_658 * (g_rad + 0.0167 * g_rad.sin()).sin()
    }

    /// Returns the Ephemeris Time JDE past epoch
    pub fn as_jde_et_days(self) -> f64 {
        // self.as_jde_tdb_days()
        self.as_jde_tt_days() + 0.000_935 / SECONDS_PER_DAY
    }

    /// Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB)
    pub fn as_jde_tdb_days(self) -> f64 {
        use std::f64::consts::PI;
        let g_rad = 2.0 * PI * (357.528 + 35_999.050 * self.as_tt_centuries_j2k()) / 360.0;
        self.as_jde_tt_days() + (0.001_658 * (g_rad + 0.0167 * g_rad.sin()).sin()) / SECONDS_PER_DAY
    }

    /// Increment this epoch by the number of days provided.
    pub fn mut_add_days(&mut self, days: f64) {
        self.0 += days * SECONDS_PER_DAY
    }

    /// Increment this epoch by the number of days provided.
    pub fn mut_add_secs(&mut self, seconds: f64) {
        self.0 += seconds
    }

    /// Decrement this epoch by the number of days provided.
    pub fn mut_sub_days(&mut self, days: f64) {
        self.0 -= days * SECONDS_PER_DAY
    }

    /// Decrement this epoch by the number of days provided.
    pub fn mut_sub_secs(&mut self, seconds: f64) {
        self.0 -= seconds
    }

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
    /// ```
    pub fn from_gregorian_str(s: &str) -> Result<Self, Errors> {
        lazy_static! {
            static ref RE_GREG: Regex = Regex::new(
                r"^(\d{4})-(\d{2})-(\d{2})(?:T|\W)(\d{2}):(\d{2}):(\d{2})\.?(\d+)?\W?(\w{2,3})?$"
            )
            .unwrap();
        }
        match RE_GREG.captures(s) {
            Some(cap) => {
                let nanos = match cap.get(7) {
                    Some(val) => val.as_str().parse::<u32>().unwrap(),
                    None => 0,
                };

                match cap.get(8) {
                    Some(ts_str) => {
                        let ts = TimeSystem::map(ts_str.as_str().to_owned());
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
                        // Asumme UTC
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
            None => Err(Errors::ParseError(
                "Input not in ISO8601 format without offset (e.g. 2018-01-27T00:41:55)".to_owned(),
            )),
        }
    }

    /// Converts the Epoch to the Gregorian UTC equivalent as (year, month, day, hour, minute, second).
    /// WARNING: Nanoseconds are lost in this conversion!
    ///
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    /// let dt_str = "2017-01-14T00:31:55";
    /// let dt = Epoch::from_gregorian_str(dt_str).unwrap();
    /// let (y, m, d, h, min, s) = dt.as_gregorian_utc();
    /// assert_eq!(y, 2017);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 14);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 31);
    /// assert_eq!(s, 55);
    /// assert_eq!(dt_str, dt.as_gregorian_utc_str().to_owned());
    /// ```
    pub fn as_gregorian_utc(self) -> (i32, u8, u8, u8, u8, u8) {
        Self::compute_gregorian(self.as_utc_seconds())
    }

    /// Converts the Epoch to UTC Gregorian in the ISO8601 format.
    pub fn as_gregorian_utc_str(self) -> String {
        let (y, mm, dd, hh, min, s) = Self::compute_gregorian(self.as_utc_seconds());
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, mm, dd, hh, min, s)
    }

    /// Converts the Epoch to the Gregorian TAI equivalent as (year, month, day, hour, minute, second).
    /// WARNING: Nanoseconds are lost in this conversion!
    ///
    /// # Example
    /// ```
    /// use hifitime::Epoch;
    /// let dt = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    /// let (y, m, d, h, min, s) = dt.as_gregorian_tai();
    /// assert_eq!(y, 1972);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 1);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 0);
    /// assert_eq!(s, 0);
    /// ```
    pub fn as_gregorian_tai(self) -> (i32, u8, u8, u8, u8, u8) {
        Self::compute_gregorian(self.as_tai_seconds())
    }

    #[deprecated(
        since = "1.0.11",
        note = "Use as_gregorian_tai_str() instead; this function was a typo"
    )]
    pub fn as_gregorian_utc_tai(self) -> String {
        self.as_gregorian_tai_str()
    }

    /// Converts the Epoch to TAI Gregorian in the ISO8601 format with " TAI" appended to the string
    pub fn as_gregorian_tai_str(self) -> String {
        let (y, mm, dd, hh, min, s) = Self::compute_gregorian(self.as_utc_seconds());
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} TAI",
            y, mm, dd, hh, min, s
        )
    }

    fn compute_gregorian(absolute_seconds: f64) -> (i32, u8, u8, u8, u8, u8) {
        let (mut year, mut year_fraction) = quorem(absolute_seconds, 365.0 * SECONDS_PER_DAY);
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
        let (_, month_fraction) = quorem(
            year_fraction - seconds_til_this_month,
            f64::from(days_this_month) * SECONDS_PER_DAY,
        );
        // Get the day by the exact number of seconds in a day
        let (mut day, day_fraction) = quorem(month_fraction, SECONDS_PER_DAY);
        if day < 0 {
            // Overflow backwards (this happens for end of year calculations)
            month -= 1;
            if month == 0 {
                month = 12;
                year -= 1;
            }
            day = i32::from(USUAL_DAYS_PER_MONTH[(month - 1) as usize]);
        }
        day += 1; // Otherwise the day count starts at 0
                  // Get the hours by the exact number of seconds in an hour
        let (hours, hours_fraction) = quorem(day_fraction, 60.0 * 60.0);
        // Get the minutes and seconds by the exact number of seconds in a minute
        let (mins, secs) = quorem(hours_fraction, 60.0);
        (
            year,
            month as u8,
            day as u8,
            hours as u8,
            mins as u8,
            secs as u8,
        )
    }
}

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
        lazy_static! {
            static ref RE_GENERIC: Regex =
                Regex::new(r"^(\w{2,3})\W?(\d+\.?\d+)\W?(\w{2,3})?$").unwrap();
        }
        // Try to match Gregorian date
        match Self::from_gregorian_str(s) {
            Ok(e) => Ok(e),
            Err(_) => match RE_GENERIC.captures(s) {
                Some(cap) => {
                    let format = cap[1].to_owned().parse::<String>().unwrap();
                    let value = cap[2].to_owned().parse::<f64>().unwrap();
                    let ts = TimeSystem::map(cap[3].to_owned().parse::<String>().unwrap());

                    match format.as_str() {
                        "JD" => match ts {
                            TimeSystem::ET => Ok(Epoch::from_jde_et(value)),
                            TimeSystem::TAI => Ok(Epoch::from_jde_tai(value)),
                            TimeSystem::TDB => Ok(Epoch::from_jde_tdb(value)),
                            _ => Err(Errors::ParseError(format!(
                                "Cannot initialize JD in {:?}",
                                ts
                            ))),
                        },
                        "MJD" => match ts {
                            TimeSystem::TAI => Ok(Epoch::from_mjd_tai(value)),
                            _ => Err(Errors::ParseError(format!(
                                "Cannot initialize MJD in {:?}",
                                ts
                            ))),
                        },
                        "SEC" => match ts {
                            TimeSystem::TAI => Ok(Epoch::from_tai_seconds(value)),
                            TimeSystem::ET => Ok(Epoch::from_et_seconds(value)),
                            TimeSystem::TDB => Ok(Epoch::from_tdb_seconds(value)),
                            TimeSystem::TT => Ok(Epoch::from_tt_seconds(value)),
                            _ => Err(Errors::ParseError(format!(
                                "Cannot initialize SEC in {:?}",
                                ts
                            ))),
                        },
                        _ => Err(Errors::ParseError(format!("Unknown format  {}", format))),
                    }
                }
                None => Err(Errors::ParseError("Input not understood".to_owned())),
            },
        }
    }
}

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

#[test]
fn utc_epochs() {
    use std::f64::EPSILON;
    assert!(Epoch::from_mjd_tai(J1900_OFFSET).as_tai_seconds() < EPSILON);
    assert!((Epoch::from_mjd_tai(J1900_OFFSET).as_mjd_tai_days() - J1900_OFFSET).abs() < EPSILON);

    // Tests are chronological dates.
    // All of the following examples are cross validated against NASA HEASARC,
    // refered to as "X-Val" for "cross validation."

    // X-Val: 03 January 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=1&d2=03&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_199_333_568.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 1, 3, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 28 February 1938 00:00:00 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=02&d2=28&y2=1938&h1=0&i1=0&s1=0&h2=0&i2=0&s2=0
    let this_epoch = Epoch::from_tai_seconds(1_204_156_800.0);
    let epoch_utc =
        Epoch::maybe_from_gregorian_utc(1938, 2, 28, 00, 00, 00, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // 28 February 1938 23:59:59 (no X-Val: took the next test and subtracted one second)
    let this_epoch = Epoch::from_tai_seconds(1_204_243_199.0);
    let epoch_utc =
        Epoch::maybe_from_gregorian_utc(1938, 2, 28, 23, 59, 59, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 01 March 1938 00:00:00 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=3&d2=01&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_204_243_200.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 3, 1, 00, 00, 00, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 31 March 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=03&d2=31&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_206_850_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 3, 31, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 24 June 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=6&d2=24&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_214_194_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 6, 24, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 31 August 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=8&d2=31&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_220_069_568.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 8, 31, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 31 December 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=12&d2=31&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_230_610_368.0);
    let epoch_utc =
        Epoch::maybe_from_gregorian_utc(1938, 12, 31, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 January 1939 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=01&d2=1&y2=1939&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_230_696_768.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1939, 1, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 March 1939 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=3&d2=1&y2=1939&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_235_794_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1939, 3, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 01 March 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=3&d2=1&y2=1940&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_267_416_768.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 3, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 February 1939 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=1&y2=1939&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_233_375_168.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1939, 2, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 February 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=01&y2=1940&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_264_911_168.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 2, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 28 February 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=28&y2=1940&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_267_243_968.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 2, 28, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 29 February 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=29&y2=1940&h1=0&i1=0&s1=0&h2=04&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_267_330_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 2, 29, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // Test the specific leap second times
    let epoch_from_tai_secs = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    assert!((epoch_from_tai_secs.as_tai_seconds() - 2_272_060_800.0) < EPSILON);
    let epoch_from_tai_greg = Epoch::from_tai_seconds(2_272_060_800.0);
    assert_eq!(epoch_from_tai_greg, epoch_from_tai_secs, "Incorrect epoch");

    // Check that second leap second happens
    let epoch_from_utc_greg = Epoch::from_gregorian_utc_hms(1972, 6, 30, 23, 59, 59);
    let epoch_from_utc_greg1 = Epoch::from_gregorian_utc_hms(1972, 7, 1, 0, 0, 0);
    assert!(
        (epoch_from_utc_greg1.as_tai_seconds() - epoch_from_utc_greg.as_tai_seconds() - 2.0).abs()
            < EPSILON
    );

    // Just prior to the 2017 leap second, there should be an offset of 36 seconds between UTC and TAI
    let this_epoch = Epoch::from_tai_seconds(3_692_217_599.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 23);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    assert!((this_epoch.as_tai_seconds() - epoch_utc.as_utc_seconds()) - 36.0 < EPSILON);

    // Just after to the 2017 leap second, there should be an offset of 37 seconds between UTC and TAI
    let this_epoch = Epoch::from_tai_seconds(3_692_217_600.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 24);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    assert!((this_epoch.as_tai_seconds() - epoch_utc.as_utc_seconds()) - 37.0 < EPSILON);

    let mut this_epoch = Epoch::from_tai_seconds(3_692_217_600.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 24);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    this_epoch.mut_add_secs(3600.0);
    assert_eq!(
        this_epoch,
        Epoch::from_gregorian_utc_hms(2017, 1, 1, 0, 59, 23),
        "Incorrect epoch when adding an hour across leap second"
    );
    this_epoch.mut_sub_secs(3600.0);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch after sub");

    let this_epoch = Epoch::from_gregorian_tai_at_midnight(2020, 1, 1);
    assert!((this_epoch.as_jde_tai_days() - 2_458_849.5).abs() < std::f64::EPSILON)
}

#[test]
fn utc_tai() {
    // General note: TAI "ahead" of UTC means that there are _less_ TAI seconds since epoch for a given date
    // than there are seconds for that UTC epoch: the same TAI time happens _before_ that UTC time.
    use std::f64::EPSILON;
    // flp = first leap second
    let flp_from_secs_tai = Epoch::from_tai_seconds(2_272_060_800.0);
    let flp_from_greg_tai = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    assert_eq!(flp_from_secs_tai, flp_from_greg_tai);
    // Right after the discontinuity, UTC time should be ten seconds behind TAI, i.e. TAI is ten second ahead of UTC
    // In other words, the following date times are equal:
    assert_eq!(
        Epoch::from_gregorian_tai_hms(1972, 1, 1, 0, 0, 10),
        Epoch::from_gregorian_utc_at_midnight(1972, 1, 1),
        "UTC discontinuity failed"
    );
    // Noon UTC after the first leap second is in fact ten seconds _after_ noon TAI.
    // Hence, there are as many TAI seconds since Epoch between UTC Noon and TAI Noon + 10s.
    assert!(
        Epoch::from_gregorian_utc_at_noon(1972, 1, 1)
            > Epoch::from_gregorian_tai_at_noon(1972, 1, 1),
        "TAI is not ahead of UTC (via PartialEq) at noon after first leap second"
    );
    assert!(
        flp_from_secs_tai.as_tai_seconds() > flp_from_secs_tai.as_utc_seconds(),
        "TAI is not ahead of UTC (via function call)"
    );
    assert!(
        (flp_from_secs_tai.as_tai_seconds() - flp_from_secs_tai.as_utc_seconds() - 10.0) < EPSILON,
        "TAI is not ahead of UTC"
    );

    // Check that all of the TAI/UTC time differences are of 37.0 as of today.
    let epoch_utc = Epoch::from_gregorian_utc_hms(2019, 8, 1, 20, 10, 23);
    let epoch_tai = Epoch::from_gregorian_tai_hms(2019, 8, 1, 20, 10, 23);
    assert!(epoch_tai < epoch_utc, "TAI is not ahead of UTC");
    assert!(
        (epoch_utc - epoch_tai - 37.0) < EPSILON,
        "TAI is not ahead of UTC"
    );
    assert!(
        (epoch_utc.as_tai_seconds() - epoch_tai.as_tai_seconds() - 37.0).abs() < EPSILON,
        "TAI is not ahead of UTC"
    );
    assert!(
        (epoch_utc.as_utc_seconds() - epoch_tai.as_utc_seconds() - 37.0).abs() < EPSILON,
        "TAI is not ahead of UTC"
    );
}

#[test]
fn julian_epoch() {
    use std::f64::EPSILON;
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let nist_j1900 = Epoch::from_tai_days(0.0);
    assert!((nist_j1900.as_mjd_tai_days() - 15_020.0).abs() < EPSILON);
    assert!((nist_j1900.as_jde_tai_days() - 2_415_020.5).abs() < EPSILON);
    let mjd = Epoch::from_gregorian_utc_at_midnight(1900, 1, 1);
    assert!((mjd.as_mjd_tai_days() - 15_020.0).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+12%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j1900 = Epoch::from_tai_days(0.5);
    assert!((j1900.as_mjd_tai_days() - 15_020.5).abs() < EPSILON);
    assert!((j1900.as_jde_tai_days() - 2_415_021.0).abs() < EPSILON);
    let mjd = Epoch::from_gregorian_utc_at_noon(1900, 1, 1);
    assert!((mjd.as_mjd_tai_days() - 15_020.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-08+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let mjd = Epoch::from_gregorian_utc_at_midnight(1900, 1, 8);
    assert!((mjd.as_mjd_tai_days() - 15_027.0).abs() < EPSILON);
    assert!((mjd.as_jde_tai_days() - 2_415_027.5).abs() < EPSILON);
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1980-01-06+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let gps_std_epoch = Epoch::from_gregorian_tai_at_midnight(1980, 1, 6);
    assert!((gps_std_epoch.as_mjd_tai_days() - 44_244.0).abs() < EPSILON);
    assert!((gps_std_epoch.as_jde_tai_days() - 2_444_244.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2000-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j2000 = Epoch::from_gregorian_tai_at_midnight(2000, 1, 1);
    assert!((j2000.as_mjd_tai_days() - 51_544.0).abs() < EPSILON);
    assert!((j2000.as_jde_tai_days() - 2_451_544.5).abs() < EPSILON);

    assert!(
        Epoch::from_gregorian_tai_at_midnight(2000, 1, 1)
            < Epoch::from_gregorian_utc_at_midnight(2000, 1, 1),
        "TAI not ahead of UTC on J2k"
    );

    assert!(
        (Epoch::from_gregorian_utc_at_midnight(2000, 1, 1)
            - Epoch::from_gregorian_tai_at_midnight(2000, 1, 1)
            - 32.0)
            .abs()
            < EPSILON
    );

    let j2000 = Epoch::from_gregorian_utc_at_midnight(2000, 1, 1);
    assert!((j2000.as_mjd_utc_days() - 51_544.0).abs() < EPSILON);
    assert!((j2000.as_jde_utc_days() - 2_451_544.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2002-02-07+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let jd020207 = Epoch::from_gregorian_tai_at_midnight(2002, 2, 7);
    assert!((jd020207.as_mjd_tai_days() - 52_312.0).abs() < EPSILON);
    assert!((jd020207.as_jde_tai_days() - 2_452_312.5).abs() < EPSILON);

    // Test leap seconds and Julian at the same time
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // NOTE: Precision of HEASARC is less than hifitime, hence the last four digit difference
    // HEASARC reports 57203.99998843 but hifitime computes 57203.99998842592 (three additional)
    // significant digits.
    assert!(
        (Epoch::from_gregorian_tai_hms(2015, 6, 30, 23, 59, 59).as_mjd_tai_days()
            - 57_203.999_988_425_92)
            .abs()
            < EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (Epoch::from_gregorian_tai_hms(2015, 6, 30, 23, 59, 60).as_mjd_tai_days()
            - 57_203.999_988_425_92)
            .abs()
            < EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (Epoch::from_gregorian_tai_at_midnight(2015, 7, 1).as_mjd_tai_days() - 57_204.0).abs()
            < EPSILON,
        "Incorrect Post July 2015 leap second MJD computed"
    );
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
fn datetime_invalid_dates() {
    assert!(!is_gregorian_valid(2001, 2, 29, 22, 8, 47, 0));
    assert!(!is_gregorian_valid(2016, 12, 31, 23, 59, 61, 0));
    assert!(!is_gregorian_valid(2015, 6, 30, 23, 59, 61, 0));
}

#[test]
fn gpst() {
    use std::f64::EPSILON;
    // let now = Epoch::from_gregorian_utc_hms(2019, 8, 24, 3, 49, 9);
    let now = Epoch::from_gregorian_tai_hms(2019, 8, 24, 3, 49, 9);
    assert!(
        now.as_tai_seconds() > now.as_utc_seconds(),
        "TAI is not ahead of UTC"
    );
    assert!((now.as_tai_seconds() - now.as_utc_seconds() - 37.0).abs() < EPSILON);
    assert!(
        now.as_tai_seconds() > now.as_gpst_seconds(),
        "TAI is not head of GPS Time"
    );
    assert!((now.as_tai_seconds() - now.as_gpst_seconds() - 19.0).abs() < EPSILON);
    assert!(
        now.as_gpst_seconds() > now.as_utc_seconds(),
        "GPS Time is not head of UTC"
    );
    assert!((now.as_gpst_seconds() - now.as_utc_seconds() - 18.0).abs() < EPSILON);
}

#[test]
fn spice_et_tdb() {
    use crate::J2000_NAIF;
    /*
    >>> sp.str2et("2012-02-07 11:22:33 UTC")
    381885819.18493587
    >>> sp.et2utc(381885819.18493587, 'C', 9)
    '2012 FEB 07 11:22:33.000000000'
    >>> sp.et2utc(381885819.18493587, 'J', 9)
    'JD 2455964.9739931'
    */
    let sp_ex = Epoch::from_gregorian_utc_hms(2012, 2, 7, 11, 22, 33);
    let expected_et_s = 381_885_819.184_935_87;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s).abs() < 1e-6);
    assert!((sp_ex.as_jde_utc_days() - 2455964.9739931).abs() < 1e-7);
    assert!(
        dbg!(sp_ex.as_tai_seconds() - Epoch::from_et_seconds(expected_et_s).as_tai_seconds()).abs()
            < 1e-6
    );
    // Second example
    let sp_ex = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);
    let expected_et_s = 66_312_064.184_938_76;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s).abs() < 1e-6);
    assert!(
        (sp_ex.as_tai_seconds() - Epoch::from_et_seconds(expected_et_s).as_tai_seconds()).abs()
            < 1e-5
    );
    // Third example
    let sp_ex = Epoch::from_gregorian_utc_hms(1996, 2, 7, 11, 22, 33);
    let expected_et_s = -123_035_784.815_060_48;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s).abs() < 1e-6);
    assert!(
        (sp_ex.as_tai_seconds() - Epoch::from_et_seconds(expected_et_s).as_tai_seconds()).abs()
            < 1e-5
    );
    // Fourth example
    /*
    >>> sp.str2et("2015-02-07 11:22:33 UTC")
    476580220.1849411
    >>> sp.et2utc(476580220.1849411, 'C', 9)
    '2015 FEB 07 11:22:33.000000000'
    >>> sp.et2utc(476580220.1849411, 'J', 9)
    'JD 2457060.9739931'
    >>>
    */
    let sp_ex = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
    let expected_et_s = 476580220.1849411;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s).abs() < 1e-6);
    assert!((sp_ex.as_jde_utc_days() - 2457060.9739931).abs() < 1e-7);

    // JDE TDB tests
    /* Initial JDE from sp.et2utc:
    >>> sp.str2et("JD 2452312.500372511 TDB")
    66312032.18493909
    */
    let sp_ex = Epoch::from_et_seconds(66_312_032.184_939_09);
    assert!(dbg!(2452312.500372511 - sp_ex.as_jde_tdb_days()).abs() < std::f64::EPSILON);
    assert!(dbg!(2452312.500372511 - sp_ex.as_jde_et_days()).abs() < std::f64::EPSILON);

    let sp_ex = Epoch::from_et_seconds(381_885_753.003_859_5);
    assert!((2455964.9739931 - sp_ex.as_jde_tdb_days()).abs() < std::f64::EPSILON);

    let sp_ex = Epoch::from_et_seconds(0.0);
    assert!(dbg!(J2000_NAIF - sp_ex.as_jde_tdb_days()).abs() < 1e-7);
    assert!(dbg!(J2000_NAIF - sp_ex.as_jde_et_days()).abs() < std::f64::EPSILON);

    // This shows, I think, some approximation error.
    dbg!(Epoch::from_et_seconds(0.0).as_et_seconds());
    dbg!(Epoch::from_tdb_seconds(0.0).as_tdb_seconds());
}

#[test]
fn test_from_str() {
    let dt = Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 0);
    assert_eq!(
        dt,
        Epoch::from_gregorian_str("2017-01-14T00:31:55 UTC").unwrap()
    );
    assert_eq!(
        dt,
        Epoch::from_gregorian_str("2017-01-14T00:31:55").unwrap()
    );
    assert_eq!(
        dt,
        Epoch::from_gregorian_str("2017-01-14 00:31:55").unwrap()
    );
}

#[test]
fn ops() {
    // Test adding a second
    let sp_ex = Epoch::from_gregorian_utc_hms(2012, 2, 7, 11, 22, 33) + 1.0;
    let expected_et_s = 381_885_819.184_935_87;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s - 1.0).abs() < 1e-5);
    let sp_ex = sp_ex - 1.0;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s).abs() < 1e-5);
}

/// `quorem` returns a tuple of the quotient and the remainder a numerator and a denominator.
fn quorem(numerator: f64, denominator: f64) -> (i32, f64) {
    if denominator == 0.0 {
        panic!("cannot divide by zero");
    }
    let quotient = (numerator / denominator).floor() as i32;
    let remainder = numerator % denominator;
    if remainder >= 0.0 {
        (quotient, remainder)
    } else {
        (quotient - 1, remainder + denominator)
    }
}

#[test]
fn quorem_nominal_test() {
    assert_eq!(quorem(24.0, 6.0), (4, 0.0));
    assert_eq!(quorem(25.0, 6.0), (4, 1.0));
    assert_eq!(quorem(6.0, 6.0), (1, 0.0));
    assert_eq!(quorem(5.0, 6.0), (0, 5.0));
    assert_eq!(quorem(3540.0, 3600.0), (0, 3540.0));
    assert_eq!(quorem(3540.0, 60.0), (59, 0.0));
    assert_eq!(quorem(24.0, -6.0), (-4, 0.0));
    assert_eq!(quorem(-24.0, 6.0), (-4, 0.0));
    assert_eq!(quorem(-24.0, -6.0), (4, 0.0));
}

#[test]
#[should_panic]
fn quorem_nil_den_test() {
    assert_eq!(quorem(24.0, 0.0), (4, 0.0));
}
