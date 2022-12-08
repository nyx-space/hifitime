/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;

use crate::{parser::Token, Duration, Epoch, TimeScale};

use super::epoch_format::EpochFormat;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub(crate) struct FormatItem {
    pub(crate) token: Token,
    pub(crate) sep_char: Option<char>,
    pub(crate) second_sep_char: Option<char>,
    /// If set to true, then only a non-zero value is printed.
    pub(crate) optional: bool,
}

impl FormatItem {
    pub(crate) fn new(token: Token, sep_char: Option<char>, second_sep_char: Option<char>) -> Self {
        let mut me = Self {
            token,
            sep_char,
            second_sep_char,
            optional: false,
        };

        // Maybe the user provided the question mark first
        if let Some(char) = me.sep_char {
            if char == '?' {
                me.sep_char = None;
                me.optional = true;
            }
        }

        if let Some(char) = me.second_sep_char {
            if char == '?' {
                me.second_sep_char = None;
                me.optional = true;
            }
        }

        // Finally rearrange if needed.
        if me.sep_char.is_none() && me.second_sep_char.is_some() {
            core::mem::swap(&mut me.sep_char, &mut me.second_sep_char);
        }

        me
    }
}

pub struct EpochFormatter {
    epoch: Epoch,
    offset: Duration,
    format: EpochFormat,
}

impl EpochFormatter {
    pub fn new(epoch: Epoch, format: EpochFormat) -> Self {
        Self {
            epoch,
            offset: Duration::ZERO,
            format,
        }
    }

    pub fn with_timezone(epoch: Epoch, offset: Duration, format: EpochFormat) -> Self {
        Self {
            epoch: epoch + offset,
            offset,
            format,
        }
    }

    pub fn in_time_scale(epoch: Epoch, format: EpochFormat, time_scale: TimeScale) -> Self {
        Self::new(epoch.in_time_scale(time_scale), format)
    }

    pub fn set_timezone(&mut self, offset: Duration) {
        self.offset = offset;
    }
}

impl fmt::Display for EpochFormatter {
    /// The default format of an epoch is in UTC
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.format.need_gregorian() {
            // This is a specific branch so we don't recompute the gregorian information for each token.
            let (y, mm, dd, hh, min, s, nanos) = Epoch::compute_gregorian(self.epoch.to_duration());
            // And format.
            for (i, maybe_item) in self
                .format
                .items
                .iter()
                .enumerate()
                .take(self.format.num_items)
            {
                let item = maybe_item.as_ref().unwrap();

                // We make sure to only call this as needed.
                let mut write_sep = || -> fmt::Result {
                    if i > 0 {
                        // Print the first separation character of the previous item
                        if let Some(sep) = self.format.items[i - 1].unwrap().sep_char {
                            write!(f, "{sep}")?;
                        }
                        // Print the second separation character
                        if let Some(sep) = self.format.items[i - 1].unwrap().second_sep_char {
                            write!(f, "{sep}")?;
                        }
                    }
                    Ok(())
                };

                match item.token {
                    Token::Year => {
                        write_sep()?;
                        write!(f, "{y:04}")?
                    }
                    Token::Month => {
                        write_sep()?;
                        write!(f, "{mm:02}")?
                    }
                    Token::Day => {
                        write_sep()?;
                        write!(f, "{dd:02}")?
                    }
                    Token::Hour => {
                        write_sep()?;
                        write!(f, "{hh:02}")?
                    }
                    Token::Minute => {
                        write_sep()?;
                        write!(f, "{min:02}")?
                    }
                    Token::Second => {
                        write_sep()?;
                        write!(f, "{s:02}")?
                    }
                    Token::Subsecond => {
                        if !item.optional || nanos > 0 {
                            write_sep()?;
                            write!(f, "{nanos:09}")?
                        }
                    }
                    Token::OffsetHours => {
                        write_sep()?;
                        let (sign, days, mut hours, minutes, seconds, _, _, _) =
                            self.offset.decompose();

                        if days > 0 {
                            hours += 24 * days;
                        }

                        write!(
                            f,
                            "{}{:02}:{:02}",
                            if sign >= 0 { '+' } else { '-' },
                            hours,
                            minutes
                        )?;

                        if seconds > 0 {
                            write!(f, "{:02}", seconds)?;
                        }
                    }
                    Token::OffsetMinutes => {
                        // To print the offset, someone should use OffsetHours, so return an error here.
                        return Err(fmt::Error);
                    }
                    Token::Timescale => {
                        if !item.optional || self.epoch.time_scale != TimeScale::UTC {
                            write_sep()?;
                            write!(f, "{}", self.epoch.time_scale)?;
                        }
                    }
                    Token::DayOfYear => {
                        write_sep()?;
                        write!(f, "{:03}", self.epoch.day_of_year().floor() as u16)?
                    }
                    Token::Weekday => {
                        write_sep()?;
                        write!(f, "{}", self.epoch.weekday())?
                    }
                    Token::WeekdayShort => {
                        write_sep()?;
                        write!(f, "{:x}", self.epoch.weekday())?
                    }
                    Token::WeekdayDecimal => {
                        write_sep()?;
                        write!(f, "{}", self.epoch.weekday().to_c89_weekday())?
                    }
                    Token::MonthName => {
                        write_sep()?;
                        write!(f, "{}", self.epoch.month_name())?
                    }
                    Token::MonthNameShort => {
                        write_sep()?;
                        write!(f, "{:x}", self.epoch.month_name())?
                    }
                };
            }
        } else {
            for maybe_item in self.format.items.iter().take(self.format.num_items) {
                let item = maybe_item.as_ref().unwrap();
                match item.token {
                    Token::OffsetHours => todo!(),
                    Token::OffsetMinutes => todo!(),
                    Token::Timescale => write!(f, "{}", self.epoch.time_scale)?,
                    Token::DayOfYear => write!(f, "{}", self.epoch.day_of_year())?,
                    _ => unreachable!(),
                };

                if let Some(sep) = item.sep_char {
                    write!(f, "{sep}")?;
                } else if let Some(sep) = item.second_sep_char {
                    write!(f, "{sep}")?;
                }
            }
        }
        Ok(())
    }
}
