use std::fmt::Display;
use std::cmp::PartialOrd;
use super::instant::Instant;
use super::utils::{Errors, Offset};

pub trait TimeSystem: PartialOrd {
    fn from_instant(Instant) -> Self;
    fn as_instant(self) -> Instant;
}

pub trait TimeZone: Display {
    /// utc_offset returns the difference between a given TZ and UTC.
    fn utc_offset() -> Offset;
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
}
