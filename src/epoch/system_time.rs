/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{Duration, Epoch, EpochError};

/// Converts the webtime Duration into a hifitime Duration.
///
/// Clippy thinks these are the same type, but they aren't.
#[allow(clippy::unnecessary_fallible_conversions)]
pub(crate) fn duration_since_unix_epoch() -> Result<Duration, EpochError> {
    // TODO: Check why there is a map_err and and_then
    web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map_err(|_| EpochError::SystemTimeError)
        .and_then(|d| d.try_into().map_err(|_| EpochError::SystemTimeError))
}

// This is in its separate impl far away from the Python feature because pyO3's classmethod does not work with cfg_attr
#[cfg(feature = "std")]
impl Epoch {
    /// Initializes a new Epoch from `now`.
    /// WARNING: This assumes that the system time returns the time in UTC (which is the case on Linux)
    /// Uses [`std::time::SystemTime::now`](https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.now)
    /// or javascript interop under the hood
    pub fn now() -> Result<Self, EpochError> {
        let duration = duration_since_unix_epoch()?;
        Ok(Self::from_unix_duration(duration))
    }
}
