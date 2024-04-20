use crate::{Duration, Errors};

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
