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

use crate::{Errors, ParsingErrors};

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
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            TimeScale::GPST => 4,
            TimeScale::TAI | TimeScale::TDB | TimeScale::UTC | TimeScale::GST | TimeScale::BDT => 3,
            TimeScale::ET | TimeScale::TT => 2,
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
