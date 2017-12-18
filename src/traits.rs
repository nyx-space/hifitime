// use std::fmt;
use super::instant::Instant;
use super::utils::{Errors, Offset};

pub trait TimeSystem {
    fn from_instant(Instant) -> Self;
    fn as_instant(self) -> Instant;
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

pub trait TimeZone {
    fn utc_offset() -> Offset; // Returns the difference between a given TZ and UTC
    fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Self, Errors>
    where
        Self: Sized;
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
