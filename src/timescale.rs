/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

#[cfg(kani)]
use kani::Arbitrary;

use core::fmt;
use core::str::FromStr;

use crate::{
    Duration, Epoch, Errors, ParsingErrors, J2000_REF_EPOCH_ET, J2000_REF_EPOCH_TDB,
    J2000_TO_J1900_DURATION, SECONDS_PER_DAY,
};

/// The J1900 reference epoch (1900-01-01 at noon) TAI.
pub const J1900_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration::ZERO);

/// The J2000 reference epoch (2000-01-01 at midnight) TAI.
pub const J2000_REF_EPOCH: Epoch = Epoch::from_tai_duration(J2000_TO_J1900_DURATION);

/// GPS reference epoch is UTC midnight between 05 January and 06 January 1980; cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>.
pub const GPST_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 2_524_953_619_000_000_000, // XXX
});
pub const SECONDS_GPS_TAI_OFFSET: f64 = 2_524_953_619.0;
pub const SECONDS_GPS_TAI_OFFSET_I64: i64 = 2_524_953_619;
pub const DAYS_GPS_TAI_OFFSET: f64 = SECONDS_GPS_TAI_OFFSET / SECONDS_PER_DAY;

/// QZSS and GPS share the same reference epoch
pub const QZSST_REF_EPOCH: Epoch = GPST_REF_EPOCH;

/// GST (Galileo) reference epoch is 13 seconds before 1999 August 21 UTC at midnight.
pub const GST_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 3_144_268_819_000_000_000,
});
pub const SECONDS_GST_TAI_OFFSET: f64 = 3_144_268_819.0;
pub const SECONDS_GST_TAI_OFFSET_I64: i64 = 3_144_268_819;

/// BDT(BeiDou): 2005 Dec 31st Midnight
/// BDT (BeiDou) reference epoch is 2005 December 31st UTC at midnight. **This time scale is synchronized with UTC.**
pub const BDT_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 1,
    nanoseconds: 189_302_433_000_000_000,
});
pub const SECONDS_BDT_TAI_OFFSET: f64 = 3_345_062_433.0;
pub const SECONDS_BDT_TAI_OFFSET_I64: i64 = 3_345_062_433;

/// The UNIX reference epoch of 1970-01-01 in TAI duration, accounting only for IERS leap seconds.
pub const UNIX_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 2_208_988_800_000_000_000,
});

/// Enum of the different time systems available
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// GPS Time scale
    GPST,
    /// Galileo Time scale
    GST,
    /// BeiDou Time scale
    BDT,
    /// QZSS Time scale has the same properties as GPST,
    /// but with dedicated clocks
    QZSST,
}

#[cfg(kani)]
impl Arbitrary for TimeScale {
    #[inline(always)]
    fn any() -> Self {
        let ts_u8: u8 = kani::any();
        Self::from(ts_u8)
    }
}

impl Default for TimeScale {
    /// Builds default TAI time scale
    fn default() -> Self {
        /*
         * We use TAI as default Time scale,
         * because `Epoch` is always defined with respect to TAI.
         * Also, a default `Epoch` is then a null duration into TAI.
         */
        Self::TAI
    }
}

impl TimeScale {
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            Self::QZSST => 5,
            Self::GPST => 4,
            Self::TAI | Self::TDB | Self::UTC | Self::GST | Self::BDT => 3,
            Self::ET | Self::TT => 2,
        }
    }

    /// Returns true if Self is based off a GNSS constellation
    pub const fn is_gnss(&self) -> bool {
        matches!(self, Self::GPST | Self::GST | Self::BDT | Self::QZSST)
    }

    /// Returns Self's Reference Epoch: Time Scale initialization date,
    /// expressed as an Epoch in TAI
    pub const fn tai_reference_epoch(&self) -> Epoch {
        match self {
            Self::GPST => GPST_REF_EPOCH,
            Self::GST => GST_REF_EPOCH,
            Self::BDT => BDT_REF_EPOCH,
            Self::ET => J2000_REF_EPOCH_ET,
            Self::TDB => J2000_REF_EPOCH_TDB,
            Self::QZSST => QZSST_REF_EPOCH,
            // Explicit on purpose in case more time scales end up being supported.
            Self::TT | Self::TAI | Self::UTC => J1900_REF_EPOCH,
        }
    }
}

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

#[cfg_attr(feature = "python", pymethods)]
impl TimeScale {
    /// Returns true if self takes leap seconds into account
    pub const fn uses_leap_seconds(&self) -> bool {
        matches!(self, Self::UTC)
    }
}

/// Allows conversion of a TimeSystem into a u8
/// Mapping: TAI: 0; TT: 1; ET: 2; TDB: 3; UTC: 4; GPST: 5; GST: 6; BDT: 7; QZSST: 8;
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
            TimeScale::QZSST => 8,
        }
    }
}

/// Allows conversion of a u8 into a TimeSystem.
/// Mapping: 1: TT; 2: ET; 3: TDB; 4: UTC; 5: GPST; 6: GST; 7: BDT; 8: QZSST; anything else: TAI
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
            8 => Self::QZSST,
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
        } else if val == "GPST" || val == "GPS" {
            Ok(Self::GPST)
        } else if val == "GST" || val == "GAL" {
            Ok(Self::GST)
        } else if val == "BDT" || val == "BDS" {
            Ok(Self::BDT)
        } else if val == "QZSST" || val == "QZSS" {
            Ok(Self::QZSST)
        } else {
            Err(Errors::ParseError(ParsingErrors::TimeSystem))
        }
    }
}

#[test]
#[cfg(feature = "serde")]
fn test_serdes() {
    let ts = TimeScale::UTC;
    let content = "\"UTC\"";
    assert_eq!(content, serde_json::to_string(&ts).unwrap());
    let parsed: TimeScale = serde_json::from_str(content).unwrap();
    assert_eq!(ts, parsed);
}

#[test]
fn test_ts() {
    for ts_u8 in 0..u8::MAX {
        let ts = TimeScale::from(ts_u8);
        let ts_u8_back: u8 = ts.into();
        // If the u8 is greater than 5, it isn't valid and necessarily encoded as TAI.
        if ts_u8 < 9 {
            assert_eq!(ts_u8_back, ts_u8, "got {ts_u8_back} want {ts_u8}");
        } else {
            assert_eq!(ts, TimeScale::TAI);
        }
    }
}

#[cfg(kani)]
#[kani::proof]
fn formal_time_scale() {
    let _time_scale: TimeScale = kani::any();
}
