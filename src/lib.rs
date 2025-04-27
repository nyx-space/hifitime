#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

/// Julian date for the J1900 epoch, as per NAIF SPICE.
pub const JD_J1900: f64 = 2_415_020.0;
/// Julian date for the J2000 epoch, as per NAIF SPICE.
pub const JD_J2000: f64 = 2_451_545.0;
/// Julian days between 01 Jan 1900 at midnight and the Modified Julian Day at 17 November 1858.
pub const MJD_J1900: f64 = 15_020.0;
/// Julian days between 01 Jan 2000 at **noon** and the Modified Julian Day at 17 November 1858.
pub const MJD_J2000: f64 = 51_544.5;
/// The Ephemeris Time epoch, in seconds
pub const ET_EPOCH_S: i64 = 3_155_716_800;
/// Modified Julian Date in seconds as defined [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
pub const MJD_OFFSET: f64 = 2_400_000.5;
/// `DAYS_PER_YEAR` corresponds to the number of days per year in the Julian calendar.
pub const DAYS_PER_YEAR: f64 = 365.25;
/// `DAYS_PER_YEAR_NLD` corresponds to the number of days per year **without leap days**.
pub const DAYS_PER_YEAR_NLD: f64 = 365.0;
/// `DAYS_PER_CENTURY` corresponds to the number of days per century in the Julian calendar.
pub const DAYS_PER_CENTURY: f64 = 36525.0;
pub const DAYS_PER_CENTURY_I64: i64 = 36525;
pub const DAYS_PER_WEEK: f64 = 7.0;
pub const DAYS_PER_WEEK_I64: i64 = 7;
/// `SECONDS_PER_MINUTE` defines the number of seconds per minute.
pub const SECONDS_PER_MINUTE: f64 = 60.0;
/// `SECONDS_PER_HOUR` defines the number of seconds per hour.
pub const SECONDS_PER_HOUR: f64 = 3_600.0;
/// `SECONDS_PER_DAY` defines the number of seconds per day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const SECONDS_PER_DAY_I64: i64 = 86_400;
/// `SECONDS_PER_CENTURY` defines the number of seconds per century.
pub const SECONDS_PER_CENTURY: f64 = SECONDS_PER_DAY * DAYS_PER_CENTURY;
/// `SECONDS_PER_YEAR` corresponds to the number of seconds per julian year from [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/jyear_c.html).
pub const SECONDS_PER_YEAR: f64 = 31_557_600.0;
pub const SECONDS_PER_YEAR_I64: i64 = 31_557_600;
/// `SECONDS_PER_TROPICAL_YEAR` corresponds to the number of seconds per tropical year from [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/tyear_c.html).
pub const SECONDS_PER_TROPICAL_YEAR: f64 = 31_556_925.974_7;
/// `SECONDS_PER_SIDEREAL_YEAR` corresponds to the number of seconds per sidereal year from [NIST](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9#TIME).
pub const SECONDS_PER_SIDEREAL_YEAR: f64 = 31_558_150.0;

// Epoch formatting module is called `efmt` to avoid collision with `std::fmt` and `core::fmt`.
pub mod efmt;
mod parser;

pub mod errors;
pub use errors::{DurationError, HifitimeError, ParsingError};

mod epoch;
pub use epoch::*;

mod duration;
pub use duration::*;

mod timescale;
pub use timescale::*;

mod timeunits;
pub use timeunits::*;

mod timeseries;
pub use timeseries::*;

mod weekday;
pub use weekday::*;

mod month;
pub use month::*;

mod polynomial;
pub use polynomial::Polynomial;

pub mod prelude {
    pub use crate::efmt::{Format, Formatter};
    pub use crate::{
        Duration, DurationError, Epoch, Freq, Frequencies, HifitimeError, ParsingError, TimeScale,
        TimeSeries, TimeUnits, Unit, Weekday,
    };
}

#[cfg(kani)]
mod kani_verif;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "std")]
extern crate core;
