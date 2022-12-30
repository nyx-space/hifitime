/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use super::format::Format;
use super::formatter::Item;
use crate::parser::Token;

pub const ISO8601: Format = Format {
    items: [
        Some(Item {
            token: Token::Year,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Month,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Day,
            sep_char: Some('T'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Hour,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Minute,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Second,
            sep_char: Some('.'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Subsecond,
            sep_char: Some(' '),
            optional: false,
            second_sep_char: None,
        }),
        Some(Item {
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
///
/// # Limitation
/// When parsing a date, the time scale is only allowed if the subseconds are set.
/// For example, `2015-02-07T11:22:33 UTC` is _invalid_ but `2015-02-07T11:22:33.0 UTC` is valid.
pub const ISO8601_FLEX: Format = Format {
    items: [
        Some(Item {
            token: Token::Year,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Month,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Day,
            sep_char: Some('T'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Hour,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Minute,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Second,
            sep_char: Some('.'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Subsecond,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: true,
        }),
        Some(Item {
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

pub const RFC3339: Format = Format {
    items: [
        Some(Item {
            token: Token::Year,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Month,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Day,
            sep_char: Some('T'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Hour,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Minute,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Second,
            sep_char: Some('.'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Subsecond,
            sep_char: None,
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
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
pub const RFC3339_FLEX: Format = Format {
    items: [
        Some(Item {
            token: Token::Year,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Month,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Day,
            sep_char: Some('T'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Hour,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Minute,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Second,
            sep_char: Some('.'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Subsecond,
            sep_char: None,
            second_sep_char: None,
            optional: true,
        }),
        Some(Item {
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

pub const ISO8601_DATE: Format = Format {
    items: [
        Some(Item {
            token: Token::Year,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Month,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
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

pub const ISO8601_ORDINAL: Format = Format {
    items: [
        Some(Item {
            token: Token::Year,
            sep_char: Some('-'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
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

pub const RFC2822: Format = Format {
    items: [
        Some(Item {
            token: Token::WeekdayShort,
            sep_char: Some(','),
            second_sep_char: Some(' '),
            optional: false,
        }),
        Some(Item {
            token: Token::Day,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::MonthNameShort,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Year,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Hour,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Minute,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
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

/// RFC 2822 date time format
///
/// # Parsing limitation
///
/// When parsing, if the month is provided in short (but valid) form, then the parsing will still succeed. For example, if the month is `Feb` instead of `February`, then the parsing will still succeed.
pub const RFC2822_LONG: Format = Format {
    items: [
        Some(Item {
            token: Token::Weekday,
            sep_char: Some(','),
            second_sep_char: Some(' '),
            optional: false,
        }),
        Some(Item {
            token: Token::Day,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::MonthName,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Year,
            sep_char: Some(' '),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Hour,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
            token: Token::Minute,
            sep_char: Some(':'),
            second_sep_char: None,
            optional: false,
        }),
        Some(Item {
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
