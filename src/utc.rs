pub use super::utils::Offset;
pub use super::traits::{TimeZone, TimeSystem};
use super::utils::{Errors, quorem};
use super::instant::{Era, Instant};
use super::julian::SECONDS_PER_DAY;
use std::cmp::{PartialOrd, Ord, Eq, Ordering};
use std::fmt;
use std::marker::Sized;

// There is no way to define a constant map in Rust (yet), so we're combining several structures
// to store when the leap seconds should be added. An updated list of leap seconds can be found
// here: https://www.ietf.org/timezones/data/leap-seconds.list .
const JANUARY_YEARS: [i32; 17] = [
    1972,
    1973,
    1974,
    1975,
    1976,
    1977,
    1978,
    1979,
    1980,
    1988,
    1990,
    1991,
    1996,
    1999,
    2006,
    2009,
    2017,
];

const JULY_YEARS: [i32; 11] = [
    1972,
    1981,
    1982,
    1983,
    1985,
    1992,
    1993,
    1994,
    1997,
    2012,
    2015,
];

pub const USUAL_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const USUAL_DAYS_PER_YEAR: f64 = 365.0;

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Utc is the interface between a time system and a time zone. All time zones are defined with
/// respect to UTC. Moreover, Utc inherently supports the past leap seconds, as reported by the
/// IETF and NIST at https://www.ietf.org/timezones/data/leap-seconds.list . NOTE: leap seconds
/// cannot be predicted! This module will be updated as soon as possible after a new leap second
/// has been announced. WARNING: The historical oddities with calendars are not yet supported.
/// Moreover, despite the fields of Utc being public, it is recommended to use the `new` function
/// to ensure proper overflow.
#[derive(Copy, Clone, Debug)]
pub struct Utc {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32,
}

impl TimeZone for Utc
where
    Self: Sized,
{
    /// Returns the offset between this TimeZone and UTC. In this case, the offset is strictly zero.
    fn utc_offset() -> Offset {
        Offset::new(0, 0, Era::Present)
    }

    /// Creates a new UTC date, with support for all the leap seconds with respect to TAI.
    /// *NOTE:* UTC leap seconds may be confusing because several dates have the **same** number
    /// of seconds since TAI epoch.
    /// *WARNING:* Does not support automatic carry and will return an error if so.
    ///
    /// # Examples
    /// ```
    /// use hifitime::utc::{Utc, TimeZone, TimeSystem};
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let epoch = Utc::new(1900, 01, 01, 0, 0, 0, 0).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.as_instant(),
    ///     Instant::new(0, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 leap second failed")
    ///         .as_instant(),
    ///     Instant::new(2272060799, 0, Era::Present),
    ///     "Incorrect January 1972 pre-leap second number computed"
    /// );
    /// assert_eq!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant(),
    ///     Utc::new(1971, 12, 31, 23, 59, 60, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant(),
    ///     "Incorrect January 1972 leap second number computed"
    /// );
    /// ```
    fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Utc, Errors> {
        let mut max_seconds: u8 = 59;
        if (month == 12 || month == 6) && day == USUAL_DAYS_PER_MONTH[month as usize - 1] &&
            hour == 23 && minute == 59
        {
            if (month == 6 && JULY_YEARS.contains(&year)) ||
                (month == 12 && JANUARY_YEARS.contains(&(year + 1)))
            {
                max_seconds = 60;
            }
        }
        // General incorrect date times
        if month == 0 || month > 12 || day == 0 || day > 31 || hour > 24 || minute > 59 ||
            second > max_seconds || nanos as f64 > 1e9
        {
            return Err(Errors::Carry);
        }
        if day > USUAL_DAYS_PER_MONTH[month as usize - 1] {
            if month != 2 || !is_leap_year(year) {
                // Not in February or not a leap year
                return Err(Errors::Carry);
            }
        }
        Ok(Utc {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanos,
        })
    }
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl TimeSystem for Utc {
    /// `from_instant` converts an Instant to a ModifiedJulian as detailed
    /// in https://www.ietf.org/timezones/data/leap-seconds.list , specifically the following
    /// quote:
    /// The NTP timestamps are in units of seconds since the NTP epoch,
    /// which is 1 January 1900, 00:00:00. The Modified Julian Day number
    /// corresponding to the NTP time stamp, X, can be computed as
    ///
    /// X/86400 + 15020
    ///
    /// where the first term converts seconds to days and the second
    /// term adds the MJD corresponding to the time origin defined above.
    /// The integer portion of the result is the integer MJD for that
    /// day, and any remainder is the time of day, expressed as the
    /// fraction of the day since 0 hours UTC. The conversion from day
    /// fraction to seconds or to hours, minutes, and seconds may involve
    /// rounding or truncation, depending on the method used in the
    /// computation.
    fn from_instant(instant: Instant) -> Utc {
        let (year, year_fraction) = quorem(instant.secs() as f64, 365.0 * SECONDS_PER_DAY);
        let (mut month, month_fraction) = quorem(year_fraction, 30.4365 * SECONDS_PER_DAY);
        month += 1; // Otherwise the month count starts at 0
        let mut days_this_month = USUAL_DAYS_PER_MONTH[(month - 1) as usize];
        if month == 2 && is_leap_year(year) {
            days_this_month += 1;
        }
        let (mut day, day_fraction) =
            quorem(month_fraction, SECONDS_PER_DAY * days_this_month as f64);
        day += 1; // Otherwise the day count starts at 0
        let (hours, hours_fraction) = quorem(day_fraction, 60.0 * 60.0);
        let (mins, secs) = quorem(hours_fraction, 60.0);
        match instant.era() {
            Era::Past => {
                Utc::new(
                    1900 - year,
                    month as u8,
                    day as u8,
                    hours as u8,
                    mins as u8,
                    secs as u8,
                    instant.nanos(),
                ).expect("date computed from instant is invalid (past)")
            }
            Era::Present => {
                Utc::new(
                    1900 + year,
                    month as u8,
                    day as u8,
                    hours as u8,
                    mins as u8,
                    secs as u8,
                    instant.nanos(),
                ).expect("date computed from instant is invalid")
            }
        }
    }

    /// `as_instant` returns an Instant from the Utc.
    fn as_instant(self) -> Instant {
        let era: Era;
        if self.year >= 1900 {
            era = Era::Present;
        } else {
            era = Era::Past;
        }

        let mut seconds_wrt_1900: f64 = ((self.year - 1900).abs() as f64) * SECONDS_PER_DAY *
            USUAL_DAYS_PER_YEAR;

        // Now add the seconds for all the years prior to the current year
        for year in 1900..self.year {
            if is_leap_year(year) {
                seconds_wrt_1900 += SECONDS_PER_DAY;
            }
        }
        // Add the seconds for the months prior to the current month
        for month in 0..self.month - 1 {
            seconds_wrt_1900 += SECONDS_PER_DAY * USUAL_DAYS_PER_MONTH[(month) as usize] as f64;
        }
        if is_leap_year(self.year) && ((self.month == 2 && self.day == 29) || self.month > 2) {
            seconds_wrt_1900 += SECONDS_PER_DAY;
        }
        seconds_wrt_1900 += (self.day - 1) as f64 * SECONDS_PER_DAY + self.hour as f64 * 3600.0 +
            self.minute as f64 * 60.0 +
            self.second as f64;
        if self.second == 60 {
            // Herein lies the whole ambiguity of leap seconds. Two different UTC dates exist at the
            // same number of second afters J1900.0.
            seconds_wrt_1900 -= 1.0;
        }
        Instant::new(seconds_wrt_1900 as u64, self.nanos as u32, era)
    }
}

impl fmt::Display for Utc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        )
    }
}

impl PartialEq for Utc {
    fn eq(&self, other: &Utc) -> bool {
        self.as_instant() == other.as_instant()
    }
}

impl PartialOrd for Utc {
    fn partial_cmp(&self, other: &Utc) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Utc) -> bool {
        self.as_instant() < other.as_instant()
    }
    fn le(&self, other: &Utc) -> bool {
        self.as_instant() <= other.as_instant()
    }
    fn gt(&self, other: &Utc) -> bool {
        self.as_instant() > other.as_instant()
    }
    fn ge(&self, other: &Utc) -> bool {
        self.as_instant() >= other.as_instant()
    }
}
