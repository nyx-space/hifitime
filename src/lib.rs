//! # hifitime 0.0.1
//!
//! Precise date and time handling in Rust built on top of
//! [` std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html).
//! The Epoch used is TAI Epoch of 01 Jan 1900 at midnight, but that should not matter in
//! day-to-day use of this library.
//!
//! ## Features
//!
//!  * Leap seconds (as announced by the IETF on a yearly basis)
//!  * Julian dates and Modified Julian dates
//!  * UTC representation with ISO8601 formatting
//!  * Allows building custom `TimeSystem` (e.g. Julian days)
//!  * Simple to use `Offset`s to represent fixed or time-varying UTC offsets (e.g. for very high speed reference frames)
//!
//! Almost all examples are validated with external references, as detailed on a test-by-test
//! basis.
//!
//! ### Leap second support
//! Each time computing library may decide when the extra leap second exists as explained
//! in the [IETF leap second reference](https://www.ietf.org/timezones/data/leap-seconds.list).
//! To ease computation, `hifitime` decides that second is the 60th of a UTC date, if such exists.
//! Note that this second exists at a different time than defined on
//! [NASA HEASARC](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?). That tool is
//! used for validation of Julian dates. As an example of how this is handled, check the Julian
//! day computations for [2015-06-30 23:59:59](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes),
//! [2015-06-30 23:59:60](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes)
//! and [2015-07-01 00:00:00](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes).
//!
//! ## Does not include
//!
//! * Dates only, or times only (i.e. handles only the combination of both), but the `Datetime::{at_midnight, at_noon}` help
//! * Custom formatting of date time objects
//! * An initializer from machine time
//!
//! ## Usage
//!
//! Put this in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hifitime = "0.0.1"
//! ```
//!
//! And add the following to your crate root:
//!
//! ```rust
//! extern crate hifitime;
//! ```
//!
//! ### Examples:
//!
//! ```rust
//! use hifitime::datetime::{Datetime, TimeSystem};
//! use hifitime::instant::Duration;
//! use hifitime::julian::ModifiedJulian;
//!
//! let santa = Datetime::new(2017, 12, 25, 01, 02, 14, 0).expect("Xmas failed");
//!
//! assert_eq!(
//!     santa.into_instant() + Duration::new(3600, 0),
//!     Datetime::new(2017, 12, 25, 02, 02, 14, 0)
//!         .expect("Xmas failed")
//!         .into_instant(),
//!     "Could not add one hour to Christmas"
//! );
//! assert_eq!(format!("{}", santa), "2017-12-25T01:02:14+00:00");
//! assert_eq!(
//!     ModifiedJulian::from_instant(santa.into_instant()).days,
//!     58112.043217592596
//! );
//! assert_eq!(
//!     ModifiedJulian::from_instant(santa.into_instant()).julian_days(),
//!     2458112.5432175924
//! );
//! ```
//!

/// The `instant` module is built on top of `std::time::Duration`. It is the basis of almost
/// all computations in this library. It is the only common denominator allowing for conversions
/// between Time Systems.
pub mod instant;
/// The `julian` module supports (Modified) Julian Days, which are heavily used in astronomy
/// and its engineering friends.
pub mod julian;
/// The `datetime` module supports conversions between seconds past TAI epoch and a Datetime struct.
/// The main advantage (and challenge) is the inherent support for leap seconds. Refer to module
/// documentation for leap second implementation details.
pub mod datetime;

#[cfg(feature = "simulation")]
pub mod sim;

use std::cmp::PartialOrd;
use instant::Instant;
use std::fmt;

/// A `TimeSystem` enables the creation of system for measuring spans of time, such as UTC or Julian
/// days.
pub trait TimeSystem: PartialOrd {
    /// Use this method to convert between different `TimeSystem` implementors.
    fn from_instant(Instant) -> Self;
    /// Also use this method to convert between different `TimeSystem` implementors
    fn into_instant(self) -> Instant;
}

/// Errors handles all oddities which may occur in this library.
#[derive(Debug)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a call to `Datetime::new` receives 60 seconds and there are only 59 seconds in the provided
    /// date time then a Carry Error is returned as the Result.
    Carry,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Errors::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
        }
    }
}

/// The `durations` module provides helpers for initializing an `std::time::Duration`.
pub mod durations {
    use std::time::Duration;
    /// Returns a duration from the while number of days requested.
    pub fn from_days(days: u64) -> Duration {
        Duration::new(86_400 * days, 0)
    }
    /// Returns a duration from the while number of hours requested.
    pub fn from_hours(hours: u64) -> Duration {
        Duration::new(3600 * hours, 0)
    }
    /// Returns a duration from the while number of minutes requested.
    pub fn from_mins(mins: u64) -> Duration {
        Duration::new(60 * mins, 0)
    }
}

#[test]
fn error_unittest() {
    assert_eq!(
        format!("{}", Errors::Carry),
        "a carry error (e.g. 61 seconds)"
    );
}

#[test]
fn durations() {
    assert_eq!(durations::from_days(10).as_secs(), 864_000);
    assert_eq!(durations::from_hours(10).as_secs(), 36000);
    assert_eq!(durations::from_mins(10).as_secs(), 600);
}
