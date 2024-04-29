/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use core::str::FromStr;

use crate::ParsingError;

use super::TimeScale;

impl fmt::Display for TimeScale {
    /// Prints given TimeScale
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TAI => write!(f, "TAI"),
            Self::TT => write!(f, "TT"),
            Self::ET => write!(f, "ET"),
            Self::TDB => write!(f, "TDB"),
            Self::UTC => write!(f, "UTC"),
            Self::GPST => write!(f, "GPST"),
            Self::GST => write!(f, "GST"),
            Self::BDT => write!(f, "BDT"),
            Self::QZSST => write!(f, "QZSST"),
        }
    }
}

impl fmt::LowerHex for TimeScale {
    /// Prints given TimeScale in RINEX format
    /// ie., standard GNSS constellation name is preferred when possible
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GPST => write!(f, "GPS"),
            Self::GST => write!(f, "GAL"),
            Self::BDT => write!(f, "BDS"),
            Self::QZSST => write!(f, "QZSS"),
            _ => write!(f, "{self}"),
        }
    }
}

impl FromStr for TimeScale {
    type Err = ParsingError;

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
        } else if val == "GPST" || val == "GPS" {
            Ok(Self::GPST)
        } else if val == "GST" || val == "GAL" {
            Ok(Self::GST)
        } else if val == "BDT" || val == "BDS" {
            Ok(Self::BDT)
        } else if val == "QZSST" || val == "QZSS" {
            Ok(Self::QZSST)
        } else {
            Err(ParsingError::TimeSystem)
        }
    }
}
