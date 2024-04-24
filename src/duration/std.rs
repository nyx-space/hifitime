/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the implementations that are only built with the std flag

extern crate core;

use super::{Duration, Unit};

impl From<Duration> for std::time::Duration {
    /// Converts a duration into an std::time::Duration
    ///
    /// # Limitations
    /// 1. If the duration is negative, this will return a std::time::Duration::ZERO.
    /// 2. If the duration larger than the MAX duration, this will return std::time::Duration::MAX
    fn from(hf_duration: Duration) -> Self {
        let (sign, days, hours, minutes, seconds, milli, us, nano) = hf_duration.decompose();
        if sign < 0 {
            std::time::Duration::ZERO
        } else {
            // Build the seconds separately from the nanos.
            let above_ns_f64: f64 =
                Duration::compose(sign, days, hours, minutes, seconds, milli, us, 0).to_seconds();
            std::time::Duration::new(above_ns_f64 as u64, nano as u32)
        }
    }
}

impl From<std::time::Duration> for Duration {
    /// Converts a duration into an std::time::Duration
    ///
    /// # Limitations
    /// 1. If the duration is negative, this will return a std::time::Duration::ZERO.
    /// 2. If the duration larger than the MAX duration, this will return std::time::Duration::MAX
    fn from(std_duration: std::time::Duration) -> Self {
        std_duration.as_secs_f64() * Unit::Second
    }
}
