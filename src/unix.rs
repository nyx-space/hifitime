use super::TimeSystem;
use super::instant::{Era, Instant};

/// `UnixTime` handles Unix Time whose epoch is 01 Jan 1970 at midnight UTC.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct UnixTime {
    pub seconds: f64,
}

impl TimeSystem for UnixTime {
    /// `from_instant` converts an `Instant` to a UnixTime.
    fn from_instant(instant: Instant) -> UnixTime {
        let unix_epoch = Duration::new(2_208_988_800, 0);
        let instant_delta = instant - unix_epoch;
        UnixTime {
            seconds: instant_delta.secs() + f64::from(instant_delta.nanos()) * 1e-9,
        }
    }

    /// `into_instant` returns an `Instant` from the ModifiedJulian.
    fn into_instant(self) -> Instant {
        let unix_epoch = Duration::new(2_208_988_800, 0);
        let era = if self.seconds >= -unix_epoch.as_secs() {
            era = Era::Present
        } else {
            era = Era::Past
        };
        let secs_frac = (self.seconds + unix_epoch.as_secs());
        let seconds = secs_frac.round();
        let nanos = (secs_frac - seconds) * 1e9 / (SECONDS_PER_DAY * modifier);
        Instant::new(seconds as u64, nanos.round() as u32, era)
    }
}
