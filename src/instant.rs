// Disclaimer: this is heavily inspired by std::time::Duration, but it supports longer
// time spans and leap seconds. Moreover, an Instant is defined with respect to
// 01 Jan 1900, as per NTP and TAI specifications.

pub use std::time::Duration;

use std::cmp::PartialEq;
use std::ops::{Add, Sub};
use std::time::SystemTime;
use std::fmt;

/// An `Era` represents whether the associated `Instant` is before the TAI Epoch
/// (01 Jan 1900, midnight) or afterwards. If it is before, than it's refered to as "Past",
/// otherwise is in the "Present" era.
///
/// ```
/// use hifitime::instant::Era;
/// assert!(Era::Past < Era::Present);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Era {
    Past,
    Present,
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
#[derive(Clone, Copy, Debug, PartialOrd)]
pub struct Instant {
    era: Era,
    duration: Duration,
}

impl Instant {
    /// Creates a new `Instant` with respect to TAI Epoch: 01 January 1900, 00:00:00.0.
    /// All time systems are represented with respect to this epoch.
    /// Note: this constructor relies on the constructor for std::time::Duration; as such,
    /// refer to [`std::time::Duration::new`](https://doc.rust-lang.org/std/time/struct.Duration.html#method.new)
    /// for pertinent warnings and limitations.
    ///
    /// # Examples
    /// ```
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let epoch = Instant::new(0, 0, Era::Present);
    /// assert_eq!(epoch.secs(), 0);
    /// assert_eq!(epoch.nanos(), 0);
    ///
    /// let one_second_before_1900 = Instant::new(1, 0, Era::Past);
    /// assert_eq!(one_second_before_1900.secs(), 1);
    /// assert_eq!(one_second_before_1900.era(), Era::Past);
    ///
    /// let one_second_after_1900 = Instant::new(1, 0, Era::Present);
    /// assert_eq!(one_second_after_1900.secs(), 1);
    /// assert_eq!(one_second_after_1900.era(), Era::Present);
    ///
    /// assert!(one_second_after_1900 > epoch);
    /// assert!(one_second_after_1900 >= epoch);
    /// assert!(one_second_before_1900 < epoch);
    /// assert!(one_second_before_1900 <= epoch);
    /// assert!(Instant::new(1, 0, Era::Past) < Instant::new(0, 0, Era::Present));
    /// assert!(Instant::new(1, 0, Era::Past) < Instant::new(1, 0, Era::Present));
    /// // NOTE: Equality exists at epoch (or zero offset)
    /// assert_eq!(Instant::new(0, 0, Era::Past), Instant::new(0, 0, Era::Present));
    /// assert_ne!(Instant::new(0, 1, Era::Past), Instant::new(0, 1, Era::Present));
    /// assert_ne!(Instant::new(1, 1, Era::Past), Instant::new(1, 1, Era::Present));
    /// assert_ne!(Instant::new(1, 0, Era::Past), Instant::new(1, 0, Era::Present));
    /// ```
    pub fn new(seconds: u64, nanos: u32, era: Era) -> Instant {
        Instant {
            duration: Duration::new(seconds, nanos),
            era: era,
        }
    }

    /// Creates a new `Instant` from the number of seconds compared to `Era`, provided as a floating point value.
    ///
    /// # Example
    /// ```
    /// use hifitime::instant::{Era, Instant};
    ///
    /// let example = Instant::new(159, 159, Era::Present);
    /// let in_secs = example.secs() as f64 + (example.nanos() as f64) * 1e-9;
    /// let precise = Instant::from_precise_seconds(in_secs, Era::Present);
    /// assert_eq!(precise, example);
    ///
    /// let example = Instant::new(159, 159, Era::Past);
    /// let in_secs = example.secs() as f64 + (example.nanos() as f64) * 1e-9;
    /// let precise = Instant::from_precise_seconds(in_secs, Era::Past);
    /// assert_eq!(precise, example);
    /// ```
    pub fn from_precise_seconds(seconds: f64, era: Era) -> Instant {
        let secs_u = seconds.round();
        let nanos = (seconds - secs_u) * 1e9;
        Instant {
            duration: Duration::new(seconds as u64, nanos.round() as u32),
            era: era,
        }
    }

    /// Creates a new `Instant` corresponding to the UNIX epoch of 1970 Jan 01, midnight.
    ///
    /// # Example
    /// ```
    /// use hifitime::TimeSystem;
    /// use hifitime::instant::Instant;
    /// use hifitime::datetime::Datetime;
    ///
    /// let epoch_dt = Datetime::new(1970, 1, 1, 0, 0, 0, 0).expect("epoch");
    /// assert_eq!(Datetime::from_instant(Instant::unix_epoch()), epoch_dt);
    /// ```
    pub fn unix_epoch() -> Instant {
        Instant::new(2_208_988_800, 0, Era::Present)
    }
    /*
    pub fn now() -> Instant {
        let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
            Err(e) => {
                // unlikely but should be handled
                let dur = e.duration();
                let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
                if nsec == 0 {
                    (-sec, 0)
                } else {
                    (-sec - 1, 1_000_000_000 - nsec)
                }
            }
        };
    }
*/
    /// Returns the Duration with respect to Epoch (past OR present), check the `era()`
    pub fn duration(self) -> Duration {
        self.duration
    }

    /// Returns the number of seconds with respect to the epoch.
    pub fn secs(self) -> u64 {
        self.duration.as_secs()
    }

    /// Returns the number of nanoseconds of the given instant.
    pub fn nanos(self) -> u32 {
        self.duration.subsec_nanos()
    }

    /// Returns the Era associated with this instant, i.e. whether it's before or after
    /// the TAI Epoch.
    pub fn era(self) -> Era {
        self.era
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Instant) -> bool {
        let spans_eq = self.secs() == other.secs() && self.nanos() == other.nanos();
        if spans_eq && self.secs() == 0 && self.nanos() == 0 {
            // Do not check the era if both Instants are strictly zero seconds before or after epoch
            true
        } else {
            spans_eq && self.era() == other.era()
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    /// Adds a given `std::time::Duration` to an `Instant`.
    ///
    /// # Examples
    /// ```
    /// use hifitime::instant::{Era, Instant, Duration};
    /// // Add in the Present era.
    /// let tick = Instant::new(159, 10, Era::Present) + Duration::new(5, 2);
    /// assert_eq!(tick.secs(), 164);
    /// assert_eq!(tick.nanos(), 12);
    /// assert_eq!(tick.era(), Era::Present);

    /// // Add in the Past era.
    /// let tick = Instant::new(159, 10, Era::Past) + Duration::new(5, 2);
    /// assert_eq!(tick.secs(), 154);
    /// assert_eq!(tick.nanos(), 8);
    /// assert_eq!(tick.era(), Era::Past);

    /// // Add from the Past to overflow into the Present
    /// let tick = Instant::new(159, 0, Era::Past) + Duration::new(160, 0);
    /// assert_eq!(tick.secs(), 1);
    /// assert_eq!(tick.nanos(), 0);
    /// assert_eq!(tick.era(), Era::Present);

    /// let tick = Instant::new(0, 5, Era::Past) + Duration::new(0, 6);
    /// assert_eq!(tick.secs(), 0);
    /// assert_eq!(tick.nanos(), 1);
    /// assert_eq!(tick.era(), Era::Present);
    /// ```
    fn add(self, delta: Duration) -> Instant {
        if delta.as_secs() == 0 && delta.subsec_nanos() == 0 {
            self
        } else {
            // Switch the era, an exact time of zero is in the Present era
            match self.era {
                Era::Past => {
                    if (delta.as_secs() >= self.duration.as_secs())
                        || (delta.as_secs() >= self.duration.as_secs() && delta.as_secs() == 0
                            && delta.subsec_nanos() >= self.duration.subsec_nanos())
                    {
                        Instant::new(
                            delta.as_secs() - self.duration.as_secs(),
                            delta.subsec_nanos() - self.duration.subsec_nanos(),
                            Era::Present,
                        )
                    } else {
                        let mut cln = self;
                        cln.duration -= delta;
                        cln
                    }
                }
                Era::Present => {
                    // Adding a duration in the present is trivial
                    let mut cln = self;
                    cln.duration += delta;
                    cln
                }
            }
        }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    /// Subtracts a given `std::time::Duration` from an `Instant`.
    /// # Examples
    ///
    /// ```
    /// use hifitime::instant::{Era, Instant, Duration};
    /// // Sub in the Present era.
    /// let tick = Instant::new(159, 10, Era::Present) - Duration::new(5, 2);
    /// assert_eq!(tick.secs(), 154);
    /// assert_eq!(tick.nanos(), 8);
    /// assert_eq!(tick.era(), Era::Present);

    /// // Sub in the Past era.
    /// let tick = Instant::new(159, 10, Era::Past) - Duration::new(5, 2);
    /// assert_eq!(tick.secs(), 164);
    /// assert_eq!(tick.nanos(), 12);
    /// assert_eq!(tick.era(), Era::Past);

    /// // Sub from the Present to overflow into the Past
    /// let tick = Instant::new(159, 0, Era::Present) - Duration::new(160, 0);
    /// assert_eq!(tick.secs(), 1);
    /// assert_eq!(tick.nanos(), 0);
    /// assert_eq!(tick.era(), Era::Past);

    /// let tick = Instant::new(0, 5, Era::Present) - Duration::new(0, 6);
    /// assert_eq!(tick.secs(), 0);
    /// assert_eq!(tick.nanos(), 1);
    /// assert_eq!(tick.era(), Era::Past);
    /// ```
    fn sub(self, delta: Duration) -> Instant {
        if delta.as_secs() == 0 && delta.subsec_nanos() == 0 {
            self
        } else {
            // Switch the era, an exact time of zero is in the Present era
            match self.era {
                Era::Past => {
                    // Subtracting a duration in the past is trivial
                    let mut cln = self;
                    cln.duration += delta;
                    cln
                }
                Era::Present => {
                    if (delta.as_secs() >= self.duration.as_secs())
                        || (delta.as_secs() >= self.duration.as_secs() && delta.as_secs() == 0
                            && delta.subsec_nanos() >= self.duration.subsec_nanos())
                    {
                        Instant::new(
                            delta.as_secs() - self.duration.as_secs(),
                            delta.subsec_nanos() - self.duration.subsec_nanos(),
                            Era::Past,
                        )
                    } else {
                        let mut cln = self;
                        cln.duration -= delta;
                        cln
                    }
                }
            }
        }
    }
}

#[test]
fn era_unittest() {
    assert_eq!(format!("{}", Era::Past), "Past");
    assert_eq!(format!("{}", Era::Present), "Present");
    assert!(Era::Past < Era::Present);
}

#[test]
fn instant_unittest() {
    // NOTE: These tests are copy-pasted into the documentation.
    // Add in the Present era.
    let tick = Instant::new(159, 10, Era::Present) + Duration::new(5, 2);
    assert_eq!(tick.secs(), 164);
    assert_eq!(tick.nanos(), 12);
    assert_eq!(tick.era(), Era::Present);

    // Add in the Past era.
    let tick = Instant::new(159, 10, Era::Past) + Duration::new(5, 2);
    assert_eq!(tick.secs(), 154);
    assert_eq!(tick.nanos(), 8);
    assert_eq!(tick.era(), Era::Past);

    // Add from the Past to overflow into the Present
    let tick = Instant::new(159, 0, Era::Past) + Duration::new(160, 0);
    assert_eq!(tick.secs(), 1);
    assert_eq!(tick.nanos(), 0);
    assert_eq!(tick.era(), Era::Present);

    let tick = Instant::new(0, 5, Era::Past) + Duration::new(0, 6);
    assert_eq!(tick.secs(), 0);
    assert_eq!(tick.nanos(), 1);
    assert_eq!(tick.era(), Era::Present);

    // Sub in the Present era.
    let tick = Instant::new(159, 10, Era::Present) - Duration::new(5, 2);
    assert_eq!(tick.secs(), 154);
    assert_eq!(tick.nanos(), 8);
    assert_eq!(tick.era(), Era::Present);

    // Sub in the Past era.
    let tick = Instant::new(159, 10, Era::Past) - Duration::new(5, 2);
    assert_eq!(tick.secs(), 164);
    assert_eq!(tick.nanos(), 12);
    assert_eq!(tick.era(), Era::Past);

    // Sub from the Present to overflow into the Past
    let tick = Instant::new(159, 0, Era::Present) - Duration::new(160, 0);
    assert_eq!(tick.secs(), 1);
    assert_eq!(tick.nanos(), 0);
    assert_eq!(tick.era(), Era::Past);

    let tick = Instant::new(0, 5, Era::Present) - Duration::new(0, 6);
    assert_eq!(tick.secs(), 0);
    assert_eq!(tick.nanos(), 1);
    assert_eq!(tick.era(), Era::Past);
}
