/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use super::epoch_formatter::FormatItem;
use crate::{parser::Token, ParsingErrors};
use core::fmt;
use core::str::FromStr;

/// EpochFormat allows the user to format an Epoch with some custom arrangement of the Epoch items.
/// This provides almost all of the options from the 1989 C standard.
///
/// You can construct a format with `EpochFormat::from_str` (no allocation needed) where the string
/// contains a succession of tokens (each starting with `%`) and up to two separators between tokens.
///
/// Then you can provide this format to the `EpochFormatter` for formatting. This is also no-std.
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
/// | `%m` | Month number, zero-padded to 2 digits | `03` for March | N/A |
/// | `%b` | Month name in short form | `Mar` for March | N/A |
/// | `%B` | Month name in long form | `March` | N/A |
/// | `%d` | Day number, zero-padded to 2 digits | `07` for the 7th day of the month | N/A |
/// | `%j` | Day of year, zero-padded to 3 digits | `059` for 29 February 2000 | N/A |
/// | `%A` | Weekday name in long form | `Monday` | N/A |
/// | `%a` | Weekday name in short form | `Mon` for Monday | N/A |
/// | `%H` | Minute number, zero-padded to 2 digits | `39` for the 39th minutes of the hour | N/A |
/// | `%S` | Seconds, zero-padded to 2 digits | `27` for the 27th second of the minute | N/A |
/// | `%f` | Sub-seconds, zero-padded to 9 digits | `000000007` for the 7th nanosecond past the second | (2) |
/// | `%w` | Weekday in decimal form with C89 standard | `01` for Dynamical barycentric time | (3) |
/// | `%z` | Offset timezone if the formatter is provided with an epoch. | `+15:00` For GMT +15 hours and zero minutes | N/A |
///
/// * (1): Hifitime supports years from -30868 to 34668. If your epoch is larger than +/- 9999 years, this will return
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
/// use core::str::FromStr;
///
/// let bday = Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 37);
///
/// let fmt = EpochFormat::from_str("%Y-%m-%d").unwrap();
/// assert_eq!(fmt, EpochFormat::ISODATE);
///
/// let fmtd_bday = EpochFormatter::new(bday, fmt);
/// assert_eq!(format!("{fmtd_bday}"), format!("2000-02-29"));
///
/// let fmt = EpochFormat::from_str("%Y-%m-%dT%H:%M:%S.%f %T").unwrap();
/// assert_eq!(fmt, EpochFormat::ISO);
///
/// let fmtd_bday = EpochFormatter::new(bday, fmt);
/// // ISO with the timescale is the default format
/// assert_eq!(format!("{fmtd_bday}"), format!("{bday}"));
///
/// let fmt = EpochFormat::from_str("%Y-%j").unwrap();
/// assert_eq!(fmt, EpochFormat::ISO_ORDINAL);
///
/// let fmt_iso_ord = EpochFormatter::new(bday, EpochFormat::ISO_ORDINAL);
/// assert_eq!(format!("{fmt_iso_ord}"), "2000-059");
///
/// let fmt = EpochFormat::from_str("%A, %d %B %Y %H:%M:%S").unwrap();
/// assert_eq!(fmt, EpochFormat::RFC2822_LONG);
///
/// let fmt = EpochFormatter::new(bday, EpochFormat::RFC2822_LONG);
/// assert_eq!(
///     format!("{fmt}"),
///     format!("Tuesday, 29 February 2000 14:57:29")
/// );
///
/// let fmt = EpochFormat::from_str("%a, %d %b %Y %H:%M:%S").unwrap();
/// assert_eq!(fmt, EpochFormat::RFC2822);
///
/// let fmt = EpochFormatter::new(bday, EpochFormat::RFC2822);
/// assert_eq!(format!("{fmt}"), format!("Tue, 29 Feb 2000 14:57:29"));
/// ```
#[derive(Default, PartialEq)]
pub struct EpochFormat {
    pub(crate) items: [Option<FormatItem>; 16],
    pub(crate) num_items: usize,
}

impl EpochFormat {
    pub(crate) fn need_gregorian(&self) -> bool {
        for item in self.items.iter().take(self.num_items) {
            match item.as_ref().unwrap().token {
                Token::Year
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
                | Token::WeekdayDecimal => {}
            }
        }
        false
    }

    pub const ISO: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Month,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: Some('T'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Hour,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Minute,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Second,
                sep_char: Some('.'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Subsecond,
                sep_char: Some(' '),
                optional: false,
                second_sep_char: None,
            }),
            Some(FormatItem {
                token: Token::Timescale,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 8,
    };

    /// The ISO8601 format unless the subseconds are zero, then they are not printed. The time scale is also only printed if it is different from UTC.
    pub const ISO_FLEX: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Month,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: Some('T'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Hour,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Minute,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Second,
                sep_char: Some('.'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Subsecond,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: true,
            }),
            Some(FormatItem {
                token: Token::Timescale,
                sep_char: None,
                second_sep_char: None,
                optional: true,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 8,
    };

    pub const RFC3339: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Month,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: Some('T'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Hour,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Minute,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Second,
                sep_char: Some('.'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Subsecond,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::OffsetHours,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 8,
    };

    /// The RFC3339 format unless the subseconds are zero, then they are not printed.
    pub const RFC3339_FLEX: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Month,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: Some('T'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Hour,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Minute,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Second,
                sep_char: Some('.'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Subsecond,
                sep_char: None,
                second_sep_char: None,
                optional: true,
            }),
            Some(FormatItem {
                token: Token::OffsetHours,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 8,
    };

    pub const ISODATE: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Month,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 3,
    };

    pub const ISO_ORDINAL: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some('-'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::DayOfYearInteger,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 2,
    };

    pub const RFC2822: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::WeekdayShort,
                sep_char: Some(','),
                second_sep_char: Some(' '),
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::MonthNameShort,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Hour,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Minute,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Second,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 7,
    };

    pub const RFC2822_LONG: EpochFormat = EpochFormat {
        items: [
            Some(FormatItem {
                token: Token::Weekday,
                sep_char: Some(','),
                second_sep_char: Some(' '),
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Day,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::MonthName,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Year,
                sep_char: Some(' '),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Hour,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Minute,
                sep_char: Some(':'),
                second_sep_char: None,
                optional: false,
            }),
            Some(FormatItem {
                token: Token::Second,
                sep_char: None,
                second_sep_char: None,
                optional: false,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        num_items: 7,
    };
}

impl fmt::Debug for EpochFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EpochFormat:`")?;
        for maybe_item in self.items.iter().take(self.num_items) {
            let item = maybe_item.as_ref().unwrap();
            write!(f, "{:?}", item.token)?;
            if let Some(char) = item.sep_char {
                write!(f, "{}", char)?;
            }
            if let Some(char) = item.second_sep_char {
                write!(f, "{}", char)?;
            }
            if item.optional {
                write!(f, "?")?;
            }
        }
        write!(f, "`")?;
        Ok(())
    }
}

impl FromStr for EpochFormat {
    type Err = ParsingErrors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut me = EpochFormat::default();
        for token in s.split('%') {
            match token.chars().next() {
                Some(char) => match char {
                    'Y' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Year,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'm' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Month,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'b' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::MonthNameShort,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'B' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::MonthName,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'd' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Day,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'j' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::DayOfYearInteger,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'J' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::DayOfYear,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'A' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Weekday,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'a' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::WeekdayShort,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'H' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Hour,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'M' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Minute,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'S' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Second,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'f' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Subsecond,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'T' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::Timescale,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'w' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::WeekdayDecimal,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    'z' => {
                        me.items[me.num_items] = Some(FormatItem::new(
                            Token::OffsetHours,
                            token.chars().nth(1),
                            token.chars().nth(2),
                        ));
                        me.num_items += 1;
                    }
                    _ => return Err(ParsingErrors::UnknownFormattingToken(char)),
                },
                None => continue, // We're probably just at the start of the string
            }
        }

        Ok(me)
    }
}

#[test]
fn epoch_format_from_str() {
    let fmt = EpochFormat::from_str("%Y-%m-%d").unwrap();
    assert_eq!(fmt, EpochFormat::ISODATE);

    let fmt = EpochFormat::from_str("%Y-%m-%dT%H:%M:%S.%f %T").unwrap();
    assert_eq!(fmt, EpochFormat::ISO);

    let fmt = EpochFormat::from_str("%Y-%m-%dT%H:%M:%S.%f? %T?").unwrap();
    assert_eq!(fmt, EpochFormat::ISO_FLEX);

    let fmt = EpochFormat::from_str("%Y-%j").unwrap();
    assert_eq!(fmt, EpochFormat::ISO_ORDINAL);

    let fmt = EpochFormat::from_str("%A, %d %B %Y %H:%M:%S").unwrap();
    assert_eq!(fmt, EpochFormat::RFC2822_LONG);

    let fmt = EpochFormat::from_str("%a, %d %b %Y %H:%M:%S").unwrap();
    assert_eq!(fmt, EpochFormat::RFC2822);
}
