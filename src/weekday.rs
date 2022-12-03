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
        let val = s.trim().to_lowercase();
        match val.as_str() {
            "monday" => Ok(Self::Monday),
            "tuesday" => Ok(Self::Tuesday),
            "wednesday" => Ok(Self::Wednesday),
            "thursday" => Ok(Self::Thursday),
            "friday" => Ok(Self::Friday),
            "saturday" => Ok(Self::Saturday),
            "sunday" => Ok(Self::Sunday),
            _ => Err(ParsingErrors::ParseWeekdayError),
        }
    }
}

impl Add for Weekday {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from(self as u8 + rhs as u8)
    }
}

impl Sub for Weekday {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::from(self as u8 - rhs as u8)
    }
}

impl Add<u8> for Weekday {
    type Output = Self;
    fn add(self, rhs: u8) -> Self {
        Self::from(self as u8 + rhs)
    }
}

impl Sub<u8> for Weekday {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self {
        Self::from(self as i8 - rhs as i8)
    }
}

impl AddAssign for Weekday {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Weekday {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
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
    }
}
