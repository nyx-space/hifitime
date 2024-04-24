use crate::{Duration, Epoch, Errors};

/// Converts the webtime Duration into a hifitime Duration.
///
/// Clippy thinks these are the same type, but they aren't.
#[allow(clippy::unnecessary_fallible_conversions)]
pub(crate) fn duration_since_unix_epoch() -> Result<Duration, Errors> {
    web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map_err(|_| Errors::SystemTimeError)
        .and_then(|d| d.try_into().map_err(|_| Errors::SystemTimeError))
}

// This is in its separate impl far away from the Python feature because pyO3's classmethod does not work with cfg_attr
#[cfg(feature = "std")]
impl Epoch {
    /// Initializes a new Epoch from `now`.
    /// WARNING: This assumes that the system time returns the time in UTC (which is the case on Linux)
    /// Uses [`std::time::SystemTime::now`](https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.now)
    /// or javascript interop under the hood
    pub fn now() -> Result<Self, Errors> {
        let duration = duration_since_unix_epoch()?;
        Ok(Self::from_unix_duration(duration))
    }
}
