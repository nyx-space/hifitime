// Disclamer: this is heavily inspired by std::time::Duration, but it supports longer
// time spans and leap seconds. Moreover, an Instant is defined with respect to
// 01 Jan 1900, as per NTP specifications.

use std::ops::{Add, Sub};
use std::time::Duration;
use std::fmt;

/// An `Era` represents whether the associated `Instant` is before the TAI Epoch
/// (01 Jan 1900, midnight) or afterwards. If it is before, than it's refered to as "Past",
/// otherwise is in the "Present" era.
#[derive(Copy, Clone, Debug)]
pub enum Era {
    Present,
    Past,
}

impl fmt::Display for Era {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Era::Present => write!(f, "Present"),
            Era::Past => write!(f, "Past"),
        }
    }
}
/// An `Instant` type represents an instant with respect to 01 Jan 1900 at midnight, as per
/// the International Atomic Time (TAI) system.
#[derive(Copy, Clone, Debug)]
pub struct Instant {
    duration: Duration,
    era: Era,
}

impl Instant {
    /// Creates a new `Instant` with respect to TAI Epoch. All time systems are represented
    /// with respect to this epoch.
    /// Note: this constructor relies on the constructor for std::time::Duration; as such,
    /// refer to https://doc.rust-lang.org/std/time/struct.Duration.html#method.new for
    /// pertinent warnings and limitations.
    pub fn new(seconds: u64, nanos: u32, era: Era) -> Instant {
        Instant {
            duration: Duration::new(seconds, nanos),
            era: era,
        }
    }

    pub fn secs(self) -> u64 {
        self.duration.as_secs()
    }

    pub fn nanos(self) -> u32 {
        self.duration.subsec_nanos()
    }

    pub fn era(self) -> Era {
        self.era
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, delta: Duration) -> Instant {
        // Switch the era, an exact time of zero is in the Present era
        match self.era {
            Era::Past => {
                if (delta.as_secs() >= self.duration.as_secs()) ||
                    (delta.as_secs() >= self.duration.as_secs() && delta.as_secs() == 0 &&
                         delta.subsec_nanos() >= self.duration.subsec_nanos())
                {
                    return Instant::new(
                        delta.as_secs() - self.duration.as_secs(),
                        delta.subsec_nanos() - self.duration.subsec_nanos(),
                        Era::Present,
                    );
                } else {
                    let mut cln = self.clone();
                    cln.duration -= delta;
                    return cln;
                }
            }
            Era::Present => {
                // Adding a duration in the present is trivial
                let mut cln = self.clone();
                cln.duration += delta;
                return cln;
            }
        }
    }
}
