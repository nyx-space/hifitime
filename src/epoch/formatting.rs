/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use core::fmt;

use crate::{DurationTrait, GenericEpoch, TimeScale};

/// Fractional part of the seconds: nine digits when there is no sub-nanosecond part (the historical [crate::Epoch] format),
/// otherwise all of the significant digits down to the zeptosecond (trailing zeros removed).
pub(crate) struct FractionalDigits {
    nanos: u32,
    subns: u64,
}

impl FractionalDigits {
    /// Define [FractionalDigits] from both parts, nanos and subnanos (when they exist, zero
    /// otherwise).
    pub fn new(nanos: u32, subns: u64) -> Self {
        Self { nanos, subns }
    }
}

impl fmt::Display for FractionalDigits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.subns == 0 {
            write!(f, "{:09}", self.nanos)
        } else {
            let mut subns = self.subns;
            let mut width = 12;
            while subns.is_multiple_of(10) {
                subns /= 10;
                width -= 1;
            }
            write!(f, "{:09}{:0width$}", self.nanos, subns, width = width)
        }
    }
}

impl<D: DurationTrait> GenericEpoch<D> {
    /// Writes the Gregorian representation of this epoch in the provided time scale.
    fn fmt_gregorian(&self, f: &mut fmt::Formatter, ts: TimeScale) -> fmt::Result {
        let (y, mm, dd, hh, min, s, nanos, subns) = self.to_gregorian_precise(ts);
        if nanos == 0 && subns == 0 {
            write!(f, "{y:04}-{mm:02}-{dd:02}T{hh:02}:{min:02}:{s:02} {ts}")
        } else {
            write!(
                f,
                "{y:04}-{mm:02}-{dd:02}T{hh:02}:{min:02}:{s:02}.{} {ts}",
                FractionalDigits::new(nanos, subns)
            )
        }
    }
}

impl<D: DurationTrait> fmt::Display for GenericEpoch<D> {
    /// Print this epoch in Gregorian in the time scale used at initialization
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_gregorian(f, self.time_scale)
    }
}

impl<D: DurationTrait> fmt::Debug for GenericEpoch<D> {
    /// The default format of an epoch is in UTC
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_gregorian(f, TimeScale::UTC)
    }
}

impl<D: DurationTrait> fmt::LowerHex for GenericEpoch<D> {
    /// Prints the Epoch in TAI
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_gregorian(f, TimeScale::TAI)
    }
}

impl<D: DurationTrait> fmt::UpperHex for GenericEpoch<D> {
    /// Prints the Epoch in TT
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_gregorian(f, TimeScale::TT)
    }
}

impl<D: DurationTrait> fmt::LowerExp for GenericEpoch<D> {
    /// Prints the Epoch in TDB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_gregorian(f, TimeScale::TDB)
    }
}

impl<D: DurationTrait> fmt::UpperExp for GenericEpoch<D> {
    /// Prints the Epoch in ET
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_gregorian(f, TimeScale::ET)
    }
}

impl<D: DurationTrait> fmt::Pointer for GenericEpoch<D> {
    /// Prints the Epoch in UNIX
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_unix_seconds())
    }
}

impl<D: DurationTrait> fmt::Octal for GenericEpoch<D> {
    /// Prints the Epoch in GPS nanoseconds (floored for a [crate::PreciseEpoch])
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_gpst_duration().to_duration().total_nanoseconds()
        )
    }
}
