/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

#[cfg(feature = "python")]
use pyo3::prelude::*;

use core::str::FromStr;

use crate::{
    Errors, ParsingErrors,
    SECONDS_PER_YEAR, SECONDS_PER_DAY,    
    SECONDS_PER_YEAR_I64, SECONDS_PER_DAY_I64,    
};

/// Enum of the different time systems available
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
pub enum TimeScale {
    /// TAI is the representation of an Epoch internally
    TAI,
    /// Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    TT,
    /// Ephemeris Time as defined by SPICE (slightly different from true TDB)
    ET,
    /// Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time)
    TDB,
    /// Universal Coordinated Time
    UTC,
    /// GPST Time also applies to QZSS, IRNSS and GAL constellations
    GPST,
    /// Galileo Time scale 
    GST,
    /// BeiDou Time scale
    BDT,
}

impl TimeScale {
    /// Maximal value when casting to unsigned integer.
    /// Increment when introducing new timescales.
    pub const MAX_U8: u8 = 7;
    
    /// GPS: 1980 Jan 5th Midnight,
    /// |TAI-UTC| = +19 on that day
    pub const SECONDS_GPS_TAI_OFFSET: f64 = 
        80.0 * SECONDS_PER_YEAR + 4.0 * SECONDS_PER_DAY + 19.0;
    pub const SECONDS_GPS_TAI_OFFSET_I64: i64 = 
        80 * SECONDS_PER_YEAR_I64 + 4 * SECONDS_PER_DAY_I64 + 19;
    pub const DAYS_GPS_TAI_OFFSET: f64 = Self::SECONDS_GPS_TAI_OFFSET / SECONDS_PER_DAY;

    /// GST(Galileo): 1999 August 21st Midnight 
    /// |TAI-UTC| = +32 on that day, cf. https://en.wikipedia.org/wiki/Leap_second
    pub const SECONDS_GST_TAI_OFFSET: f64 =
        /* August 21st midnight: +233 days */
        99.0 * SECONDS_PER_YEAR + 233.0 * SECONDS_PER_DAY + 32.0;
    pub const SECONDS_GST_TAI_OFFSET_I64: i64 =
        99 * SECONDS_PER_YEAR_I64 + 233 * SECONDS_PER_DAY_I64 + 32;
    pub const DAYS_GST_TAI_OFFSET: f64 = Self::SECONDS_GST_TAI_OFFSET / SECONDS_PER_YEAR;
    
    /// BDT(BeiDou): 2005 Dec 31st Midnight
    /// |TAI-UTC| = +33 on that day, cf. https://en.wikipedia.org/wiki/Leap_second
    pub const SECONDS_BDT_TAI_OFFSET: f64 =
        106.0 * SECONDS_PER_YEAR + 33.0;
    pub const SECONDS_BDT_TAI_OFFSET_I64: i64 =
        106 * SECONDS_PER_YEAR_I64 + 33;
    pub const DAYS_BDT_TAI_OFFSET: f64 = Self::SECONDS_BDT_TAI_OFFSET / SECONDS_PER_YEAR;

    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            Self::GPST => 4,
            Self::TAI | Self::TDB | Self::UTC | Self::GST | Self::BDT => 3,
            Self::ET | Self::TT => 2,
        }
    }

    /// Returns true if self takes Leap Seconds into account
    pub(crate) const fn uses_leap(&self) -> bool {
        match self {
            Self::UTC => true,
            _ => false,
        }
    }

    /// (Positive) offset in seconds, to apply in reference to TAI J1900
    /// for this timescale
    pub(crate) const fn tai_j1900_offset_seconds_i64(&self) -> i64 {
        match self {
            Self::GST  => Self::SECONDS_GST_TAI_OFFSET_I64,
            Self::BDT  => Self::SECONDS_BDT_TAI_OFFSET_I64,
            Self::GPST => Self::SECONDS_GPS_TAI_OFFSET_I64,
            _ => 0,
        }
    }
}

/// Allows conversion of a TimeSystem into a u8
/// Mapping: TAI: 0; TT: 1; ET: 2; TDB: 3; UTC: 4; GPST: 5; GST: 6; BDT: 7;
impl From<TimeScale> for u8 {
    fn from(ts: TimeScale) -> Self {
        match ts {
            TimeScale::TAI => 0,
            TimeScale::TT => 1,
            TimeScale::ET => 2,
            TimeScale::TDB => 3,
            TimeScale::UTC => 4,
            TimeScale::GPST => 5,
            TimeScale::GST => 6,
            TimeScale::BDT => 7,
        }
    }
}

/// Allows conversion of a u8 into a TimeSystem.
/// Mapping: 1: TT; 2: ET; 3: TDB; 4: UTC; 5: GPST; 6: GST; 7: BDT; anything else: TAI
impl From<u8> for TimeScale {
    fn from(val: u8) -> Self {
        match val {
            1 => Self::TT,
            2 => Self::ET,
            3 => Self::TDB,
            4 => Self::UTC,
            5 => Self::GPST,
            6 => Self::GST,
            7 => Self::BDT,
            _ => Self::TAI,
        }
    }
}

impl FromStr for TimeScale {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s.trim();
        if val == "UTC" {
            Ok(Self::UTC)
        } else if val == "TT" {
            Ok(Self::TT)
        } else if val == "TAI" {
            Ok(Self::TAI)
        } else if val == "TDB" {
            Ok(Self::TDB)
        } else if val == "ET" {
            Ok(Self::ET)
        } else if val == "GPST" {
            Ok(Self::GPST)
        } else if val == "GST" {
            Ok(Self::GST)
        } else if val == "BDT" {
            Ok(Self::BDT)
        } else {
            Err(Errors::ParseError(ParsingErrors::TimeSystem))
        }
    }
}
