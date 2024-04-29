/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{EpochError, ParsingError};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Year,
    YearShort,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Subsecond,
    OffsetHours,
    OffsetMinutes,
    Timescale,
    DayOfYearInteger,
    DayOfYear,
    Weekday,
    WeekdayShort,
    WeekdayDecimal,
    MonthName,
    MonthNameShort,
}

impl Default for Token {
    fn default() -> Self {
        Self::Year
    }
}

impl Token {
    // Check that the _integer_ value is valid at first sight.
    pub fn value_ok(&self, val: i32) -> Result<(), EpochError> {
        match &self {
            Self::Year => Ok(()),      // No validation
            Self::YearShort => Ok(()), // No validation
            Self::Month => {
                if !(0..=13).contains(&val) {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid month",
                    })
                } else {
                    Ok(())
                }
            }
            Self::Day => {
                if !(0..=31).contains(&val) {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid day",
                    })
                } else {
                    Ok(())
                }
            }
            Self::Hour | Self::OffsetHours => {
                if !(0..=23).contains(&val) {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid hour",
                    })
                } else {
                    Ok(())
                }
            }
            Self::Minute | Self::OffsetMinutes => {
                if !(0..=59).contains(&val) {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid minutes",
                    })
                } else {
                    Ok(())
                }
            }
            Self::Second => {
                if !(0..=60).contains(&val) {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid seconds",
                    })
                } else {
                    Ok(())
                }
            }
            Self::Subsecond => {
                if val < 0 {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid subseconds",
                    })
                } else {
                    Ok(())
                }
            }
            Self::Timescale => Ok(()),
            Self::DayOfYearInteger => {
                if !(0..=366).contains(&val) {
                    Err(EpochError::Parse {
                        source: ParsingError::ValueError,
                        details: "invalid day of year",
                    })
                } else {
                    Ok(())
                }
            }
            Self::WeekdayDecimal => {
                Ok(()) // We modulo it anyway
            }
            Self::Weekday
            | Self::WeekdayShort
            | Self::MonthName
            | Self::MonthNameShort
            | Self::DayOfYear => {
                // These cannot be parsed as integers
                Err(EpochError::Parse {
                    source: ParsingError::ValueError,
                    details: "invalid name or day of year",
                })
            }
        }
    }

    /// Returns the position in the array for a Gregorian date for this token
    pub(crate) fn gregorian_position(&self) -> Option<usize> {
        match &self {
            Token::Year | Token::YearShort => Some(0),
            Token::Month => Some(1),
            Token::Day => Some(2),
            Token::Hour => Some(3),
            Token::Minute => Some(4),
            Token::Second => Some(5),
            Token::Subsecond => Some(6),
            Token::OffsetHours => Some(7),
            Token::OffsetMinutes => Some(8),
            _ => None,
        }
    }

    /// Updates the token to what it should be seeking next given the delimiting character
    /// and returns the position in the array where the parsed integer should live
    pub fn advance_with(&mut self, ending_char: char) -> Result<(), EpochError> {
        match &self {
            Token::Year | Token::YearShort => {
                if ending_char == '-' {
                    *self = Token::Month;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid year",
                    })
                }
            }
            Token::Month => {
                if ending_char == '-' {
                    *self = Token::Day;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid month",
                    })
                }
            }
            Token::Day => {
                if ending_char == 'T' || ending_char == ' ' {
                    *self = Token::Hour;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid day",
                    })
                }
            }
            Token::Hour => {
                if ending_char == ':' {
                    *self = Token::Minute;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid hour",
                    })
                }
            }
            Token::Minute => {
                if ending_char == ':' {
                    *self = Token::Second;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid minutes",
                    })
                }
            }
            Token::Second => {
                if ending_char == '.' {
                    *self = Token::Subsecond;
                } else if ending_char == ' ' || ending_char == 'Z' {
                    // There are no subseconds here, only room for a time scale
                    *self = Token::Timescale;
                } else if ending_char == '-' || ending_char == '+' {
                    // There are no subseconds here, but we're seeing the start of an offset
                    *self = Token::OffsetHours;
                } else {
                    return Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid seconds",
                    });
                }
                Ok(())
            }
            Token::Subsecond => {
                if ending_char == ' ' || ending_char == 'Z' {
                    // There are no subseconds here, only room for a time scale
                    *self = Token::Timescale;
                } else if ending_char == '-' || ending_char == '+' {
                    // There are no subseconds here, but we're seeing the start of an offset
                    *self = Token::OffsetHours;
                } else {
                    return Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid subseconds",
                    });
                }
                Ok(())
            }
            Token::OffsetHours => {
                if ending_char == ':' {
                    *self = Token::OffsetMinutes;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid hours offset",
                    })
                }
            }
            Token::OffsetMinutes => {
                if ending_char == ' ' || ending_char == 'Z' {
                    // Only room for a time scale
                    *self = Token::Timescale;
                    Ok(())
                } else {
                    Err(EpochError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "invalid minutes offset",
                    })
                }
            }
            _ => Ok(()),
        }
    }

    pub(crate) const fn is_numeric(self) -> bool {
        !matches!(
            self,
            Token::Timescale
                | Token::Weekday
                | Token::WeekdayShort
                | Token::MonthName
                | Token::MonthNameShort
        )
    }
}
