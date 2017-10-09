use super::utils::{Errors, Offset};
use super::traits;
use super::instant::{Era, Instant};
use std::time::Duration;
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

impl traits::TimeZone for Utc
where
    Self: Sized,
{
    fn utc_offset() -> Offset {
        Offset::new(0, 0, Era::Present)
    }
    /// Creates a new Utc date. WARNING: Does not support automatic carry and will return an error
    /// if so.
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
            second > max_seconds || nanos as f64 > 1e9 ||
            (nanos > 0 && second == max_seconds)
        {
            return Err(Errors::Carry);
        }
        if day > USUAL_DAYS_PER_MONTH[month as usize - 1] {
            if month != 2 || !((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) {
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
// TODO: Convert an instant to a UTC
/*
impl traits::TimeSystem for Utc {
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
        let modifier: f64;
        if instant.era() == Era::Present {
            modifier = 1.0;
        } else {
            modifier = -1.0;
        }
        ModifiedJulian {
            days: J1900_OFFSET + modifier * (instant.secs() as f64) / SECONDS_PER_DAY +
                instant.nanos() as f64 * 1e-9,
        }
    }

    /// `as_instant` returns an Instant from the ModifiedJulian.
    fn as_instant(self) -> Utc {
        let era: Era;
        let modifier: f64;
        if self.days >= J1900_OFFSET {
            era = Era::Present;
            modifier = 1.0;
        } else {
            era = Era::Past;
            modifier = -1.0;
        }
        let secs_frac = (self.days - J1900_OFFSET) * SECONDS_PER_DAY * modifier;
        let seconds = secs_frac.round();
        let nanos = (secs_frac - seconds) * 1e9 / (SECONDS_PER_DAY * modifier);
        Instant::new(seconds as u64, nanos.round() as u32, era)
    }
}
*/
