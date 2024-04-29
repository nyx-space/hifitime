/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

use super::{Duration, Unit};
use crate::{EpochError, ParsingError};
use core::str::FromStr;

impl FromStr for Duration {
    type Err = EpochError;

    /// Attempts to convert a simple string to a Duration. Does not yet support complicated durations.
    ///
    /// Identifiers:
    ///  + d, days, day
    ///  + h, hours, hour
    ///  + min, mins, minute
    ///  + s, second, seconds
    ///  + ms, millisecond, milliseconds
    ///  + us, microsecond, microseconds
    ///  + ns, nanosecond, nanoseconds
    ///  + `+` or `-` indicates a timezone offset
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, Unit};
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Duration::from_str("1 d").unwrap(), Unit::Day * 1);
    /// assert_eq!(Duration::from_str("10.598 days").unwrap(), Unit::Day * 10.598);
    /// assert_eq!(Duration::from_str("10.598 min").unwrap(), Unit::Minute * 10.598);
    /// assert_eq!(Duration::from_str("10.598 us").unwrap(), Unit::Microsecond * 10.598);
    /// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), Unit::Second * 10.598);
    /// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), Unit::Nanosecond * 10.598);
    /// assert_eq!(Duration::from_str("5 h 256 ms 1 ns").unwrap(), 5 * Unit::Hour + 256 * Unit::Millisecond + Unit::Nanosecond);
    /// assert_eq!(Duration::from_str("-01:15:30").unwrap(), -(1 * Unit::Hour + 15 * Unit::Minute + 30 * Unit::Second));
    /// assert_eq!(Duration::from_str("+3615").unwrap(), 36 * Unit::Hour + 15 * Unit::Minute);
    /// ```
    fn from_str(s_in: &str) -> Result<Self, Self::Err> {
        // Each part of a duration as days, hours, minutes, seconds, millisecond, microseconds, and nanoseconds
        let mut decomposed = [0.0_f64; 7];

        let mut prev_idx = 0;
        let mut seeking_number = true;
        let mut latest_value = 0.0;

        let s = s_in.trim();

        if s.is_empty() {
            return Err(EpochError::Parse {
                source: ParsingError::NothingToParse,
                details: "input string is empty",
            });
        }

        // There is at least one character, so we can unwrap this.
        if let Some(char) = s.chars().next() {
            if char == '+' || char == '-' {
                // This is a timezone offset.
                let offset_sign = if char == '-' { -1 } else { 1 };

                let indexes: (usize, usize, usize) = (1, 3, 5);
                let colon = if s.len() == 3 || s.len() == 5 || s.len() == 7 {
                    // There is a zero or even number of separators between the hours, minutes, and seconds.
                    // Only zero (or one) characters separator is supported. This will return a ValueError later if there is
                    // an even but greater than one character separator.
                    0
                } else if s.len() == 4 || s.len() == 6 || s.len() == 9 {
                    // There is an odd number of characters as a separator between the hours, minutes, and seconds.
                    // Only one character separator is supported. This will return a ValueError later if there is
                    // an odd but greater than one character separator.
                    1
                } else {
                    // This invalid
                    return Err(EpochError::Parse {
                        source: ParsingError::InvalidTimezone,
                        details: "invalid timezone format [+/-]HH:MM",
                    });
                };

                // Fetch the hours
                let hours: i64 = match lexical_core::parse(s[indexes.0..indexes.1].as_bytes()) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(EpochError::Parse {
                            source: ParsingError::Lexical { err },
                            details: "invalid hours",
                        })
                    }
                };

                let mut minutes: i64 = 0;
                let mut seconds: i64 = 0;

                match s.get(indexes.1 + colon..indexes.2 + colon) {
                    None => {
                        //Do nothing, we've reached the end of the useful data.
                    }
                    Some(subs) => {
                        // Fetch the minutes
                        match lexical_core::parse(subs.as_bytes()) {
                            Ok(val) => minutes = val,
                            Err(_) => {
                                return Err(EpochError::Parse {
                                    source: ParsingError::ValueError,
                                    details: "invalid minute",
                                })
                            }
                        }

                        match s.get(indexes.2 + 2 * colon..) {
                            None => {
                                // Do nothing, there are no seconds in this offset
                            }
                            Some(subs) => {
                                if !subs.is_empty() {
                                    // Fetch the seconds
                                    match lexical_core::parse(subs.as_bytes()) {
                                        Ok(val) => seconds = val,
                                        Err(_) => {
                                            return Err(EpochError::Parse {
                                                source: ParsingError::ValueError,
                                                details: "invalid seconds",
                                            })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Return the constructed offset
                if offset_sign == -1 {
                    return Ok(-(hours * Unit::Hour
                        + minutes * Unit::Minute
                        + seconds * Unit::Second));
                } else {
                    return Ok(hours * Unit::Hour
                        + minutes * Unit::Minute
                        + seconds * Unit::Second);
                }
            }
        };

        for (idx, char) in s.chars().enumerate() {
            if char == ' ' || idx == s.len() - 1 {
                if seeking_number {
                    if prev_idx == idx {
                        // We've reached the end of the string and it didn't end with a unit
                        return Err(EpochError::Parse {
                            source: ParsingError::UnknownOrMissingUnit,
                            details: "expect a unit after a numeric",
                        });
                    }
                    // We've found a new space so let's parse whatever precedes it
                    match lexical_core::parse(s[prev_idx..idx].as_bytes()) {
                        Ok(val) => latest_value = val,
                        Err(_) => {
                            return Err(EpochError::Parse {
                                source: ParsingError::ValueError,
                                details: "could not parse what precedes the space",
                            })
                        }
                    }
                    // We'll now seek a unit
                    seeking_number = false;
                } else {
                    // We're seeking a unit not a number, so let's parse the unit we just found and remember the position.
                    let end_idx = if idx == s.len() - 1 { idx + 1 } else { idx };
                    let pos = match &s[prev_idx..end_idx] {
                        "d" | "days" | "day" => 0,
                        "h" | "hours" | "hour" => 1,
                        "min" | "mins" | "minute" | "minutes" => 2,
                        "s" | "second" | "seconds" => 3,
                        "ms" | "millisecond" | "milliseconds" => 4,
                        "us" | "microsecond" | "microseconds" => 5,
                        "ns" | "nanosecond" | "nanoseconds" => 6,
                        _ => {
                            return Err(EpochError::Parse {
                                source: ParsingError::UnknownOrMissingUnit,
                                details: "unknown unit",
                            });
                        }
                    };
                    // Store the value
                    decomposed[pos] = latest_value;
                    // Now we switch to seeking a value
                    seeking_number = true;
                }
                prev_idx = idx + 1;
            }
        }

        Ok(Duration::compose_f64(
            1,
            decomposed[0],
            decomposed[1],
            decomposed[2],
            decomposed[3],
            decomposed[4],
            decomposed[5],
            decomposed[6],
        ))
    }
}
