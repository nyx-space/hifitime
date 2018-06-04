extern crate regex;

pub use super::instant::{Duration, Era, Instant};
pub use super::TimeSystem;
use super::{Errors, SECONDS_PER_DAY};
use std::fmt;
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

// There is no way to define a constant map in Rust (yet), so we're combining several structures
// to store when the leap seconds should be added. An updated list of leap seconds can be found
// here: https://www.ietf.org/timezones/data/leap-seconds.list .
const JANUARY_YEARS: [i32; 17] = [
    1972, 1973, 1974, 1975, 1976, 1977, 1978, 1979, 1980, 1988, 1990, 1991, 1996, 1999, 2006, 2009,
    2017,
];

const JULY_YEARS: [i32; 11] = [
    1972, 1981, 1982, 1983, 1985, 1992, 1993, 1994, 1997, 2012, 2015,
];

const USUAL_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const USUAL_DAYS_PER_YEAR: f64 = 365.0;

/// `Offset` is an alias of Instant. It contains the same kind of information, but is used in the
/// context of defining an offset with respect to Utc.
pub type Offset = Instant;

/// Negates an Offset
///
/// # Examples
/// ```
/// use hifitime::datetime::Offset;
/// use hifitime::instant::Era;
///
/// assert_eq!(
///     -Offset::new(3600, 159, Era::Past),
///     Offset::new(3600, 159, Era::Present),
///     "Incorrect neg for Past offset"
/// );
///
/// assert_eq!(
///     -Offset::new(3600, 159, Era::Present),
///     Offset::new(3600, 159, Era::Past),
///     "Incorrect neg for Present offset"
/// );
/// ```
impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Offset {
        let era = match self.era() {
            Era::Past => Era::Present,
            Era::Present => Era::Past,
        };
        Offset::new(self.secs(), self.nanos(), era)
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sign = match self.era() {
            Era::Present => "+",
            Era::Past => "-",
        };
        let (hours, hours_fraction) = quorem(self.secs() as f64, 60.0 * 60.0);
        // Get the minutes and seconds by the exact number of seconds in a minute
        let (mins, _) = quorem(hours_fraction, 60.0);
        write!(f, "{:}{:02}:{:02}", sign, hours, mins)
    }
}

/// `FixedOffset` implements a time fixed offset of a certain number of hours with regard to UTC.
pub struct FixedOffset {}

impl FixedOffset {
    /// `east_with_hours` returns an eastward offset (i.e. "before" the UTC time)
    ///
    /// # Example
    /// ```
    /// use hifitime::datetime::FixedOffset;
    /// use hifitime::instant::Era;
    ///
    /// let whiskey_tz = FixedOffset::east_with_hours(10);
    /// assert_eq!(
    ///     whiskey_tz.secs(),
    ///     36000,
    ///     "Incorrect number of hours computed"
    /// );
    /// assert_eq!(
    ///     whiskey_tz.era(),
    ///     Era::Past,
    ///     "Incorrect era used"
    /// );
    /// ```
    pub fn east_with_hours(hours: u64) -> Offset {
        Offset::new(hours * 3600, 0, Era::Past)
    }

    /// `west_with_hours` returns an eastward offset (i.e. "before" the UTC time)
    ///
    /// # Example
    /// ```
    /// use hifitime::datetime::FixedOffset;
    /// use hifitime::instant::Era;
    ///
    /// let kilo_tz = FixedOffset::west_with_hours(10);
    /// assert_eq!(
    ///     kilo_tz.secs(),
    ///     36000,
    ///     "Incorrect number of hours computed"
    /// );
    /// assert_eq!(
    ///     kilo_tz.era(),
    ///     Era::Present,
    ///     "Incorrect era used"
    /// );
    /// ```
    pub fn west_with_hours(hours: u64) -> Offset {
        Offset::new(hours * 3600, 0, Era::Present)
    }
}

/// Datetime supports date time has used by most humans. All time zones are defined with
/// respect to UTC. Moreover, `Datetime` inherently supports the past leap seconds, as reported by the
/// IETF and NIST at [here](https://www.ietf.org/timezones/data/leap-seconds.list). NOTE: leap seconds
/// cannot be predicted! This module will be updated as soon as possible after a new leap second
/// has been announced.
/// **WARNING**: The historical oddities with calendars are not yet supported.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Datetime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanos: u32,
    offset: Offset,
}

impl Datetime {
    /// Creates a new UTC-offsetted datetime, with support for all the leap seconds with respect to TAI.
    /// *NOTE:* UTC leap seconds may be confusing because several dates have the **same** number
    /// of seconds since TAI epoch.
    /// **WARNING:** Does not support automatic carry and will return an error if so.
    /// **WARNING:** Although `PartialOrd` is implemented for Utc, the ambiguity of leap seconds
    /// as explained elsewhere in this documentation may lead to odd results (cf. examples below).
    ///
    /// # Examples
    /// ```
    /// use hifitime::datetime::{Datetime, TimeSystem};
    /// use hifitime::instant::{Duration, Era, Instant};
    /// use hifitime::julian::ModifiedJulian;
    ///
    /// let epoch = Datetime::new(1900, 01, 01, 0, 0, 0, 0).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.into_instant(),
    ///     Instant::new(0, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Datetime::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 leap second failed")
    ///         .into_instant(),
    ///     Instant::new(2272060799, 0, Era::Present),
    ///     "Incorrect January 1972 pre-leap second number computed"
    /// );
    /// assert_eq!(
    ///     Datetime::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .into_instant(),
    ///     Datetime::new(1971, 12, 31, 23, 59, 60, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .into_instant(),
    ///     "Incorrect January 1972 leap second number computed"
    /// );
    ///
    /// // Example of odd behavior when comparing/ordering dates using Utc or `into_instant`
    /// // Utc order claims (correctly) that the 60th second is _after_ the 59th. But the instant
    /// // is actually different because the 60th second is where we've inserted the leap second.
    /// assert!(
    ///     Datetime::new(1971, 12, 31, 23, 59, 59, 0).expect(
    ///         "January 1972 1 second before leap second failed",
    ///     ) <
    ///         Datetime::new(1971, 12, 31, 23, 59, 60, 0).expect(
    ///             "January 1972 1 second before leap second failed",
    ///         ),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// assert!(
    ///     Datetime::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .into_instant() ==
    ///         Datetime::new(1971, 12, 31, 23, 59, 60, 0)
    ///             .expect("January 1972 1 second before leap second failed")
    ///             .into_instant(),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// // Hence one second after the leap second, we get the following behavior (note the change
    /// // from equality to less when comparing via instant).
    /// assert!(
    ///     Datetime::new(1971, 12, 31, 23, 59, 60, 0).expect(
    ///         "January 1972 1 second before leap second failed",
    ///     ) <
    ///         Datetime::new(1972, 01, 01, 00, 00, 00, 0).expect(
    ///             "January 1972 1 second before leap second failed",
    ///         ),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// assert!(
    ///     Datetime::new(1971, 12, 31, 23, 59, 60, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .into_instant() <
    ///         Datetime::new(1972, 01, 01, 00, 00, 00, 0)
    ///             .expect("January 1972 1 second before leap second failed")
    ///             .into_instant(),
    ///     "60th second should have a different instant than 59th second"
    /// );
    ///
    /// let santa = Datetime::new(2017, 12, 25, 01, 02, 14, 0).expect("Xmas failed");
    ///
    /// assert_eq!(
    ///     santa.into_instant() + Duration::new(3600, 0),
    ///     Datetime::new(2017, 12, 25, 02, 02, 14, 0)
    ///         .expect("Xmas failed")
    ///         .into_instant(),
    ///     "Could not add one hour to Christmas"
    /// );
    /// assert_eq!(format!("{}", santa), "2017-12-25T01:02:14+00:00");
    /// assert_eq!(
    ///     ModifiedJulian::from_instant(santa.into_instant()).days,
    ///     58112.043217592596
    /// );
    /// assert_eq!(
    ///     ModifiedJulian::from_instant(santa.into_instant()).julian_days(),
    ///     2458112.5432175924
    /// );
    /// ```
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Datetime, Errors> {
        Datetime::with_offset(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanos,
            FixedOffset::west_with_hours(0),
        )
    }

    /// Creates a new Datetime with the specified UTC time offset. Works like `Datetime::new` in
    /// every way but it sets the UTC time offset to the one provided.
    ///
    /// # Examples
    /// ```
    /// use hifitime::datetime::{Datetime, FixedOffset, TimeSystem};
    ///
    /// let santa_ktz = Datetime::with_offset(
    ///     2017,
    ///     12,
    ///     25,
    ///     00,
    ///     00,
    ///     00,
    ///     00,
    ///     FixedOffset::west_with_hours(10),
    /// ).expect("Santa failed");
    /// assert_eq!(format!("{}", santa_ktz), "2017-12-25T00:00:00+10:00");

    /// let santa_wtz = Datetime::with_offset(
    ///     2017,
    ///     12,
    ///     25,
    ///     00,
    ///     00,
    ///     00,
    ///     00,
    ///     FixedOffset::east_with_hours(10),
    /// ).expect("Santa failed");
    /// assert_eq!(format!("{}", santa_wtz), "2017-12-25T00:00:00-10:00");
    /// assert!(
    ///     santa_wtz < santa_ktz,
    ///     "PartialOrd with different timezones failed"
    /// );
    /// assert!(
    ///     santa_wtz.into_instant() < santa_ktz.into_instant(),
    ///     "PartialOrd with different timezones failed via Instant"
    /// );
    /// assert_eq!(
    ///     format!("{}", santa_wtz.to_utc()),
    ///     "2017-12-24T14:00:00+00:00"
    /// );
    /// ```
    pub fn with_offset(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        offset: Offset,
    ) -> Result<Datetime, Errors> {
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
            return Err(Errors::Carry);
        }
        if day > USUAL_DAYS_PER_MONTH[month as usize - 1] && (month != 2 || !is_leap_year(year)) {
            // Not in February or not a leap year
            return Err(Errors::Carry);
        }
        Ok(Datetime {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            nanos: nanos,
            offset: offset,
        })
    }

    /// Returns the year of this Datetime date time.
    pub fn year(&self) -> &i32 {
        &self.year
    }
    /// Returns the month of this Datetime date time.
    pub fn month(&self) -> &u8 {
        &self.month
    }
    /// Returns the day of this Datetime date time.
    pub fn day(&self) -> &u8 {
        &self.day
    }
    /// Returns the hour of this Datetime date time.
    pub fn hour(&self) -> &u8 {
        &self.hour
    }
    /// Returns the minute of this Datetime date time.
    pub fn minute(&self) -> &u8 {
        &self.minute
    }
    /// Returns the second of this Datetime date time.
    pub fn second(&self) -> &u8 {
        &self.second
    }
    /// Returns the nanoseconds of this Datetime date time.
    pub fn nanos(&self) -> &u32 {
        &self.nanos
    }

    /// Returns the offset of this Datetime date time.
    pub fn offset(&self) -> &Offset {
        &self.offset
    }
    /// Creates a new UTC date at midnight (i.e. hours = 0, mins = 0, secs = 0, nanos = 0)
    ///
    /// # Examples
    /// ```
    /// use hifitime::datetime::{Datetime, TimeSystem};
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let epoch = Datetime::at_midnight(1900, 01, 01).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.into_instant(),
    ///     Instant::new(0, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Datetime::at_midnight(1972, 01, 01)
    ///         .expect("Post January 1972 leap second failed")
    ///         .into_instant(),
    ///     Instant::new(2272060800, 0, Era::Present),
    ///     "Incorrect January 1972 post-leap second number computed at midnight"
    /// );
    /// ```

    pub fn at_midnight(year: i32, month: u8, day: u8) -> Result<Datetime, Errors> {
        Ok(Datetime::new(year, month, day, 00, 00, 00, 00)?)
    }

    /// Creates a new UTC date at noon (i.e. hours = 12, mins = 0, secs = 0, nanos = 0)
    ///
    /// # Examples
    /// ```
    /// use hifitime::datetime::{Datetime, TimeSystem};
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let epoch = Datetime::at_noon(1900, 01, 01).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.into_instant(),
    ///     Instant::new(43200, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Datetime::at_noon(1972, 01, 01)
    ///         .expect("Post January 1972 leap second failed")
    ///         .into_instant(),
    ///     Instant::new(2272104000, 0, Era::Present),
    ///     "Incorrect January 1972 post-leap second number computed at noon"
    /// );
    /// ```
    pub fn at_noon(year: i32, month: u8, day: u8) -> Result<Datetime, Errors> {
        Ok(Datetime::new(year, month, day, 12, 00, 00, 00)?)
    }

    pub fn to_utc(self) -> Datetime {
        self.to_offset(FixedOffset::east_with_hours(0))
    }

    pub fn to_offset(self, offset: Offset) -> Datetime {
        // Start by canceling the initial offset and then apply the desired one
        match offset.era() {
            Era::Past => Datetime::from_instant(self.into_instant() - offset.duration()),
            Era::Present => Datetime::from_instant(self.into_instant() + offset.duration()),
        }
    }
}

impl TimeSystem for Datetime {
    /// `from_instant` converts an Instant to a Datetime with an offset of Utc (i.e zero).
    fn from_instant(instant: Instant) -> Datetime {
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
        Datetime::new(
            year,
            month as u8,
            day as u8,
            hours as u8,
            mins as u8,
            secs as u8,
            instant.nanos(),
        ).expect("date computed from instant is invalid (past)")
    }

    /// `into_instant` returns an Instant from the Datetime while correcting for the offset.
    fn into_instant(self) -> Instant {
        let era = if self.year >= 1900 {
            Era::Present
        } else {
            Era::Past
        };

        let mut seconds_wrt_1900: f64 =
            f64::from((self.year - 1900).abs()) * SECONDS_PER_DAY * USUAL_DAYS_PER_YEAR;

        // Now add the seconds for all the years prior to the current year
        for year in 1900..self.year {
            if is_leap_year(year) {
                seconds_wrt_1900 += SECONDS_PER_DAY;
            }
        }
        // Add the seconds for the months prior to the current month
        for month in 0..self.month - 1 {
            seconds_wrt_1900 += SECONDS_PER_DAY * f64::from(USUAL_DAYS_PER_MONTH[(month) as usize]);
        }
        if is_leap_year(self.year) && self.month > 2 {
            // NOTE: If on 29th of February, then the day is not finished yet, and therefore
            // the extra seconds are added below as per a normal day.
            seconds_wrt_1900 += SECONDS_PER_DAY;
        }
        seconds_wrt_1900 += f64::from(self.day - 1) * SECONDS_PER_DAY
            + f64::from(self.hour) * 3600.0
            + f64::from(self.minute) * 60.0
            + f64::from(self.second);
        if self.second == 60 {
            // Herein lies the whole ambiguity of leap seconds. Two different UTC dates exist at the
            // same number of second afters J1900.0.
            seconds_wrt_1900 -= 1.0;
        }
        match self.offset.era() {
            Era::Past => {
                Instant::new(seconds_wrt_1900 as u64, self.nanos as u32, era)
                    - self.offset.duration()
            }
            Era::Present => {
                Instant::new(seconds_wrt_1900 as u64, self.nanos as u32, era)
                    + self.offset.duration()
            }
        }
    }
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{:}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.offset
        )
    }
}

impl fmt::LowerHex for Datetime {
    /// Formats as human readable with date and time separated by a space and no offset.
    ///
    /// # Example
    /// ```
    /// use std::str::FromStr;
    /// use hifitime::datetime::{Datetime, FixedOffset};
    ///
    /// let dt =
    ///     Datetime::with_offset(2017, 1, 14, 0, 31, 55, 0, FixedOffset::east_with_hours(5)).unwrap();
    /// assert_eq!(format!("{:x}", dt), "2017-01-14 00:31:55");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl fmt::UpperHex for Datetime {
    /// Formats as ISO8601 but _without_ the offset.
    ///
    /// # Example
    /// ```
    /// use std::str::FromStr;
    /// use hifitime::datetime::{Datetime, FixedOffset};
    ///
    /// let dt =
    ///     Datetime::with_offset(2017, 1, 14, 0, 31, 55, 0, FixedOffset::east_with_hours(5)).unwrap();
    /// assert_eq!(format!("{:X}", dt), "2017-01-14T00:31:55");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl Add<Duration> for Datetime {
    type Output = Datetime;

    /// Adds a given `std::time::Duration` to a `Datetime`.
    ///
    /// # Examples
    /// ```
    /// use hifitime::datetime::Datetime;
    /// use std::time::Duration;
    /// let santa = Datetime::at_midnight(2017, 12, 25).unwrap();
    /// let santa_1h = Datetime::at_midnight(2017, 12, 25).unwrap() + Duration::new(3600, 0);
    /// assert_eq!(santa.hour() + &1, *santa_1h.hour());
    /// ```
    fn add(self, delta: Duration) -> Datetime {
        Datetime::from_instant(self.into_instant() + delta)
    }
}

impl Sub<Duration> for Datetime {
    type Output = Datetime;

    /// Adds a given `std::time::Duration` to a `Datetime`.
    ///
    /// # Examples
    /// ```
    /// use hifitime::datetime::Datetime;
    /// use std::time::Duration;
    /// let santa = Datetime::at_midnight(2017, 12, 25).unwrap();
    /// let santa_1h = Datetime::at_midnight(2017, 12, 25).unwrap() - Duration::new(3600, 0);
    /// assert_eq!(santa.day() - &1, *santa_1h.day()); // Day underflow
    /// assert_eq!(santa_1h.hour(), &23);
    /// ```
    fn sub(self, delta: Duration) -> Datetime {
        Datetime::from_instant(self.into_instant() - delta)
    }
}

impl FromStr for Datetime {
    type Err = Errors;

    /// Converts an ISO8601 Datetime representation with offset to a `Datetime` object with correct offset.
    /// The `T` which separates the date from the time can be replaced with a single whitespace character (`\W`).
    /// The offset is also optional, cf. the examples below.
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use hifitime::datetime::{Datetime, Offset};
    /// use hifitime::instant::Era;
    /// let offset = Offset::new(3600 * 2 + 60 * 15, 0, Era::Past);
    /// let dt = Datetime::with_offset(2017, 1, 14, 0, 31, 55, 0, offset).unwrap();
    /// assert_eq!(
    ///     dt,
    ///     Datetime::from_str("2017-01-14T00:31:55-02:15").unwrap()
    /// );
    /// assert_eq!(
    ///     dt,
    ///     Datetime::from_str("2017-01-14 00:31:55-02:15").unwrap()
    /// );
    /// let dt = Datetime::new(2017, 1, 14, 0, 31, 55, 0).unwrap();
    /// assert_eq!(
    ///     dt,
    ///     Datetime::from_str("2017-01-14T00:31:55").unwrap()
    /// );
    /// assert_eq!(
    ///     dt,
    ///     Datetime::from_str("2017-01-14 00:31:55").unwrap()
    /// );
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(\d{4})-(\d{2})-(\d{2})(?:T|\W)(\d{2}):(\d{2}):(\d{2})(([\+|-]\d{2}):(\d{2}))?$"
            ).unwrap();
        }
        match RE.captures(s) {
            Some(cap) => {
                let offset = match cap.get(7) {
                    Some(_) => {
                        let offset_hours = cap.get(8).unwrap().as_str().to_owned().parse::<i32>()?;
                        let offset_mins = cap.get(9).unwrap().as_str().to_owned().parse::<i32>()?;
                        // Check if negative, and if so, multiply by negative seconds to get a positive number
                        if offset_hours < 0 {
                            Offset::new(
                                (-3600 * offset_hours + 60 * offset_mins) as u64,
                                0,
                                Era::Past,
                            )
                        } else {
                            Offset::new(
                                (3600 * offset_hours + 60 * offset_mins) as u64,
                                0,
                                Era::Present,
                            )
                        }
                    }
                    None => Offset::new(0, 0, Era::Present),
                };
                Datetime::with_offset(
                    cap[1].to_owned().parse::<i32>()?,
                    cap[2].to_owned().parse::<u8>()?,
                    cap[3].to_owned().parse::<u8>()?,
                    cap[4].to_owned().parse::<u8>()?,
                    cap[5].to_owned().parse::<u8>()?,
                    cap[6].to_owned().parse::<u8>()?,
                    0,
                    offset,
                )
            }
            None => Err(Errors::ParseError(
                "Input not in ISO8601 format with offset (e.g. 2018-01-27T00:41:55+03:00)"
                    .to_owned(),
            )),
        }
    }
}

/// `is_leap_year` returns whether the provided year is a leap year or not.
/// Tests for this function are part of the Datetime tests.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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
