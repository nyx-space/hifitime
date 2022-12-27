/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use super::epoch_format::EpochFormat;
use super::epoch_formatter::FormatItem;
use crate::parser::Token;

pub const ISO8601: EpochFormat = EpochFormat {
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
pub const ISO8601_FLEX: EpochFormat = EpochFormat {
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

pub const ISO8601_DATE: EpochFormat = EpochFormat {
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

pub const ISO8601_ORDINAL: EpochFormat = EpochFormat {
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
