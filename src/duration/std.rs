/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the implementations that are only built with the std flag

extern crate core;

use super::Duration;

impl From<Duration> for std::time::Duration {
    /// Converts a duration into an std::time::Duration
    ///
    /// # Limitations
    /// 1. If the duration is negative, this will return a std::time::Duration::ZERO.
    /// 2. If the duration is larger than the MAX duration, this will return std::time::Duration::MAX
    fn from(hf_duration: Duration) -> Self {
        use crate::NANOSECONDS_PER_SECOND;
        if hf_duration.signum() == -1 {
            std::time::Duration::ZERO
        } else {
            let nanos = hf_duration.total_nanoseconds();
            let unsigned_nanos = u128::try_from(nanos).unwrap_or(0);

            let secs: u64 = (unsigned_nanos / NANOSECONDS_PER_SECOND as u128)
                .try_into()
                .unwrap_or(u64::MAX);
            let subsec_nanos = (unsigned_nanos % NANOSECONDS_PER_SECOND as u128) as u32;

            std::time::Duration::new(secs, subsec_nanos)
        }
    }
}

impl From<std::time::Duration> for Duration {
    /// Converts a duration into an std::time::Duration
    ///
    /// # Limitations
    /// 1. If the duration is negative, this will return a std::time::Duration::ZERO.
    /// 2. If the duration is larger than the MAX duration, this will return std::time::Duration::MAX
    fn from(std_duration: std::time::Duration) -> Self {
        Duration::from_total_nanoseconds(std_duration.as_nanos().try_into().unwrap_or(i128::MAX))
    }
}
