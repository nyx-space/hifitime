#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

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
pub const ET_EPOCH_S: i64 = 3_155_716_800;
/// Modified Julian Date in seconds as defined [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
pub const MJD_OFFSET: f64 = 2_400_000.5;
/// The JDE offset in days
pub const JDE_OFFSET_DAYS: f64 = J1900_OFFSET + MJD_OFFSET;
/// The JDE offset in seconds
pub const JDE_OFFSET_SECONDS: f64 = JDE_OFFSET_DAYS * SECONDS_PER_DAY;
/// `DAYS_PER_YEAR` corresponds to the number of days per year in the Julian calendar.
pub const DAYS_PER_YEAR: f64 = 365.25;
/// `DAYS_PER_YEAR_NLD` corresponds to the number of days per year **without leap days**.
pub const DAYS_PER_YEAR_NLD: f64 = 365.0;
/// `DAYS_PER_CENTURY` corresponds to the number of days per century in the Julian calendar.
pub const DAYS_PER_CENTURY: f64 = 36525.0;
pub const DAYS_PER_CENTURY_I64: i64 = 36525;
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
/// `SECONDS_PER_SIDERAL_YEAR` corresponds to the number of seconds per sidereal year from [NIST](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9#TIME).
#[deprecated(
    since = "3.8.0",
    note = "Use SECONDS_PER_SIDEREAL_YEAR instead (does not have the typo)"
)]
pub const SECONDS_PER_SIDERAL_YEAR: f64 = 31_558_150.0;
/// `SECONDS_PER_SIDEREAL_YEAR` corresponds to the number of seconds per sidereal year from [NIST](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9#TIME).
pub const SECONDS_PER_SIDEREAL_YEAR: f64 = 31_558_150.0;

/// The duration between J2000 and J1900: one century **minus** twelve hours. J1900 starts at _noon_ but J2000 is at midnight.
pub const J2000_TO_J1900_DURATION: Duration = Duration {
    centuries: 0,
    nanoseconds: 3_155_716_800_000_000_000,
};

/// The Ephemeris Time reference epoch J2000.
pub const J2000_REF_EPOCH_ET: Epoch = Epoch {
    duration_since_j1900_tai: Duration {
        centuries: 0,
        nanoseconds: 3_155_716_767_816_072_748,
    },
    time_scale: TimeScale::ET,
};

/// The Dynamic Barycentric Time reference epoch J2000.
pub const J2000_REF_EPOCH_TDB: Epoch = Epoch {
    duration_since_j1900_tai: Duration {
        centuries: 0,
        nanoseconds: 3_155_716_767_816_072_704,
    },
    time_scale: TimeScale::ET,
};

// Epoch formatting module is called `efmt` to avoid collision with `std::fmt` and `core::fmt`.
pub mod efmt;
mod parser;

pub mod errors;
pub use errors::{Errors, ParsingErrors};

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

pub mod leap_seconds;

#[cfg(feature = "std")]
mod leap_seconds_file;

#[cfg(feature = "ut1")]
pub mod ut1;

/// This module defines all of the deprecated methods.
mod deprecated;

#[allow(deprecated)]
pub mod prelude {
    pub use crate::efmt::{Format, Formatter};
    pub use crate::{
        deprecated::TimeSystem, Duration, Epoch, Errors, Freq, Frequencies, TimeScale, TimeSeries,
        TimeUnits, Unit, Weekday,
    };
}

#[cfg(feature = "asn1der")]
pub mod asn1der;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "std")]
extern crate core;
