/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::parser::Token;
use crate::{
    Duration, Epoch, Errors, ParsingErrors, TimeScale, Unit, DAYS_PER_YEAR_NLD, HIFITIME_REF_YEAR,
    NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND, NANOSECONDS_PER_SECOND_U32,
};
use core::str::FromStr;

use super::div_rem_f64;

impl Epoch {
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

/// `is_leap_year` returns whether the provided year is a leap year or not.
/// Tests for this function are part of the Datetime tests.
pub(crate) const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod ut_gregorian {
    use crate::epoch::gregorian::{is_leap_year, CUMULATIVE_DAYS_FOR_MONTH};

    #[test]
    fn cumulative_days_for_month() {
        assert_eq!(
            CUMULATIVE_DAYS_FOR_MONTH,
            [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]
        )
    }

    #[test]
    fn leap_year() {
        assert!(!is_leap_year(2019));
        assert!(!is_leap_year(2001));
        assert!(!is_leap_year(1000));
        // List of leap years from https://kalender-365.de/leap-years.php .
        let leap_years: [i32; 146] = [
            1804, 1808, 1812, 1816, 1820, 1824, 1828, 1832, 1836, 1840, 1844, 1848, 1852, 1856,
            1860, 1864, 1868, 1872, 1876, 1880, 1884, 1888, 1892, 1896, 1904, 1908, 1912, 1916,
            1920, 1924, 1928, 1932, 1936, 1940, 1944, 1948, 1952, 1956, 1960, 1964, 1968, 1972,
            1976, 1980, 1984, 1988, 1992, 1996, 2000, 2004, 2008, 2012, 2016, 2020, 2024, 2028,
            2032, 2036, 2040, 2044, 2048, 2052, 2056, 2060, 2064, 2068, 2072, 2076, 2080, 2084,
            2088, 2092, 2096, 2104, 2108, 2112, 2116, 2120, 2124, 2128, 2132, 2136, 2140, 2144,
            2148, 2152, 2156, 2160, 2164, 2168, 2172, 2176, 2180, 2184, 2188, 2192, 2196, 2204,
            2208, 2212, 2216, 2220, 2224, 2228, 2232, 2236, 2240, 2244, 2248, 2252, 2256, 2260,
            2264, 2268, 2272, 2276, 2280, 2284, 2288, 2292, 2296, 2304, 2308, 2312, 2316, 2320,
            2324, 2328, 2332, 2336, 2340, 2344, 2348, 2352, 2356, 2360, 2364, 2368, 2372, 2376,
            2380, 2384, 2388, 2392, 2396, 2400,
        ];
        for year in leap_years.iter() {
            assert!(is_leap_year(*year));
        }
    }
}
