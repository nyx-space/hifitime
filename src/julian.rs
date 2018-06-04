pub use super::instant::{Era, Instant};
pub use super::TimeSystem;
use super::{J1900_OFFSET, J2000_OFFSET, SECONDS_PER_DAY};
use std::fmt;

/// `ModifiedJulian` handles the Modified Julian Days as explained
/// [here](http://tycho.usno.navy.mil/mjd.html).
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct ModifiedJulian {
    pub days: f64,
}

impl ModifiedJulian {
    pub fn j2000() -> ModifiedJulian {
        ModifiedJulian { days: 51_544.5 }
    }

    /// `julian_days` returns the true Julian days from epoch 01 Jan -4713, 12:00
    /// as explained in "Fundamentals of astrodynamics and applications", Vallado et al.
    /// 4th edition, page 182, and on [Wikipedia](https://en.wikipedia.org/wiki/Julian_day).
    pub fn julian_days(self) -> f64 {
        self.days + 2_400_000.5
    }

    /// Returns the centuries since J2000. This number is often referred to as JD_tt.
    pub fn centuries_since_j2000(self) -> f64 {
        (self.days - J2000_OFFSET) / 36_525.0
    }
}

impl TimeSystem for ModifiedJulian {
    /// `from_instant` converts an `Instant` to a ModifiedJulian as detailed in Vallado et al.
    /// 4th edition, page 182.
    ///
    /// [Leap second source](https://www.ietf.org/timezones/data/leap-seconds.list) contains
    /// information pertinent to the NTP time definition, whose epoch is twelve hours *ahead* of
    /// the Julian Day. Here is the relevant quote:
    /// > The NTP timestamps are in units of seconds since the NTP epoch,
    /// > which is 1 January 1900, 00:00:00. The Modified Julian Day number
    /// > corresponding to the NTP time stamp, X, can be computed as
    /// >
    /// > `X/86400 + 15020`
    /// >
    /// > where the first term converts seconds to days and the second
    /// > term adds the MJD corresponding to the time origin defined above.
    /// > The integer portion of the result is the integer MJD for that
    /// > day, and any remainder is the time of day, expressed as the
    /// > fraction of the day since 0 hours UTC. The conversion from day
    /// > fraction to seconds or to hours, minutes, and seconds may involve
    /// > rounding or truncation, depending on the method used in the
    /// > computation.
    fn from_instant(instant: Instant) -> ModifiedJulian {
        let modifier = if instant.era() == Era::Present {
            1.0
        } else {
            -1.0
        };
        ModifiedJulian {
            days: J1900_OFFSET
                + modifier * (instant.secs() as f64) / SECONDS_PER_DAY
                + f64::from(instant.nanos()) * 1e-9,
        }
    }

    /// `into_instant` returns an `Instant` from the ModifiedJulian.
    fn into_instant(self) -> Instant {
        let era: Era;
        let modifier = if self.days >= J1900_OFFSET {
            era = Era::Present;
            1.0
        } else {
            era = Era::Past;
            -1.0
        };
        let secs_frac = (self.days - J1900_OFFSET) * SECONDS_PER_DAY * modifier;
        let seconds = secs_frac.round();
        let nanos = (secs_frac - seconds) * 1e9 / (SECONDS_PER_DAY * modifier);
        Instant::new(seconds as u64, nanos.round() as u32, era)
    }
}

/// The formatter will show six digits of precision. This allows for a resolution of about 0.864 seconds.
impl fmt::Display for ModifiedJulian {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.5}", self.days)
    }
}
