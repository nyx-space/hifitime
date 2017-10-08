use super::traits;
use super::instant::{Era, Instant};

#[derive(Copy, Clone, Debug)]
pub struct ModifiedJulian {
    days: f64,
}

impl traits::TimeSystem for ModifiedJulian {
    /// `from_instant` converts an Instant to a ModifiedJulian as detailed
    /// in https://www.ietf.org/timezones/data/leap-seconds.list , specifically the following
    /// quote:
    /// The NTP timestamps are in units of seconds since the NTP epoch,
    /// which is 1 January 1900, 00:00:00. The Modified Julian Day number
    /// corresponding to the NTP time stamp, X, can be computed as
    ///
    /// X/86400 + 15020
    ///
    /// where the first term converts seconds to days and the second
    /// term adds the MJD corresponding to the time origin defined above.
    /// The integer portion of the result is the integer MJD for that
    /// day, and any remainder is the time of day, expressed as the
    /// fraction of the day since 0 hours UTC. The conversion from day
    /// fraction to seconds or to hours, minutes, and seconds may involve
    /// rounding or truncation, depending on the method used in the
    /// computation.
    fn from_instant(instant: Instant) -> ModifiedJulian {
        ModifiedJulian {
            days: (instant.secs() as f64 + instant.nanos() as f64 * 1e-9) / 86400.0 + 15020.0,
        }
    }

    fn as_instant(self) -> Instant {
        let era: Era;
        if self.days >= 15020.0 {
            era = Era::Present;
        } else {
            era = Era::Past;
        }
        let seconds = (self.days * 86400.0).floor() as u64;
        let nanos = (self.days * 86400.0 - seconds as f64 * 1e9) as u32;
        Instant::new(seconds, nanos, era)
    }
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
