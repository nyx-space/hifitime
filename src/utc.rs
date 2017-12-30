pub use super::TimeSystem;
use super::Errors;
use super::instant::{Era, Instant};
use super::julian::SECONDS_PER_DAY;
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

const USUAL_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const USUAL_DAYS_PER_YEAR: f64 = 365.0;

/// Offset is an alias of Instant. It contains the same kind of information, but is used in the
/// context of defining an offset with respect to Utc.
pub type Offset = Instant;

/// TimeZone defines a timezone with respect to Utc.
pub trait TimeZone: fmt::Display {
    /// utc_offset returns the difference between a given TZ and UTC.
    fn utc_offset() -> Offset;
    fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, Errors>
    where
        Self: Sized;
}

/// Utc is the interface between a time system and a time zone. All time zones are defined with
/// respect to UTC. Moreover, Utc inherently supports the past leap seconds, as reported by the
/// IETF and NIST at https://www.ietf.org/timezones/data/leap-seconds.list . NOTE: leap seconds
/// cannot be predicted! This module will be updated as soon as possible after a new leap second
/// has been announced.
/// **WARNING**: The historical oddities with calendars are not yet supported.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Utc {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanos: u32,
}

impl Utc {
    /// Returns the year of this Utc date time.
    pub fn year(&self) -> &i32 {
        &self.year
    }
    /// Returns the month of this Utc date time.
    pub fn month(&self) -> &u8 {
        &self.month
    }
    /// Returns the day of this Utc date time.
    pub fn day(&self) -> &u8 {
        &self.day
    }
    /// Returns the hour of this Utc date time.
    pub fn hour(&self) -> &u8 {
        &self.hour
    }
    /// Returns the minute of this Utc date time.
    pub fn minute(&self) -> &u8 {
        &self.minute
    }
    /// Returns the second of this Utc date time.
    pub fn second(&self) -> &u8 {
        &self.second
    }
    /// Returns the nanoseconds of this Utc date time.
    pub fn nanos(&self) -> &u32 {
        &self.nanos
    }
    /// Creates a new UTC date at midnight (i.e. hours = 0, mins = 0, secs = 0, nanos = 0)
    ///
    /// # Examples
    /// ```
    /// use hifitime::utc::{Utc, TimeSystem, TimeZone};
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let epoch = Utc::at_midnight(1900, 01, 01).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.as_instant(),
    ///     Instant::new(0, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Utc::at_midnight(1972, 01, 01)
    ///         .expect("Post January 1972 leap second failed")
    ///         .as_instant(),
    ///     Instant::new(2272060800, 0, Era::Present),
    ///     "Incorrect January 1972 post-leap second number computed at midnight"
    /// );
    /// ```

    pub fn at_midnight(year: i32, month: u8, day: u8) -> Result<Utc, Errors> {
        Ok(Utc::new(year, month, day, 00, 00, 00, 00)?)
    }

    /// Creates a new UTC date at noon (i.e. hours = 12, mins = 0, secs = 0, nanos = 0)
    ///
    /// # Examples
    /// ```
    /// use hifitime::utc::{Utc, TimeSystem, TimeZone};
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let epoch = Utc::at_noon(1900, 01, 01).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.as_instant(),
    ///     Instant::new(43200, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Utc::at_noon(1972, 01, 01)
    ///         .expect("Post January 1972 leap second failed")
    ///         .as_instant(),
    ///     Instant::new(2272104000, 0, Era::Present),
    ///     "Incorrect January 1972 post-leap second number computed at noon"
    /// );
    /// ```
    pub fn at_noon(year: i32, month: u8, day: u8) -> Result<Utc, Errors> {
        Ok(Utc::new(year, month, day, 12, 00, 00, 00)?)
    }
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
    /// **WARNING:** Does not support automatic carry and will return an error if so.
    /// **WARNING:** Although `PartialOrd` is implemented for Utc, the ambiguity of leap seconds
    /// as explained elsewhere in this documentation may lead to odd results (cf. examples below).
    ///
    /// # Examples
    /// ```
    /// use hifitime::utc::{Utc, TimeSystem, TimeZone};
    /// use hifitime::instant::{Duration, Era, Instant};
    /// use hifitime::julian::ModifiedJulian;
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
    ///
    /// // Example of odd behavior when comparing/ordering dates using Utc or `as_instant`
    /// // Utc order claims (correctly) that the 60th second is _after_ the 59th. But the instant
    /// // is actually different because the 60th second is where we've inserted the leap second.
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0).expect(
    ///         "January 1972 1 second before leap second failed",
    ///     ) <
    ///         Utc::new(1971, 12, 31, 23, 59, 60, 0).expect(
    ///             "January 1972 1 second before leap second failed",
    ///         ),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant() ==
    ///         Utc::new(1971, 12, 31, 23, 59, 60, 0)
    ///             .expect("January 1972 1 second before leap second failed")
    ///             .as_instant(),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// // Hence one second after the leap second, we get the following behavior (note the change
    /// // from equality to less when comparing via instant).
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 60, 0).expect(
    ///         "January 1972 1 second before leap second failed",
    ///     ) <
    ///         Utc::new(1972, 01, 01, 00, 00, 00, 0).expect(
    ///             "January 1972 1 second before leap second failed",
    ///         ),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 60, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant() <
    ///         Utc::new(1972, 01, 01, 00, 00, 00, 0)
    ///             .expect("January 1972 1 second before leap second failed")
    ///             .as_instant(),
    ///     "60th second should have a different instant than 59th second"
    /// );
    ///
    /// let santa = Utc::new(2017, 12, 25, 01, 02, 14, 0).expect("Xmas failed");
    ///
    /// assert_eq!(
    ///     santa.as_instant() + Duration::new(3600, 0),
    ///     Utc::new(2017, 12, 25, 02, 02, 14, 0)
    ///         .expect("Xmas failed")
    ///         .as_instant(),
    ///     "Could not add one hour to Christmas"
    /// );
    /// assert_eq!(format!("{}", santa), "2017-12-25T01:02:14+00:00");
    /// assert_eq!(
    ///     ModifiedJulian::from_instant(santa.as_instant()).days,
    ///     58112.043217592596
    /// );
    /// assert_eq!(
    ///     ModifiedJulian::from_instant(santa.as_instant()).julian_days(),
    ///     2458112.5432175924
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
}

impl TimeSystem for Utc {
    /// `from_instant` converts an Instant to a Utc.
    fn from_instant(instant: Instant) -> Utc {
        let (mut year, mut year_fraction) = quorem(instant.secs() as f64, 365.0 * SECONDS_PER_DAY);
        year = match instant.era() {
            Era::Past => 1900 - year,
            Era::Present => 1900 + year,
        };
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
                seconds_til_this_month += SECONDS_PER_DAY *
                    USUAL_DAYS_PER_MONTH[(month - 1) as usize] as f64;
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
            days_this_month as f64 * SECONDS_PER_DAY,
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
            day = USUAL_DAYS_PER_MONTH[(month - 1) as usize] as i32;
        }
        day += 1; // Otherwise the day count starts at 0
        // Get the hours by the exact number of seconds in an hour
        let (hours, hours_fraction) = quorem(day_fraction, 60.0 * 60.0);
        // Get the minutes and seconds by the exact number of seconds in a minute
        let (mins, secs) = quorem(hours_fraction, 60.0);
        Utc::new(
            year,
            month as u8,
            day as u8,
            hours as u8,
            mins as u8,
            secs as u8,
            instant.nanos(),
        ).expect("date computed from instant is invalid (past)")
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
        if is_leap_year(self.year) && self.month > 2 {
            // NOTE: If on 29th of February, then the day is not finished yet, and therefore
            // the extra seconds are added below as per a normal day.
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

/// is_leap_year returns whether the provided year is a leap year or not.
/// Tests for this function are part of the Utc tests.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// quorem returns a tuple of the quotient and the remainder a numerator and a denominator.
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
