/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::ParsingErrors;
use core::fmt;
use core::str::FromStr;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MonthName {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Default for MonthName {
    fn default() -> Self {
        Self::January
    }
}

impl MonthName {
    const MAX: u8 = 12;
}

impl FromStr for MonthName {
    type Err = ParsingErrors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "jan" | "Jan" | "JAN" | "January" | "JANUARY" | "january" => Ok(Self::January),
            "feb" | "Feb" | "FEB" | "February" | "FEBRUARY" | "february" => Ok(Self::February),
            "mar" | "Mar" | "MAR" | "March" | "MARCH" | "march" => Ok(Self::March),
            "apr" | "Apr" | "APR" | "April" | "APRIL" | "april" => Ok(Self::April),
            "may" | "May" | "MAY" => Ok(Self::May),
            "jun" | "Jun" | "JUN" | "June" | "JUNE" | "june" => Ok(Self::June),
            "jul" | "Jul" | "JUL" | "July" | "JULY" | "july" => Ok(Self::July),
            "aug" | "Aug" | "AUG" | "August" | "AUGUST" | "august" => Ok(Self::August),
            "sep" | "Sep" | "SEP" | "September" | "SEPTEMBER" | "september" => Ok(Self::September),
            "oct" | "Oct" | "OCT" | "October" | "OCTOBER" | "october" => Ok(Self::October),
            "nov" | "Nov" | "NOV" | "November" | "NOVEMBER" | "november" => Ok(Self::November),
            "dec" | "Dec" | "DEC" | "December" | "DECEMBER" | "december" => Ok(Self::December),
            _ => Err(ParsingErrors::UnknownMonthName),
        }
    }
}

impl From<u8> for MonthName {
    fn from(u: u8) -> Self {
        match u.rem_euclid(Self::MAX) {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => Self::default(), // Defaults back to default for other values.
        }
    }
}

impl fmt::Display for MonthName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// LowerHex allows printing the week day in its shortened form in English
impl fmt::LowerHex for MonthName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::January => write!(f, "Jan"),
            Self::February => write!(f, "Feb"),
            Self::March => write!(f, "Mar"),
            Self::April => write!(f, "Apr"),
            Self::May => write!(f, "May"),
            Self::June => write!(f, "Jun"),
            Self::July => write!(f, "Jul"),
            Self::August => write!(f, "Aug"),
            Self::September => write!(f, "Sep"),
            Self::October => write!(f, "Oct"),
            Self::November => write!(f, "Nov"),
            Self::December => write!(f, "Dec"),
        }
    }
}
