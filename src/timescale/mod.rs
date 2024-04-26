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
mod kani;

mod fmt;

use crate::{Duration, Epoch, Unit, SECONDS_PER_DAY};

/// The J1900 reference epoch (1900-01-01 at noon) TAI.
pub const J1900_REF_EPOCH: Epoch = Epoch {
    duration: Duration {
        centuries: 0,
        nanoseconds: 43200000000000,
    },
    time_scale: TimeScale::TAI,
};

/// The J2000 reference epoch (2000-01-01 at midnight) TAI.
/// |UTC - TAI| = XX Leap Seconds on that day.
pub const J2000_REF_EPOCH: Epoch = Epoch {
    duration: Duration {
        centuries: 1,
        nanoseconds: 43200000000000,
    },
    time_scale: TimeScale::TAI,
};

pub const GPST_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 2_524_953_619_000_000_000, // XXX
});
pub const SECONDS_GPS_TAI_OFFSET: f64 = 2_524_953_619.0;
pub const SECONDS_GPS_TAI_OFFSET_I64: i64 = 2_524_953_619;
pub const DAYS_GPS_TAI_OFFSET: f64 = SECONDS_GPS_TAI_OFFSET / SECONDS_PER_DAY;

/// QZSS and GPS share the same reference epoch.
pub const QZSST_REF_EPOCH: Epoch = GPST_REF_EPOCH;

/// GST (Galileo) reference epoch is 13 seconds before 1999 August 21 UTC at midnight.
/// |UTC - TAI| = XX Leap Seconds on that day.
pub const GST_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 3_144_268_819_000_000_000, // 3_144_268_800_000_000_000,
});
pub const SECONDS_GST_TAI_OFFSET: f64 = 3_144_268_819.0;
pub const SECONDS_GST_TAI_OFFSET_I64: i64 = 3_144_268_819;

/// BDT(BeiDou): 2005 Dec 31st Midnight
/// BDT (BeiDou) reference epoch is 2005 December 31st UTC at midnight. **This time scale is synchronized with UTC.**
/// |UTC - TAI| = XX Leap Seconds on that day.
pub const BDT_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 1,
    nanoseconds: 189_302_433_000_000_000, //189_302_400_000_000_000,
});
pub const SECONDS_BDT_TAI_OFFSET: f64 = 3_345_062_433.0;
pub const SECONDS_BDT_TAI_OFFSET_I64: i64 = 3_345_062_433;

/// The UNIX reference epoch of 1970-01-01 in TAI duration, accounting only for IERS leap seconds.
pub const UNIX_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 2_208_988_800_000_000_000,
});

/// Reference year of the Hifitime prime epoch.
pub(crate) const HIFITIME_REF_YEAR: i32 = 1900;

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
    /// GPS Time scale whose reference epoch is UTC midnight between 05 January and 06 January 1980; cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>. |UTC - TAI| = 19 Leap Seconds on that day.
    GPST,
    /// Galileo Time scale
    GST,
    /// BeiDou Time scale
    BDT,
    /// QZSS Time scale has the same properties as GPST but with dedicated clocks
    QZSST,
}

impl Default for TimeScale {
    /// Builds default TAI time scale
    fn default() -> Self {
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

    /// Returns this time scale's reference epoch: Time Scale initialization date,
    /// expressed as an Epoch in TAI
    pub const fn reference_epoch(self) -> Epoch {
        Epoch {
            duration: Duration::ZERO,
            time_scale: self,
        }
    }

    /// Returns the duration between this time scale's reference epoch and the hifitime "prime epoch" of 1900-01-01 00:00:00 TAI (the NTP prime epoch).
    /// This is used to compute the Gregorian date representations in any time scale.
    pub(crate) const fn prime_epoch_offset(self) -> Duration {
        match self {
            TimeScale::ET | TimeScale::TDB => {
                // Both ET and TDB are defined at J2000, which is 2000-01-01 12:00:00 and there were only 36524 days in the 20th century.
                // Hence, this math is the output of (Unit.Century*1 + Unit.Hour*12 - Unit.Day*1).to_parts() via Hifitime in Python.
                Duration {
                    centuries: 0,
                    nanoseconds: 3155716800000000000,
                }
            }
            TimeScale::GPST | TimeScale::QZSST => Duration {
                centuries: 0,
                nanoseconds: 2_524_953_619_000_000_000,
            },
            TimeScale::GST => Duration {
                centuries: 0,
                nanoseconds: 3_144_268_819_000_000_000,
            },
            TimeScale::BDT => Duration {
                centuries: 1,
                nanoseconds: 189_302_433_000_000_000,
            },
            _ => Duration::ZERO,
        }
    }

    pub(crate) fn gregorian_epoch_offset(self) -> Duration {
        let prime_offset = self.prime_epoch_offset();

        prime_offset - prime_offset.subdivision(Unit::Second).unwrap()
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

#[cfg(test)]
mod unit_test_timescale {
    use super::TimeScale;

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

    #[test]
    #[cfg(feature = "std")]
    fn test_ref_epoch() {
        use crate::{Duration, Epoch, Unit};
        let prime_e = Epoch::from_duration(Duration::ZERO, TimeScale::TAI);
        assert_eq!(prime_e.duration, Duration::ZERO);
        assert_eq!(format!("{prime_e}"), "1900-01-01T00:00:00 TAI");
        // NOTE: There are only 36524 days in the 20th century, but one century is 36425, so we "overflow" the next century by one day!
        assert_eq!(
            format!("{}", prime_e + Unit::Century * 1),
            "2000-01-02T00:00:00 TAI"
        );

        assert_eq!(
            format!("{}", TimeScale::ET.reference_epoch()),
            "2000-01-01T12:00:00 ET"
        );
    }
}
