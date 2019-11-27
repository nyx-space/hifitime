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
//!  * Ephemeris Time (SPICE ET) / Barycentric Dynamical Time (IERS TDB) computations correct at least to 1e-5 seconds
//!
//! ## TODO
//!  * UTC representation with ISO8601 formatting (and parsing in that format)
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
//! hifitime = "1"
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
//! use hifitime::Epoch;
//!
//! let mut santa = Epoch::from_gregorian_utc(2017, 12, 25, 01, 02, 14, 0);
//! assert_eq!(santa.as_mjd_utc_days(), 58112.043217592596);
//! assert_eq!(santa.as_jde_utc_days(), 2458112.5432175924);
//!
//! santa.mut_add_secs(3600.0);
//! assert_eq!(
//!     santa,
//!     Epoch::from_gregorian_utc(2017, 12, 25, 02, 02, 14, 0),
//!     "Could not add one hour to Christmas"
//! );
//! ```
//!
//! ### Limitations
//! Barycentric Dynamical Time is computed using the [ESA Navipedia reference](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems).
//! In three separate examples, the error with SPICE Ephemeris Time is the following:
//!     * -9.536743e-07 seconds for 2012-Feb-7 11:22:33 UTC
//!     * -3.814697e-06 seconds for 2002-Feb-7 midnight UTC
//!     * -4.291534e-06 seconds for 1996-Feb-7 11:22:33 UTC
//!

/// `J1900_OFFSET` determines the offset in julian days between 01 Jan 1900 at midnight and the
/// Modified Julian Day at 17 November 1858.
/// NOTE: Julian days "start" at noon so that astronomical observations throughout the night
/// happen at the same Julian day. Note however that the Modified Julian Date (MJD) starts at
/// midnight, not noon, cf. <http://tycho.usno.navy.mil/mjd.html>.
pub const J1900_OFFSET: f64 = 15_020.0;
/// `J2000_OFFSET` determines the offset in julian days between 01 Jan 2000 at **noon** and the
/// Modified Julian Day at 17 November 1858.
pub const J2000_OFFSET: f64 = 51_544.5;
/// Modified Julian Date in seconds as defined [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
pub const MJD_OFFSET: f64 = 2_400_000.5;
/// `DAYS_PER_YEAR` corresponds to the number of days per year in the Julian calendar.
pub const DAYS_PER_YEAR: f64 = 365.25;
/// `SECONDS_PER_DAY` defines the number of seconds per day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
/// `SECONDS_PER_HOUR` defines the number of seconds per hour.
pub const SECONDS_PER_HOUR: f64 = 3_600.0;
/// `SECONDS_PER_MINUTE` defines the number of seconds per minute.
pub const SECONDS_PER_MINUTE: f64 = 60.0;
/// `SECONDS_PER_TROPICAL_YEAR` corresponds to the number of seconds per tropical year, as defined in `tyear_c.c` in [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/tyear_c.html).
pub const SECONDS_PER_TROPICAL_YEAR: f64 = 31_556_925.974_7;

mod sim;
pub use sim::ClockNoise;

mod epoch;

pub use epoch::*;

use std::convert;
use std::fmt;
use std::num::ParseIntError;

/// Errors handles all oddities which may occur in this library.
#[derive(Debug)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a call to `Datetime::new` receives 60 seconds and there are only 59 seconds in the provided
    /// date time then a Carry Error is returned as the Result.
    Carry,
    /// ParseError is returned when a provided string could not be parsed and converted to the desired
    /// struct (e.g. Datetime).
    ParseError(String),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Errors::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
            Errors::ParseError(ref msg) => write!(f, "ParseError: {}", msg),
        }
    }
}

impl convert::From<ParseIntError> for Errors {
    fn from(error: ParseIntError) -> Self {
        Errors::ParseError(format!("std::num::ParseIntError encountered: {}", error))
    }
}

#[test]
fn error_unittest() {
    assert_eq!(
        format!("{}", Errors::Carry),
        "a carry error (e.g. 61 seconds)"
    );
}
