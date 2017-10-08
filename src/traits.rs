use std::fmt;
use std::time::Duration;
use super::instant::{Era, Instant};

pub trait TimeSystem {
    fn from_instant(Instant) -> Self;
    fn as_instant(self) -> Instant;
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

pub trait TimeZone {
    fn UTC_offset() -> Duration; // Returns the difference between a given TZ and UTC (Offset??)
    fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8, nanos: u8) -> Self;
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
