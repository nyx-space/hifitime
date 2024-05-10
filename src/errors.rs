/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::num::ParseIntError;
use snafu::prelude::*;

#[cfg(feature = "std")]
use std::io::ErrorKind as IOError;

use lexical_core::Error as LexicalError;

#[cfg(feature = "ut1")]
use reqwest::StatusCode;

use crate::Weekday;

/// Errors handles all oddities which may occur in this library.
#[non_exhaustive]
#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum EpochError {
    InvalidGregorianDate,
    #[snafu(display("{source}, {details}"))]
    Parse {
        source: ParsingError,
        details: &'static str,
    },
    #[snafu(display("epoch initialization from system time failed"))]
    SystemTimeError,
    #[snafu(display("epoch computation failed because {source}"))]
    Duration {
        source: DurationError,
    },
}

#[non_exhaustive]
#[derive(Debug, Snafu, PartialEq)]
pub enum DurationError {
    Overflow,
    Underflow,
}

#[non_exhaustive]
#[derive(Debug, Snafu, PartialEq)]
pub enum ParsingError {
    ParseIntError {
        err: ParseIntError,
    },
    NothingToParse,
    ValueError,
    TimeSystem,
    ISO8601,
    Lexical {
        err: LexicalError,
    },
    UnknownFormat,
    UnknownOrMissingUnit,
    UnsupportedTimeSystem,
    UnknownWeekday,
    UnknownMonthName,
    UnknownToken {
        token: char,
    },
    UnexpectedCharacter {
        found: char,
        option1: Option<char>,
        option2: Option<char>,
    },
    WeekdayMismatch {
        found: Weekday,
        expected: Weekday,
    },
    InvalidTimezone,
    #[cfg(feature = "std")]
    InOut {
        err: IOError,
    },
    #[cfg(feature = "ut1")]
    DownloadError {
        code: StatusCode,
    },
}

#[cfg(test)]
mod tests {
    use crate::{EpochError, ParsingError, TimeScale};

    #[test]
    fn enum_eq() {
        // Check the equality compiles (if one compiles, then all asserts will work)
        assert!(EpochError::InvalidGregorianDate == EpochError::InvalidGregorianDate);
        assert!(ParsingError::ISO8601 == ParsingError::ISO8601);
        assert!(TimeScale::ET == TimeScale::ET);
    }
}
