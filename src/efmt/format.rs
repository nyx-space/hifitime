/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use snafu::ResultExt;

use super::formatter::Item;
use crate::errors::ParseSnafu;
use crate::{parser::Token, ParsingError};
use crate::{Epoch, HifitimeError, MonthName, TimeScale, Unit, Weekday};
use core::fmt;
use core::str::FromStr;

// Maximum number of tokens in a format.
const MAX_TOKENS: usize = 16;

/// Format allows formatting an Epoch with some custom arrangement of the Epoch items.
/// This provides almost all of the options from the 1989 C standard.
///
/// Construct a format with `Format::from_str` (no allocation needed) where the string
/// contains a succession of tokens (each starting with `%`) and up to two separators between tokens.
///
/// Then this format can then be provided to the `Formatter` for formatting. This is also no-std.
///
/// # Supported tokens
///
/// Any token may be followed by `?` to make it optional. For integer values, an optional token will only be printed
/// if its value is non-zero. For time scales, only non-UTC time scale will be printed.
///
/// ## C89 standard tokens
///
/// | Token | Explanation | Example | Notes
/// | :-- | :-- | :-- | :-- |
/// | `%Y` | Proleptic Gregorian year, zero-padded to 4 digits | `2022` | (1) |
/// | `%y` | Proleptic Gregorian year after 2000, on two digits | `23` | N/A |
/// | `%m` | Month number, zero-padded to 2 digits | `03` for March | N/A |
/// | `%b` | Month name in short form | `Mar` for March | N/A |
/// | `%B` | Month name in long form | `March` | N/A |
/// | `%d` | Day number, zero-padded to 2 digits | `07` for the 7th day of the month | N/A |
/// | `%j` | Day of year, zero-padded to 3 digits | `059` for 29 February 2000 | N/A |
/// | `%A` | Weekday name in long form | `Monday` | N/A |
/// | `%a` | Weekday name in short form | `Mon` for Monday | N/A |
/// | `%H` | Hour number, zero-padded to 2 digits | `02` for the 2nd hour of the day | N/A |
/// | `%M` | Minute number, zero-padded to 2 digits | `39` for the 39th minutes of the hour | N/A |
/// | `%S` | Seconds, zero-padded to 2 digits | `27` for the 27th second of the minute | N/A |
/// | `%f` | Sub-seconds, zero-padded to 9 digits | `000000007` for the 7th nanosecond past the second | (2) |
/// | `%w` | Weekday in decimal form with C89 standard | `01` for Dynamical barycentric time | (3) |
/// | `%z` | Offset timezone if the formatter is provided with an epoch. | `+15:00` For GMT +15 hours and zero minutes | N/A |
///
/// * (1): Hifitime supports years from -34668 to 34668. If your epoch is larger than +/- 9999 years, the formatting of the years _will_ show all five digits of the year.
/// * (2): Hifitime supports exactly nanosecond precision, and this is not lost when formatting.
///
/// ## Hifitime specific tokens
///
/// These are chosen to not conflict with strptime/strfime of the C89 standard.
/// | Token | Explanation | Example | Notes
/// | :-- | :-- | :-- | :-- |
/// | `%T` | Time scale used to represent this date | `TDB` for Dynamical barycentric time | (3) |
/// | `%J` | Full day of year as a double | `59.62325231481524` for 29 February 2000 14:57:29 UTC | N/A |
///
/// * (3): Hifitime supports many time scales and these should not be lost when formatting. **This is a novelty compared to other time management libraries** as most do not have any concept of time scales.
///
///
/// # Example
/// ```
/// use hifitime::prelude::*;
/// use hifitime::efmt;
/// use hifitime::efmt::consts;
/// use core::str::FromStr;
///
/// let bday = Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 37);
///
/// let fmt = efmt::Format::from_str("%Y-%m-%d").unwrap();
/// assert_eq!(fmt, consts::ISO8601_DATE);
///
/// let fmtd_bday = Formatter::new(bday, fmt);
/// assert_eq!(format!("{fmtd_bday}"), format!("2000-02-29"));
///
/// let fmt = Format::from_str("%Y-%m-%dT%H:%M:%S.%f %T").unwrap();
/// assert_eq!(fmt, consts::ISO8601);
///
/// let fmtd_bday = Formatter::new(bday, fmt);
/// // ISO with the timescale is the default format
/// assert_eq!(format!("{fmtd_bday}"), format!("{bday}"));
///
/// let fmt = Format::from_str("%Y-%j").unwrap();
/// assert_eq!(fmt, consts::ISO8601_ORDINAL);
///
/// let fmt_iso_ord = Formatter::new(bday, consts::ISO8601_ORDINAL);
/// assert_eq!(format!("{fmt_iso_ord}"), "2000-060");
///
/// let fmt = Format::from_str("%A, %d %B %Y %H:%M:%S").unwrap();
/// assert_eq!(fmt, consts::RFC2822_LONG);
///
/// let fmt = Formatter::new(bday, consts::RFC2822_LONG);
/// assert_eq!(
///     format!("{fmt}"),
///     format!("Tuesday, 29 February 2000 14:57:29")
/// );
///
/// let fmt = Format::from_str("%a, %d %b %Y %H:%M:%S").unwrap();
/// assert_eq!(fmt, consts::RFC2822);
///
/// let fmt = Formatter::new(bday, consts::RFC2822);
/// assert_eq!(format!("{fmt}"), format!("Tue, 29 Feb 2000 14:57:29"));
/// ```
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Copy, Clone, Default, PartialEq)]
pub struct Format {
    pub(crate) items: [Option<Item>; MAX_TOKENS],
    pub(crate) num_items: usize,
}

impl Format {
    pub(crate) fn need_gregorian(&self) -> bool {
        for item in self.items.iter().take(self.num_items) {
            match item.as_ref().unwrap().token {
                Token::Year
                | Token::YearShort
                | Token::Month
                | Token::MonthName
                | Token::MonthNameShort
                | Token::Day
                | Token::Hour
                | Token::Minute
                | Token::Second
                | Token::Subsecond
                | Token::OffsetHours
                | Token::OffsetMinutes => return true,
                Token::Timescale
                | Token::DayOfYearInteger
                | Token::DayOfYear
                | Token::Weekday
                | Token::WeekdayShort
                | Token::WeekdayDecimal => {
                    // These tokens don't need the gregorian, but other tokens in the list of tokens might.
                    // Hence, we don't return anything here and continue the loop.
                }
            }
        }
        false
    }

    pub fn parse(&self, s_in: &str) -> Result<Epoch, HifitimeError> {
        // All of the integers in a date: year, month, day, hour, minute, second, subsecond, offset hours, offset minutes
        let mut decomposed = [0_i32; MAX_TOKENS];
        // The parsed time scale, defaults to UTC
        let mut ts = TimeScale::UTC;
        // The offset sign, defaults to positive.
        let mut offset_sign = 1;
        let mut day_of_year: Option<f64> = None;
        let mut weekday: Option<Weekday> = None;

        // Previous index of interest in the string
        let mut prev_idx = 0;
        let mut cur_item_idx = 0;
        let mut cur_item = match self.items[cur_item_idx] {
            Some(item) => item,
            None => {
                return Err(HifitimeError::Parse {
                    source: ParsingError::NothingToParse,
                    details: "format string contains no tokens",
                })
            }
        };
        let mut cur_token = cur_item.token;
        let mut prev_item = cur_item;
        let mut prev_token;

        let s = s_in.trim();

        for (idx, char) in s.char_indices() {
            // We should parse if:
            // 1. we're at the end of the string
            // 2. Or we've hit a non-numeric char and the token is fully numeric
            // 3. Or, token is not numeric (e.g. month name) and the current char is the separator
            // 4. And, if the length of the current substring is longer than 1 and the char is not the optional separator of the previous token.
            if idx == s.len() - 1
                || ((cur_token.is_numeric() && !char.is_numeric())
                    || (!cur_token.is_numeric() && (cur_item.sep_char_is(char))))
            {
                // If we've found the second separator of the previous token, let's simply increment the start index of the next substring.
                if idx == prev_idx
                    && (prev_item.second_sep_char.is_none() || prev_item.second_sep_char_is(char))
                {
                    prev_idx += 1;
                    continue;
                }

                if cur_token == Token::Timescale {
                    // Then we match the timescale directly.
                    if idx != s.len() - 1 {
                        // We have some remaining characters, so let's parse those in the only formats we know.
                        ts = TimeScale::from_str(s[idx..].trim()).with_context(|_| ParseSnafu {
                            details: "when parsing from format string",
                        })?;
                    }
                    break;
                } else if char == 'Z' {
                    // This is a single character to represent UTC
                    // UTC is the default time scale, so we don't need to do anything.
                    break;
                }
                prev_item = cur_item;
                prev_token = cur_token;

                let end_idx = if idx != s.len() - 1 || !char.is_numeric() {
                    // Only advance the token if we aren't at the end of the string
                    if cur_item.sep_char_is_not(char)
                        && (cur_item.second_sep_char.is_none()
                            || (cur_item.second_sep_char_is_not(char)))
                    {
                        return Err(HifitimeError::Parse {
                            source: ParsingError::UnexpectedCharacter {
                                found: char,
                                option1: cur_item.sep_char,
                                option2: cur_item.second_sep_char,
                            },
                            details: "when parsing from format string",
                        });
                    }

                    // Advance the token, unless we're at the end of the tokens.
                    if cur_item_idx == self.num_items {
                        break;
                    }
                    cur_item_idx += 1;
                    match self.items[cur_item_idx] {
                        Some(item) => {
                            cur_item = item;
                            cur_token = cur_item.token;
                        }
                        None => break,
                    }

                    idx
                } else {
                    idx + 1
                };

                let sub_str = &s[prev_idx..end_idx];

                match prev_token {
                    Token::YearShort => {
                        decomposed[0] =
                            sub_str.parse::<i32>().map_err(|_| HifitimeError::Parse {
                                source: ParsingError::ValueError,
                                details: "could not parse year as i32",
                            })? + 2000;
                    }
                    Token::DayOfYear => {
                        // We must parse this as a floating point value.
                        match lexical_core::parse(sub_str.as_bytes()) {
                            Ok(val) => day_of_year = Some(val),
                            Err(_) => {
                                return Err(HifitimeError::Parse {
                                    source: ParsingError::ValueError,
                                    details: "could not parse day of year as f64",
                                })
                            }
                        }
                    }
                    Token::Weekday | Token::WeekdayShort => {
                        // Set the weekday
                        match Weekday::from_str(sub_str) {
                            Ok(day) => weekday = Some(day),
                            Err(source) => {
                                return Err(HifitimeError::Parse {
                                    source,
                                    details: "could not parse weekday",
                                })
                            }
                        }
                    }
                    Token::WeekdayDecimal => {
                        todo!()
                    }
                    Token::MonthName | Token::MonthNameShort => {
                        match MonthName::from_str(sub_str) {
                            Ok(month) => {
                                decomposed[1] = ((month as u8) + 1) as i32;
                            }
                            Err(_) => {
                                return Err(HifitimeError::Parse {
                                    source: ParsingError::ValueError,
                                    details: "could not parse month name",
                                })
                            }
                        }
                    }
                    _ => {
                        match lexical_core::parse(sub_str.as_bytes()) {
                            Ok(val) => {
                                // Check that this valid is OK for the token we're reading it as.
                                prev_token.value_ok(val)?;
                                match prev_token.gregorian_position() {
                                    Some(pos) => {
                                        // If these are the subseconds, we must convert them to nanoseconds
                                        if prev_token == Token::Subsecond {
                                            if end_idx - prev_idx != 9 {
                                                decomposed[pos] = val
                                                    * 10_i32.pow((9 - (end_idx - prev_idx)) as u32);
                                            } else {
                                                decomposed[pos] = val;
                                            }
                                        } else {
                                            decomposed[pos] = val
                                        }
                                    }
                                    None => match prev_token {
                                        Token::DayOfYearInteger => day_of_year = Some(val as f64),
                                        Token::Weekday => todo!(),
                                        Token::WeekdayShort => todo!(),
                                        Token::WeekdayDecimal => todo!(),
                                        Token::MonthName => todo!(),
                                        Token::MonthNameShort => todo!(),
                                        _ => unreachable!(),
                                    },
                                }
                            }
                            Err(err) => {
                                return Err(HifitimeError::Parse {
                                    source: ParsingError::Lexical { err },
                                    details: "could not parse numerical",
                                });
                            }
                        }
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

        let epoch = match day_of_year {
            Some(days) => {
                // Parse the elapsed time in the given day
                let elapsed = (decomposed[3] as i64) * Unit::Hour
                    + (decomposed[4] as i64) * Unit::Minute
                    + (decomposed[5] as i64) * Unit::Second
                    + (decomposed[6] as i64) * Unit::Nanosecond;
                Epoch::from_day_of_year(decomposed[0], days, ts) + elapsed
            }
            None => Epoch::maybe_from_gregorian(
                decomposed[0],
                decomposed[1].try_into().unwrap(),
                decomposed[2].try_into().unwrap(),
                decomposed[3].try_into().unwrap(),
                decomposed[4].try_into().unwrap(),
                decomposed[5].try_into().unwrap(),
                decomposed[6].try_into().unwrap(),
                ts,
            )?,
        };

        if let Some(weekday) = weekday {
            // Check that the weekday is correct
            if weekday != epoch.weekday() {
                return Err(HifitimeError::Parse {
                    source: ParsingError::WeekdayMismatch {
                        found: weekday,
                        expected: epoch.weekday(),
                    },
                    details: "weekday and day number do not match",
                });
            }
        }

        Ok(epoch + tz)
    }
}

impl fmt::Debug for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EpochFormat:`")?;
        for maybe_item in self.items.iter().take(self.num_items) {
            let item = maybe_item.as_ref().unwrap();
            write!(f, "{:?}", item.token)?;
            if let Some(char) = item.sep_char {
                write!(f, "{char}")?;
            }
            if let Some(char) = item.second_sep_char {
                write!(f, "{char}")?;
            }
            if item.optional {
                write!(f, "?")?;
            }
        }
        write!(f, "`")?;
        Ok(())
    }
}

impl FromStr for Format {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut me = Format::default();
        for token in s.split('%') {
            if me.num_items == MAX_TOKENS && token.chars().next().is_some() {
                return Err(ParsingError::UnknownFormat);
            }
            match token.chars().next() {
                Some(char) => match char {
                    'Y' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Year,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'y' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::YearShort,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'm' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Month,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'b' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::MonthNameShort,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'B' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::MonthName,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'd' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Day,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'j' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::DayOfYearInteger,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'J' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::DayOfYear,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'A' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Weekday,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'a' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::WeekdayShort,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'H' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Hour,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'M' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Minute,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'S' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Second,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'f' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Subsecond,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'T' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::Timescale,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'w' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::WeekdayDecimal,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'z' => {
                        me.items[me.num_items] = Some(Item::new(
                            Token::OffsetHours,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    _ => return Err(ParsingError::UnknownToken { token: char }),
                },
                None => continue, // We're probably just at the start of the string
            }
        }

        Ok(me)
    }
}

#[test]
fn epoch_format_from_str() {
    let fmt = Format::from_str("%Y-%m-%d").unwrap();
    assert_eq!(fmt, crate::efmt::consts::ISO8601_DATE);

    let fmt = Format::from_str("%Y-%m-%dT%H:%M:%S.%f %T").unwrap();
    assert_eq!(fmt, crate::efmt::consts::ISO8601);

    let fmt = Format::from_str("%Y-%m-%dT%H:%M:%S.%f? %T?").unwrap();
    assert_eq!(fmt, crate::efmt::consts::ISO8601_FLEX);

    let fmt = Format::from_str("%Y-%j").unwrap();
    assert_eq!(fmt, crate::efmt::consts::ISO8601_ORDINAL);

    let fmt = Format::from_str("%A, %d %B %Y %H:%M:%S").unwrap();
    assert_eq!(fmt, crate::efmt::consts::RFC2822_LONG);

    let fmt = Format::from_str("%a, %d %b %Y %H:%M:%S").unwrap();
    assert_eq!(fmt, crate::efmt::consts::RFC2822);
}

#[cfg(feature = "std")]
#[test]
fn gh_248_regression() {
    /*
    Update on 2023-12-30 to match the Python behavior:

    >>> from datetime import datetime
    >>> dt, fmt = "2023-117T12:55:26", "%Y-%jT%H:%M:%S"
    >>> datetime.strptime(dt, fmt)
    datetime.datetime(2023, 4, 27, 12, 55, 26)
     */

    let e = Epoch::from_format_str("2023-117T12:55:26", "%Y-%jT%H:%M:%S").unwrap();

    assert_eq!(format!("{e}"), "2023-04-27T12:55:26 UTC");
}
