//! # hifitime
//!
//! Precise date and time handling in Rust built on top of a simple f64.
//! The Epoch used is TAI Epoch of 01 Jan 1900 at midnight.
//!
//! ## Features
//!
//!  * Leap seconds (as announced by the IETF on a yearly basis)
//!  * Julian dates and Modified Julian dates
//!  * Clock drift via oscillator stability for simulation of time measuring hardware (via the `simulation` feature)
//!  * UTC representation with ISO8601 formatting (and parsing in that format #45)
//!  * High fidelity Ephemeris Time / Dynamic Barycentric Time (TDB) computations from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB) (caveat: up to 10ms difference with SPICE near 01 Jan 2000)
//!  * Trivial support of time arithmetic (e.g. `2 * TimeUnit::Hour + TimeUnit::Second * 3`)
//!  * Supports ranges of Epochs and TimeSeries (linspace of `Epoch`s and `Duration`s)
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
//! hifitime = "2"
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
//! #### Time creation
//! ```rust
//! use hifitime::{Epoch, TimeUnit};
//! use std::str::FromStr;
//!
//! let mut santa = Epoch::from_gregorian_utc(2017, 12, 25, 01, 02, 14, 0);
//! assert_eq!(santa.as_mjd_utc_days(), 58112.043217592590);
//! assert_eq!(santa.as_jde_utc_days(), 2458112.5432175924);
//!
//! santa += 3600 * TimeUnit::Second;
//! assert_eq!(
//!     santa,
//!     Epoch::from_gregorian_utc(2017, 12, 25, 02, 02, 14, 0),
//!     "Could not add one hour to Christmas"
//! );
//!
//! let dt = Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 0);
//! assert_eq!(dt, Epoch::from_str("2017-01-14T00:31:55 UTC").unwrap());
//! // And you can print it too, although by default it will print in UTC
//! assert_eq!(dt.as_gregorian_utc_str(), "2017-01-14T00:31:55 UTC".to_string());
//! assert_eq!(format!("{}", dt), "2017-01-14T00:31:55 UTC".to_string());
//! ```
//!
//! #### Time differences, time unit, and duration handling
//! Comparing times will lead to a Duration type. Printing that will automatically select the unit.
//! ```rust
//! use hifitime::{Epoch, TimeUnit, Duration};
//!
//! let at_midnight = Epoch::from_gregorian_utc_at_midnight(2020, 11, 2);
//! let at_noon = Epoch::from_gregorian_utc_at_noon(2020, 11, 2);
//! assert_eq!(at_noon - at_midnight, 12 * TimeUnit::Hour);
//! assert_eq!(at_noon - at_midnight, 1 * TimeUnit::Day / 2);
//! assert_eq!(at_midnight - at_noon, -1 * TimeUnit::Day / 2);
//!
//! let delta_time = at_noon - at_midnight;
//! // assert_eq!(format!("{}", delta_time), "12 h 0 min 0 s".to_string());
//! // And we can multiply durations by a scalar...
//! let delta2 = 2 * delta_time;
//! // assert_eq!(format!("{}", delta2), "1 days 0 h 0 min 0 s".to_string());
//! // Or divide them by a scalar.
//! // assert_eq!(format!("{}", delta2 / 2.0), "12 h 0 min 0 s".to_string());
//!
//! // And of course, these comparisons account for differences in time systems
//! let at_midnight_utc = Epoch::from_gregorian_utc_at_midnight(2020, 11, 2);
//! let at_noon_tai = Epoch::from_gregorian_tai_at_noon(2020, 11, 2);
//! // assert_eq!(format!("{}", at_noon_tai - at_midnight_utc), "11 h 59 min 23 s".to_string());
//! ```
//!
//! #### Iterating over times ("linspace" of epochs)
//! Finally, something which may come in very handy, line spaces between times with a given step.
//!
//! ```rust
//! use hifitime::{Epoch, TimeUnit, TimeSeries};
//! let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
//! let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
//! let step = 2 * TimeUnit::Hour;
//! let time_series = TimeSeries::inclusive(start, end, step);
//! let mut cnt = 0;
//! for epoch in time_series {
//!     println!("{}", epoch);
//!     cnt += 1
//! }
//! // Check that there are indeed six two-hour periods in a half a day,
//! // including start and end times.
//! assert_eq!(cnt, 7)
//! ```
//!
//! ### Limitations
//! Barycentric Dynamical Time is computed using the [ESA Navipedia reference](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems).
//! In three separate examples, the error with SPICE Ephemeris Time is the following:
//!     * -9.536743e-07 seconds for 2012-Feb-7 11:22:33 UTC
//!     * -3.814697e-06 seconds for 2002-Feb-7 midnight UTC
//!     * -4.291534e-06 seconds for 1996-Feb-7 11:22:33 UTC
//!

pub const J1900_NAIF: f64 = 2_415_020.0;
pub const J2000_NAIF: f64 = 2_451_545.0;
/// `J1900_OFFSET` determines the offset in julian days between 01 Jan 1900 at midnight and the
/// Modified Julian Day at 17 November 1858.
/// NOTE: Julian days "start" at noon so that astronomical observations throughout the night
/// happen at the same Julian day. Note however that the Modified Julian Date (MJD) starts at
/// midnight, not noon, cf. <http://tycho.usno.navy.mil/mjd.html>.
pub const J1900_OFFSET: f64 = 15_020.0;
/// `J2000_OFFSET` determines the offset in julian days between 01 Jan 2000 at **noon** and the
/// Modified Julian Day at 17 November 1858.
pub const J2000_OFFSET: f64 = 51_544.5;
/// The Ephemeris Time epoch, in seconds
pub const ET_EPOCH_S: f64 = 3_155_716_800.0;
/// Modified Julian Date in seconds as defined [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
pub const MJD_OFFSET: f64 = 2_400_000.5;
/// The JDE offset in days
pub const JDE_OFFSET_DAYS: f64 = J1900_OFFSET + MJD_OFFSET;
/// The JDE offset in seconds
pub const JDE_OFFSET_SECONDS: f64 = JDE_OFFSET_DAYS * SECONDS_PER_DAY;
/// `DAYS_PER_YEAR` corresponds to the number of days per year in the Julian calendar.
pub const DAYS_PER_YEAR: f64 = 365.25;
/// `DAYS_PER_CENTURY` corresponds to the number of days per centuy in the Julian calendar.
pub const DAYS_PER_CENTURY: f64 = 36525.0;
/// `SECONDS_PER_MINUTE` defines the number of seconds per minute.
pub const SECONDS_PER_MINUTE: f64 = 60.0;
/// `SECONDS_PER_HOUR` defines the number of seconds per hour.
pub const SECONDS_PER_HOUR: f64 = 3_600.0;
/// `SECONDS_PER_DAY` defines the number of seconds per day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
/// `SECONDS_PER_YEAR` corresponds to the number of seconds per julian year from [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/jyear_c.html).
pub const SECONDS_PER_YEAR: f64 = 31_557_600.0;
/// `SECONDS_PER_TROPICAL_YEAR` corresponds to the number of seconds per tropical year from [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/tyear_c.html).
pub const SECONDS_PER_TROPICAL_YEAR: f64 = 31_556_925.974_7;
/// `SECONDS_PER_SIDERAL_YEAR` corresponds to the number of seconds per sideral year from [NIST](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9#TIME).
pub const SECONDS_PER_SIDERAL_YEAR: f64 = 31_558_150.0;

mod sim;
pub use sim::ClockNoise;

mod epoch;

pub use epoch::*;

mod duration;

pub use duration::*;

mod timeseries;
pub use timeseries::*;

pub mod prelude {
    pub use {Duration, Epoch, TimeSeries, TimeUnit, TimeUnitHelper};
}

use std::convert;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Errors handles all oddities which may occur in this library.
#[derive(Clone, Debug, PartialEq)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a call to `Datetime::new` receives 60 seconds and there are only 59 seconds in the provided
    /// date time then a Carry Error is returned as the Result.
    Carry,
    /// ParseError is returned when a provided string could not be parsed and converted to the desired
    /// struct (e.g. Datetime).
    ParseError(String),
    /// Raised when trying to initialize an Epoch or Duration from its hi and lo values, but these overlap
    ConversionOverlapError(f64, f64),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Errors::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
            Errors::ParseError(ref msg) => write!(f, "ParseError: {}", msg),
            Errors::ConversionOverlapError(hi, lo) => {
                write!(f, "hi and lo values overlap: {}, {}", hi, lo)
            }
        }
    }
}

impl convert::From<ParseIntError> for Errors {
    fn from(error: ParseIntError) -> Self {
        Errors::ParseError(format!("std::num::ParseIntError encountered: {}", error))
    }
}

impl Error for Errors {}

/// Enum of the different time systems available
#[derive(Debug, PartialEq)]
pub enum TimeSystem {
    /// Ephemeris Time as defined by SPICE (slightly different from true TDB)
    ET,
    /// TAI is the representation of an Epoch internally
    TAI,
    /// Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    TT,
    /// Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time)
    TDB,
    UTC,
}

impl FromStr for TimeSystem {
    type Err = Errors;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        if val == "UTC" {
            Ok(TimeSystem::UTC)
        } else if val == "TT" {
            Ok(TimeSystem::TT)
        } else if val == "TAI" {
            Ok(TimeSystem::TAI)
        } else if val == "TDB" {
            Ok(TimeSystem::TDB)
        } else if val == "ET" {
            Ok(TimeSystem::ET)
        } else {
            Err(Errors::ParseError(format!("unknown time system `{}`", val)))
        }
    }
}

#[test]
fn error_unittest() {
    assert_eq!(
        format!("{}", Errors::Carry),
        "a carry error (e.g. 61 seconds)"
    );
}
