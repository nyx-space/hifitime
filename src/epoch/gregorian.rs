/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::errors::DurationError;
use crate::parser::Token;
use crate::{
    Duration, Epoch, EpochError, ParsingError, TimeScale, Unit, DAYS_PER_YEAR_NLD,
    HIFITIME_REF_YEAR, NANOSECONDS_PER_MICROSECOND, NANOSECONDS_PER_MILLISECOND,
    NANOSECONDS_PER_SECOND_U32,
};
use core::str::FromStr;

use super::div_rem_f64;

impl Epoch {
    pub(crate) fn compute_gregorian(
        duration: Duration,
        time_scale: TimeScale,
    ) -> (i32, u8, u8, u8, u8, u8, u32) {
        let duration_wrt_ref = duration + time_scale.gregorian_epoch_offset();
        let sign = duration_wrt_ref.signum();
        let (days, hours, minutes, seconds, milliseconds, microseconds, nanos) = if sign < 0 {
            // For negative epochs, the computation of days and time must account for the time as it'll cause the days computation to be off by one.
            let (_, days, hours, minutes, seconds, milliseconds, microseconds, nanos) =
                duration_wrt_ref.decompose();

            // Recompute the time since we count backward and not forward for negative durations.
            let time = Duration::compose(
                0,
                0,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanos,
            );

            // Compute the correct time.
            let (_, _, hours, minutes, seconds, milliseconds, microseconds, nanos) =
                (24 * Unit::Hour - time).decompose();

            let days_f64 = if time > Duration::ZERO {
                -((days + 1) as f64)
            } else {
                -(days as f64)
            };

            (
                days_f64,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanos,
            )
        } else {
            // For positive epochs, the computation of days and time is trivally the decomposition of the duration.
            let (_, days, hours, minutes, seconds, milliseconds, microseconds, nanos) =
                duration_wrt_ref.decompose();
            (
                days as f64,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanos,
            )
        };

        let (mut year, mut days_in_year) = div_rem_f64(days, DAYS_PER_YEAR_NLD);
        year += HIFITIME_REF_YEAR;

        // Base calculation was on 365 days, so we need to remove one day per leap year
        if year >= HIFITIME_REF_YEAR {
            for y in HIFITIME_REF_YEAR..year {
                if is_leap_year(y) {
                    days_in_year -= 1.0;
                }
            }
            if days_in_year < 0.0 {
                // We've underflowed the number of days in a year because of the leap years
                year -= 1;
                days_in_year += DAYS_PER_YEAR_NLD;
                // If we had incorrectly removed one day of the year in the previous loop, fix it here.
                if is_leap_year(year) {
                    days_in_year += 1.0;
                }
            }
        } else {
            for y in year..HIFITIME_REF_YEAR {
                if is_leap_year(y) {
                    days_in_year += 1.0;
                }
            }
            // Check for greater than or equal because the days are still zero indexed here.
            if (days_in_year >= DAYS_PER_YEAR_NLD && !is_leap_year(year))
                || (days_in_year >= DAYS_PER_YEAR_NLD + 1.0 && is_leap_year(year))
            {
                // We've overflowed the number of days in a year because of the leap years
                year += 1;
                days_in_year -= DAYS_PER_YEAR_NLD;
            }
        }

        let cumul_days = if is_leap_year(year) {
            CUMULATIVE_DAYS_FOR_MONTH_LEAP_YEARS
        } else {
            CUMULATIVE_DAYS_FOR_MONTH
        };

        let month_search = cumul_days.binary_search(&(days_in_year as u16));
        let month = match month_search {
            Ok(index) => index + 1, // Exact month found, add 1 for month number (indexing starts from 0)
            Err(insertion_point) => insertion_point, // We're before the number of months, so use the insertion point as the month number
        };

        // Directly compute the day from the computed month, and ensure that day counter is one indexed.
        let day = days_in_year - cumul_days[month - 1] as f64 + 1.0;

        (
            year,
            month as u8,
            day as u8,
            hours as u8,
            minutes as u8,
            seconds as u8,
            (nanos
                + microseconds * NANOSECONDS_PER_MICROSECOND
                + milliseconds * NANOSECONDS_PER_MILLISECOND) as u32,
        )
    }

    #[cfg(feature = "std")]
    #[must_use]
    /// Converts the Epoch to Gregorian in the provided time scale and in the ISO8601 format with the time scale appended to the string
    pub fn to_gregorian_str(&self, time_scale: TimeScale) -> String {
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(time_scale), time_scale);

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
    /// let dt_tai = Epoch::from_tai_parts(1, 537582752000000000);
    ///
    /// let dt_str = "2017-01-14T00:31:55 UTC";
    /// let dt = Epoch::from_gregorian_str(dt_str).unwrap();
    ///
    /// let (y, m, d, h, min, s, _) = dt_tai.to_gregorian_utc();
    /// assert_eq!(y, 2017);
    /// assert_eq!(m, 1);
    /// assert_eq!(d, 14);
    /// assert_eq!(h, 0);
    /// assert_eq!(min, 31);
    /// assert_eq!(s, 55);
    /// #[cfg(feature = "std")]
    /// {
    /// assert_eq!("2017-01-14T00:31:55 UTC", format!("{dt_tai:?}"));
    /// // dt_tai is initialized from TAI, so the default print is the Gregorian in that time system
    /// assert_eq!("2017-01-14T00:32:32 TAI", format!("{dt_tai}"));
    /// // But dt is initialized from UTC, so the default print and the debug print are both in UTC.
    /// assert_eq!("2017-01-14T00:31:55 UTC", format!("{dt}"));
    /// }
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
    ) -> Result<Self, EpochError> {
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
    ///
    /// Note:
    /// The month is ONE indexed, i.e. January is month 1 and December is month 12.
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
    ) -> Result<Self, EpochError> {
        if !is_gregorian_valid(year, month, day, hour, minute, second, nanos) {
            return Err(EpochError::InvalidGregorianDate);
        }

        let mut duration_wrt_ref = match year.checked_sub(HIFITIME_REF_YEAR) {
            None => {
                return Err(EpochError::Duration {
                    source: DurationError::Underflow,
                })
            }
            Some(years_since_ref) => match years_since_ref.checked_mul(DAYS_PER_YEAR_NLD as i32) {
                None => {
                    return Err(EpochError::Duration {
                        source: DurationError::Overflow,
                    })
                }
                Some(days) => {
                    // Initialize the duration as the number of days since the reference year (may be negative).
                    Unit::Day * i64::from(days)
                }
            },
        };

        // Now add the leap days for all the years prior to the current year
        if year >= HIFITIME_REF_YEAR {
            // Add days until, but not including, current year.
            for y in HIFITIME_REF_YEAR..year {
                if is_leap_year(y) {
                    duration_wrt_ref += Unit::Day;
                }
            }
        } else {
            // Remove days
            for y in year..HIFITIME_REF_YEAR {
                if is_leap_year(y) {
                    duration_wrt_ref -= Unit::Day;
                }
            }
        }

        // Add the seconds for the months prior to the current month.
        // Correctly accounts for the number of days based on whether this is a leap year or not.
        let cumul_days = if is_leap_year(year) {
            CUMULATIVE_DAYS_FOR_MONTH_LEAP_YEARS
        } else {
            CUMULATIVE_DAYS_FOR_MONTH
        };

        // Add the number of days based on the input month
        duration_wrt_ref += Unit::Day * i64::from(cumul_days[(month - 1) as usize]);
        // Add the number of days based on the input day and time.
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

        // Account for this time scale's Gregorian offset.
        duration_wrt_ref -= time_scale.gregorian_epoch_offset();

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
    ) -> Result<Self, EpochError> {
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
    pub fn from_gregorian_str(s_in: &str) -> Result<Self, EpochError> {
        // All of the integers in a date: year, month, day, hour, minute, second, subsecond, offset hours, offset minutes

        use snafu::ResultExt;

        use crate::errors::ParseSnafu;

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
                        ts = TimeScale::from_str(s[idx..].trim()).with_context(|_| ParseSnafu {
                            details: "parsing as Gregorian date with time scale",
                        })?;
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
                    return Err(EpochError::Parse {
                        source: ParsingError::ISO8601,
                        details: "parsing as Gregorian",
                    });
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
                    Err(err) => {
                        return Err(EpochError::Parse {
                            source: ParsingError::Lexical { err },
                            details: "parsing as Gregorian",
                        })
                    }
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
        && day == usual_days_per_month(month)
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
    if day > usual_days_per_month(month) && (month != 2 || !is_leap_year(year)) {
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

/// Returns the usual days in a given month (ONE indexed, i.e. January is month ONE and December is month 12)
///
/// # Warning
/// This will return 0 days if the month is invalid.
const fn usual_days_per_month(month: u8) -> u8 {
    match month {
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
        days[month] = days[month - 1] + usual_days_per_month(month as u8) as u16;
        month += 1;
    }
    days
};

/// Calculates the prefix-sum of days counted up to the month start, for leap years only
const CUMULATIVE_DAYS_FOR_MONTH_LEAP_YEARS: [u16; 12] = {
    let mut days = [0; 12];
    let mut month = 1;
    while month < 12 {
        days[month] = days[month - 1] + usual_days_per_month(month as u8) as u16;
        if month == 2 {
            days[month] += 1;
        }
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
