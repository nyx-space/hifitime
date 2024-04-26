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

use crate::{Epoch, TimeScale};

impl fmt::Display for Epoch {
    /// Print this epoch in Gregorian in the time scale used at initialization
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.duration, self.time_scale);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, self.time_scale
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, self.time_scale
            )
        }
    }
}

impl fmt::Debug for Epoch {
    /// The default format of an epoch is in UTC
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::UTC;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::LowerHex for Epoch {
    /// Prints the Epoch in TAI
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::TAI;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::UpperHex for Epoch {
    /// Prints the Epoch in TT
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::TT;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::LowerExp for Epoch {
    /// Prints the Epoch in TDB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::TDB;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::UpperExp for Epoch {
    /// Prints the Epoch in ET
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ts = TimeScale::ET;
        let (y, mm, dd, hh, min, s, nanos) =
            Self::compute_gregorian(self.to_duration_in_time_scale(ts), ts);
        if nanos == 0 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
                y, mm, dd, hh, min, s, ts
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09} {}",
                y, mm, dd, hh, min, s, nanos, ts
            )
        }
    }
}

impl fmt::Pointer for Epoch {
    /// Prints the Epoch in UNIX
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_unix_seconds())
    }
}

impl fmt::Octal for Epoch {
    /// Prints the Epoch in GPS
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_gpst_nanoseconds().unwrap())
    }
}
