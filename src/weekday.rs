/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{Duration, ParsingErrors, Unit};
use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::str::FromStr;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

impl Default for Weekday {
    fn default() -> Self {
        Self::Monday
    }
}

impl Weekday {
    /// Max: last weekday <=> `Sunday`, used only for conversion to/from u8.
    const MAX: u8 = 7;
    /// Trivial, but avoid magic numbers.
    pub(crate) const DAYS_PER_WEEK: f64 = 7.0;
    /// Trivial, but avoid magic numbers.
    pub(crate) const DAYS_PER_WEEK_I128: i128 = 7;

    // C89 defines Sunday as zero (which is stupid)
    pub(crate) fn to_c89_weekday(self) -> u8 {
        let c89_weekday: u8 = (self + 1).into();
        c89_weekday
    }
}

impl From<u8> for Weekday {
    fn from(u: u8) -> Self {
        match u.rem_euclid(Self::MAX) {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            6 => Self::Sunday,
            _ => Self::default(), // Defaults back to default for other values.
        }
    }
}

impl From<i8> for Weekday {
    fn from(i: i8) -> Self {
        Self::from((i.rem_euclid(Self::MAX as i8) + Self::MAX as i8) as u8)
    }
}

impl From<Weekday> for u8 {
    fn from(week: Weekday) -> Self {
        match week {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        }
    }
}

impl FromStr for Weekday {
    type Err = ParsingErrors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "mon" | "Mon" | "MON" | "monday" | "Monday" | "MONDAY" => Ok(Self::Monday),
            "tue" | "Tue" | "TUE" | "tuesday" | "Tuesday" | "TUESDAY" => Ok(Self::Tuesday),
            "wed" | "Wed" | "WED" | "wednesday" | "Wednesday" | "WEDNESDAY" => Ok(Self::Wednesday),
            "thu" | "Thu" | "THU" | "thursday" | "Thursday" | "THURSDAY" => Ok(Self::Thursday),
            "fri" | "Fri" | "FRI" | "friday" | "Friday" | "FRIDAY" => Ok(Self::Friday),
            "sat" | "Sat" | "SAT" | "saturday" | "Saturday" | "SATURDAY" => Ok(Self::Saturday),
            "sun" | "Sun" | "SUN" | "sunday" | "Sunday" | "SUNDAY" => Ok(Self::Sunday),
            _ => Err(ParsingErrors::UnknownWeekday),
        }
    }
}

impl Add for Weekday {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from(u8::from(self) + u8::from(rhs))
    }
}

impl Sub for Weekday {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Self::Output {
        // We can safely cast the weekdays as u8 into i8 because the maximum value is 6, and the max value of a i8 is 127.
        let self_i8 = u8::from(self) as i8;
        let mut rhs_i8 = u8::from(rhs) as i8;
        if rhs_i8 - self_i8 < 0 {
            rhs_i8 += 7;
        }
        i64::from(rhs_i8 - self_i8) * Unit::Day
    }
}

impl Add<u8> for Weekday {
    type Output = Self;
    fn add(self, rhs: u8) -> Self {
        Self::from(u8::from(self) + rhs)
    }
}

impl Sub<u8> for Weekday {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self {
        // We can safely cast the weekdays as u8 into i8 because the maximum value is 6, and the max value of a i8 is 127.
        Self::from(u8::from(self) as i8 - rhs as i8)
    }
}

impl AddAssign<u8> for Weekday {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

impl SubAssign<u8> for Weekday {
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// LowerHex allows printing the week day in its shortened form in English
impl fmt::LowerHex for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Monday => write!(f, "Mon"),
            Self::Tuesday => write!(f, "Tue"),
            Self::Wednesday => write!(f, "Wed"),
            Self::Thursday => write!(f, "Thu"),
            Self::Friday => write!(f, "Fri"),
            Self::Saturday => write!(f, "Sat"),
            Self::Sunday => write!(f, "Sun"),
        }
    }
}

#[test]
fn test_wrapping() {
    assert_eq!(Weekday::default(), Weekday::Monday);
    assert_eq!(Weekday::from(Weekday::MAX), Weekday::Monday);

    let monday = Weekday::default();
    for i in 0..24 {
        // Test wrapping
        let add = monday + i;
        let expected: Weekday = i.rem_euclid(Weekday::MAX.into()).into();
        assert_eq!(
            add, expected,
            "expecting {:?} got {:?} for {:02} conversion",
            expected, add, i
        );
        // Test FromStr
    }
}

#[test]
fn test_iso_weekday() {
    assert_eq!(Weekday::Sunday.to_c89_weekday(), 0);
    assert_eq!(Weekday::Monday.to_c89_weekday(), 1);
    assert_eq!(Weekday::Tuesday.to_c89_weekday(), 2);
    assert_eq!(Weekday::Wednesday.to_c89_weekday(), 3);
    assert_eq!(Weekday::Thursday.to_c89_weekday(), 4);
    assert_eq!(Weekday::Friday.to_c89_weekday(), 5);
    assert_eq!(Weekday::Saturday.to_c89_weekday(), 6);
}
