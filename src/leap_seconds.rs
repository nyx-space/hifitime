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

#[cfg(feature = "std")]
pub use super::leap_seconds_file::LeapSecondsFile;

use core::ops::Index;

pub trait LeapSecondProvider: DoubleEndedIterator<Item = LeapSecond> + Index<usize> {}

/// A structure representing a leap second
#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LeapSecond {
    /// Timestamp in TAI seconds for this leap second, e.g. `2_272_060_800.0` for the first IERS leap second.
    pub timestamp_tai_s: f64,
    /// ΔAT is the accumulated time offset after this leap second has past.
    pub delta_at: f64,
    /// Whether or not this leap second was announced by the IERS.
    pub announced_by_iers: bool,
}

impl LeapSecond {
    pub const fn new(timestamp_tai_s: f64, delta_at: f64, announced: bool) -> Self {
        Self {
            timestamp_tai_s,
            delta_at,
            announced_by_iers: announced,
        }
    }
}

const LATEST_LEAP_SECONDS: [LeapSecond; 42] = [
    LeapSecond::new(1_893_369_600.0, 1.417818, false), // SOFA: 01 Jan 1960
    LeapSecond::new(1_924_992_000.0, 1.422818, false), // SOFA: 01 Jan 1961
    LeapSecond::new(1_943_308_800.0, 1.372818, false), // SOFA: 01 Aug 1961
    LeapSecond::new(1_956_528_000.0, 1.845858, false), // SOFA: 01 Jan 1962
    LeapSecond::new(2_014_329_600.0, 1.945858, false), // SOFA: 01 Jan 1963
    LeapSecond::new(2_019_600_000.0, 3.24013, false),  // SOFA: 01 Jan 1964
    LeapSecond::new(2_027_462_400.0, 3.34013, false),  // SOFA: 01 Apr 1964
    LeapSecond::new(2_040_681_600.0, 3.44013, false),  // SOFA: 01 Sep 1964
    LeapSecond::new(2_051_222_400.0, 3.54013, false),  // SOFA: 01 Jan 1965
    LeapSecond::new(2_056_320_000.0, 3.64013, false),  // SOFA: 01 Mar 1965
    LeapSecond::new(2_066_860_800.0, 3.74013, false),  // SOFA: 01 Jul 1965
    LeapSecond::new(2_072_217_600.0, 3.84013, false),  // SOFA: 01 Sep 1965
    LeapSecond::new(2_082_758_400.0, 4.31317, false),  // SOFA: 01 Jan 1966
    LeapSecond::new(2_148_508_800.0, 4.21317, false),  // SOFA: 01 Feb 1968
    LeapSecond::new(2_272_060_800.0, 10.0, true),      // IERS: 01 Jan 1972
    LeapSecond::new(2_287_785_600.0, 11.0, true),      // IERS: 01 Jul 1972
    LeapSecond::new(2_303_683_200.0, 12.0, true),      // IERS: 01 Jan 1973
    LeapSecond::new(2_335_219_200.0, 13.0, true),      // IERS: 01 Jan 1974
    LeapSecond::new(2_366_755_200.0, 14.0, true),      // IERS: 01 Jan 1975
    LeapSecond::new(2_398_291_200.0, 15.0, true),      // IERS: 01 Jan 1976
    LeapSecond::new(2_429_913_600.0, 16.0, true),      // IERS: 01 Jan 1977
    LeapSecond::new(2_461_449_600.0, 17.0, true),      // IERS: 01 Jan 1978
    LeapSecond::new(2_492_985_600.0, 18.0, true),      // IERS: 01 Jan 1979
    LeapSecond::new(2_524_521_600.0, 19.0, true),      // IERS: 01 Jan 1980
    LeapSecond::new(2_571_782_400.0, 20.0, true),      // IERS: 01 Jul 1981
    LeapSecond::new(2_603_318_400.0, 21.0, true),      // IERS: 01 Jul 1982
    LeapSecond::new(2_634_854_400.0, 22.0, true),      // IERS: 01 Jul 1983
    LeapSecond::new(2_698_012_800.0, 23.0, true),      // IERS: 01 Jul 1985
    LeapSecond::new(2_776_982_400.0, 24.0, true),      // IERS: 01 Jan 1988
    LeapSecond::new(2_840_140_800.0, 25.0, true),      // IERS: 01 Jan 1990
    LeapSecond::new(2_871_676_800.0, 26.0, true),      // IERS: 01 Jan 1991
    LeapSecond::new(2_918_937_600.0, 27.0, true),      // IERS: 01 Jul 1992
    LeapSecond::new(2_950_473_600.0, 28.0, true),      // IERS: 01 Jul 1993
    LeapSecond::new(2_982_009_600.0, 29.0, true),      // IERS: 01 Jul 1994
    LeapSecond::new(3_029_443_200.0, 30.0, true),      // IERS: 01 Jan 1996
    LeapSecond::new(3_076_704_000.0, 31.0, true),      // IERS: 01 Jul 1997
    LeapSecond::new(3_124_137_600.0, 32.0, true),      // IERS: 01 Jan 1999
    LeapSecond::new(3_345_062_400.0, 33.0, true),      // IERS: 01 Jan 2006
    LeapSecond::new(3_439_756_800.0, 34.0, true),      // IERS: 01 Jan 2009
    LeapSecond::new(3_550_089_600.0, 35.0, true),      // IERS: 01 Jul 2012
    LeapSecond::new(3_644_697_600.0, 36.0, true),      // IERS: 01 Jul 2015
    LeapSecond::new(3_692_217_600.0, 37.0, true),      // IERS: 01 Jan 2017
];

/// List of leap seconds from https://www.ietf.org/timezones/data/leap-seconds.list .
/// This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
/// The unannoucned leap seconds come from dat.c in the SOFA library.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug)]
pub struct LatestLeapSeconds {
    data: [LeapSecond; 42],
    iter_pos: usize,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl LatestLeapSeconds {
    #[new]
    pub fn __new__() -> Self {
        Self::default()
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

impl Default for LatestLeapSeconds {
    fn default() -> Self {
        Self {
            data: LATEST_LEAP_SECONDS,
            iter_pos: 0,
        }
    }
}

impl Iterator for LatestLeapSeconds {
    type Item = LeapSecond;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_pos += 1;
        self.data.get(self.iter_pos - 1).copied()
    }
}

impl DoubleEndedIterator for LatestLeapSeconds {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.iter_pos == self.data.len() {
            None
        } else {
            self.iter_pos += 1;
            self.data.get(self.data.len() - self.iter_pos).copied()
        }
    }
}

impl Index<usize> for LatestLeapSeconds {
    type Output = LeapSecond;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl LeapSecondProvider for LatestLeapSeconds {}
