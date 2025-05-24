/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use super::{Duration, Unit};
use crate::{HifitimeError, ParsingError};
use core::str::FromStr;

// Lookup table for units: (unit string, index in decomposed array)
const UNITS: &[(&str, usize)] = &[
    ("d", 0),
    ("days", 0),
    ("day", 0),
    ("h", 1),
    ("hours", 1),
    ("hour", 1),
    ("hr", 1),
    ("min", 2),
    ("mins", 2),
    ("minute", 2),
    ("minutes", 2),
    ("s", 3),
    ("second", 3),
    ("seconds", 3),
    ("sec", 3),
    ("ms", 4),
    ("millisecond", 4),
    ("milliseconds", 4),
    ("μs", 5),
    ("us", 5),
    ("microsecond", 5),
    ("microseconds", 5),
    ("ns", 6),
    ("nanosecond", 6),
    ("nanoseconds", 6),
];

impl FromStr for Duration {
    type Err = HifitimeError;

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
    /// assert_eq!(Duration::from_str("-5 h 256 ms 1 ns").unwrap(), -(5 * Unit::Hour + 256 * Unit::Millisecond + Unit::Nanosecond));
    /// ```
    fn from_str(s_in: &str) -> Result<Self, Self::Err> {
        let s = s_in.trim();

        if s.is_empty() {
            return Err(HifitimeError::Parse {
                source: ParsingError::NothingToParse,
                details: "input string is empty",
            });
        }

        // There is at least one character, so we can unwrap this.
        // This is maybe a timezone offset.
        let (sign, maybe_offset, skip) = if s.chars().nth(0).unwrap() == '-' {
            (-1, true, 1)
        } else if s.chars().nth(0).unwrap() == '+' {
            (1, true, 0)
        } else {
            (1, false, 0)
        };

        if maybe_offset {
            if let Ok(duration) = parse_offset(s) {
                if sign == -1 {
                    return Ok(-duration);
                } else {
                    return Ok(duration);
                }
            }
        }

        // Fall through because a negative sign could be an offset or a duration.
        let duration = parse_duration(&s[skip..])?;

        if sign == -1 {
            Ok(-duration)
        } else {
            Ok(duration)
        }
    }
}

fn cmp_chars_to_str(s: &str, start_idx: usize, cmp_str: &str) -> bool {
    let cmp_bytes = cmp_str.as_bytes();
    let s_bytes = s.as_bytes();

    if start_idx + cmp_bytes.len() > s_bytes.len() {
        return false; // Not enough bytes left in s
    }

    &s_bytes[start_idx..start_idx + cmp_bytes.len()] == cmp_bytes
}

fn parse_duration(s: &str) -> Result<Duration, HifitimeError> {
    let mut decomposed = [0.0_f64; 7];
    let mut prev_idx = 0;
    let mut seeking_number = true;
    let mut latest_value = 0.0;
    let mut prev_char_was_space = false;

    for (idx, char) in s.char_indices() {
        if char == ' ' {
            if seeking_number {
                if !prev_char_was_space {
                    if prev_idx == idx {
                        return Err(HifitimeError::Parse {
                            source: ParsingError::UnknownOrMissingUnit,
                            details: "expect a unit after a numeric",
                        });
                    }

                    match lexical_core::parse(&s.as_bytes()[prev_idx..idx]) {
                        Ok(val) => latest_value = val,
                        Err(_) => {
                            return Err(HifitimeError::Parse {
                                source: ParsingError::ValueError,
                                details: "could not parse what precedes the space",
                            });
                        }
                    }
                    seeking_number = false;
                }
            } else {
                let end_idx = if let Some((inner_idx, _)) = s[idx..].char_indices().next() {
                    idx + inner_idx
                } else {
                    idx
                };

                let start_idx = prev_idx;
                let mut found_unit = false;
                for &(unit_str, pos) in UNITS {
                    if cmp_chars_to_str(s, start_idx, unit_str) {
                        decomposed[pos] = latest_value;
                        seeking_number = true;
                        prev_idx = end_idx;
                        found_unit = true;
                        break;
                    }
                }

                if !found_unit {
                    return Err(HifitimeError::Parse {
                        source: ParsingError::UnknownOrMissingUnit,
                        details: "unknown unit",
                    });
                }
            }
            prev_char_was_space = true;
        } else {
            if prev_char_was_space {
                prev_idx = idx;
            }
            prev_char_was_space = false;
        }
    }

    // Handle the last element if the string didn't end with a space
    if !seeking_number {
        let start_idx = prev_idx;
        let mut found_unit = false;
        for &(unit_str, pos) in UNITS {
            if cmp_chars_to_str(s, start_idx, unit_str) {
                decomposed[pos] = latest_value;
                found_unit = true;
                break;
            }
        }

        if !found_unit {
            return Err(HifitimeError::Parse {
                source: ParsingError::UnknownOrMissingUnit,
                details: "unknown unit",
            });
        }
    } else if prev_idx < s.len() {
        return Err(HifitimeError::Parse {
            source: ParsingError::UnknownOrMissingUnit,
            details: "expect a unit after the last numeric",
        });
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

fn parse_offset(s: &str) -> Result<Duration, HifitimeError> {
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
        return Err(HifitimeError::Parse {
            source: ParsingError::InvalidTimezone,
            details: "invalid timezone format [+/-]HH:MM",
        });
    };

    // Fetch the hours
    let hours: i64 = match lexical_core::parse(&s.as_bytes()[indexes.0..indexes.1]) {
        Ok(val) => val,
        Err(err) => {
            return Err(HifitimeError::Parse {
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
                    return Err(HifitimeError::Parse {
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
                                return Err(HifitimeError::Parse {
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

    Ok(hours * Unit::Hour + minutes * Unit::Minute + seconds * Unit::Second)
}
