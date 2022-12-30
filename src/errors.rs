/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::convert;
use core::fmt;
use core::num::ParseIntError;

#[cfg(feature = "std")]
use std::error::Error;

#[cfg(feature = "std")]
use std::io::ErrorKind as IOError;

#[cfg(feature = "ut1")]
use reqwest::StatusCode;

use crate::Weekday;

/// Errors handles all oddities which may occur in this library.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a call to `Datetime::new` receives 60 seconds and there are only 59 seconds in the provided
    /// date time then a Carry Error is returned as the Result.
    Carry,
    /// ParseError is returned when a provided string could not be parsed and converted to the desired
    /// struct (e.g. Datetime).
    ParseError(ParsingErrors),
    /// Raised when trying to initialize an Epoch or Duration from its hi and lo values, but these overlap
    ConversionOverlapError(f64, f64),
    /// Raised if an overflow occured
    Overflow,
    /// Raised if the initialization from system time failed
    SystemTimeError,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParsingErrors {
    ParseIntError,
    ValueError,
    TimeSystem,
    ISO8601,
    UnknownFormat,
    UnknownOrMissingUnit,
    UnsupportedTimeSystem,
    UnknownWeekday,
    UnknownMonthName,
    UnknownFormattingToken(char),
    UnexpectedCharacter {
        found: char,
        option1: Option<char>,
        option2: Option<char>,
    },
    WeekdayMismatch {
        found: Weekday,
        expected: Weekday,
    },
    #[cfg(feature = "std")]
    IOError(IOError),
    #[cfg(feature = "ut1")]
    DownloadError(StatusCode),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
            Self::ParseError(kind) => write!(f, "ParseError: {:?}", kind),
            Self::ConversionOverlapError(hi, lo) => {
                write!(f, "hi and lo values overlap: {}, {}", hi, lo)
            }
            Self::Overflow => write!(
                f,
                "overflow occured when trying to convert Duration information"
            ),
            Self::SystemTimeError => write!(f, "std::time::SystemTime returned an error"),
        }
    }
}

impl convert::From<ParseIntError> for Errors {
    fn from(_: ParseIntError) -> Self {
        Errors::ParseError(ParsingErrors::ParseIntError)
    }
}

#[cfg(feature = "std")]
impl Error for Errors {}

#[cfg(test)]
mod tests {
    use crate::{Errors, ParsingErrors, TimeScale};

    #[test]
    fn enum_eq() {
        // Check the equality compiles (if one compiles, then all asserts will work)
        assert!(Errors::Carry == Errors::Carry);
        assert!(ParsingErrors::ParseIntError == ParsingErrors::ParseIntError);
        assert!(TimeScale::ET == TimeScale::ET);
    }
}
