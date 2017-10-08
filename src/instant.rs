// Disclamer: this is heavily inspired by std::time::Duration, but it supports longer
// time spans and leap seconds. Moreover, an Instant is defined with respect to
// 01 Jan 1900, as per NTP specifications.

use std::ops::{Add, Sub};
use std::time::Duration;
use std::fmt;

/// The number of nanoseconds in seconds.
const NANOS_PER_SECOND: u32 = 1_000_000_000;

const NTP_EPOCH: Instant = Instant {
    seconds: 0,
    nanos: 0,
    era: Era::Present,
};

#[derive(Clone, Debug)]
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

// Stores the duration since 01 Jan 1900
#[derive(Clone, Debug)]
pub struct Instant {
    // TODO: Use std::Duration here and make it clean with the new call
    seconds: u64,
    nanos: u32,
    era: Era,
}

impl Instant {
    pub fn new(seconds: u64, nanos: u32, era: Era) -> Instant {
        let (extra_seconds, nanos) = div_mod_floor_64(nanos, NANOS_PER_SECOND);
        Instant {
            seconds: seconds + extra_seconds,
            nanos: nanos,
            era: era,
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, delta: Duration) -> Instant {
        let direction = match delta.num_seconds() > 0 {
            true => 1,
            false => -1,
        };
        if self.era == Era::Present {
            if delta.num_seconds() > 0 {
                let mut cln = self.clone();
                cln.seconds += delta.num_seconds();
                cln.nanos += delta.num_nanoseconds();
                return cln;
            }
        }
    }
}

// Mostly copied from Duration / libnum
fn div_mod_floor_64(this: u64, other: u64) -> (u64, u64) {
    (div_floor_64(this, other), mod_floor_64(this, other))
}

fn div_floor_64(this: u64, other: u64) -> u64 {
    match (this / other, this % other) {
        (d, r) if (r > 0 && other < 0) || (r < 0 && other > 0) => d - 1,
        (d, _) => d,
    }
}

fn mod_floor_64(this: u64, other: u64) -> u64 {
    match this % other {
        r if (r > 0 && other < 0) || (r < 0 && other > 0) => r + other,
        r => r,
    }
}
