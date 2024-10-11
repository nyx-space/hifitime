/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2017-onwards Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{Duration, Epoch, HifitimeError};

/// Converts the webtime Duration into a hifitime Duration.
///
/// Clippy thinks these are the same type, but they aren't.
#[allow(clippy::unnecessary_fallible_conversions)]
pub(crate) fn duration_since_unix_epoch() -> Result<Duration, HifitimeError> {
    // map_err maps the duration_since error into a hifitime error, if the conversion to a SystemTime fails.
    // Then we converts a valid SystemTime into a Hifitime duration, unless it fails via and_then
    web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map_err(|_| HifitimeError::SystemTimeError)
        .and_then(|d| d.try_into().map_err(|_| HifitimeError::SystemTimeError))
}

// This is in its separate impl far away from the Python feature because pyO3's classmethod does not work with cfg_attr
#[cfg(feature = "std")]
impl Epoch {
    /// Initializes a new Epoch from `now`.
    /// WARNING: This assumes that the system time returns the time in UTC (which is the case on Linux)
    /// Uses [`std::time::SystemTime::now`](https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.now)
    /// or javascript interop under the hood
    pub fn now() -> Result<Self, HifitimeError> {
        let duration = duration_since_unix_epoch()?;
        Ok(Self::from_unix_duration(duration))
    }
}
