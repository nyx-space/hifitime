/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use hifitime as ht;
use wasm_bindgen::prelude::*;

/// The scientific constants of hifitime.
///
/// A count of nanoseconds is a `bigint`, because it exceeds the 53 significant
/// bits of a JavaScript `number`.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct Constants {
    /// Julian date of the J1900 epoch, as per NAIF SPICE.
    #[wasm_bindgen(js_name = jdJ1900)]
    pub jd_j1900: f64,
    /// Julian date of the J2000 epoch, as per NAIF SPICE.
    #[wasm_bindgen(js_name = jdJ2000)]
    pub jd_j2000: f64,
    /// Modified Julian days of the J1900 epoch.
    #[wasm_bindgen(js_name = mjdJ1900)]
    pub mjd_j1900: f64,
    /// Modified Julian days of the J2000 epoch.
    #[wasm_bindgen(js_name = mjdJ2000)]
    pub mjd_j2000: f64,
    /// The ephemeris time epoch, in seconds.
    #[wasm_bindgen(js_name = etEpochSeconds)]
    pub et_epoch_seconds: f64,
    /// Difference between a Julian date and a Modified Julian date.
    #[wasm_bindgen(js_name = mjdOffset)]
    pub mjd_offset: f64,
    /// Julian date of J1900 at noon.
    #[wasm_bindgen(js_name = jdJ1900Noon)]
    pub jd_j1900_noon: f64,
    /// Days per year in the Julian calendar.
    #[wasm_bindgen(js_name = daysPerYear)]
    pub days_per_year: f64,
    /// Days per year without a leap day.
    #[wasm_bindgen(js_name = daysPerYearNld)]
    pub days_per_year_nld: f64,
    /// Days per century in the Julian calendar.
    #[wasm_bindgen(js_name = daysPerCentury)]
    pub days_per_century: f64,
    #[wasm_bindgen(js_name = daysPerWeek)]
    pub days_per_week: f64,
    #[wasm_bindgen(js_name = secondsPerMinute)]
    pub seconds_per_minute: f64,
    #[wasm_bindgen(js_name = secondsPerHour)]
    pub seconds_per_hour: f64,
    #[wasm_bindgen(js_name = secondsPerDay)]
    pub seconds_per_day: f64,
    #[wasm_bindgen(js_name = secondsPerCentury)]
    pub seconds_per_century: f64,
    /// Seconds per Julian year, as per NAIF SPICE.
    #[wasm_bindgen(js_name = secondsPerYear)]
    pub seconds_per_year: f64,
    #[wasm_bindgen(js_name = secondsPerTropicalYear)]
    pub seconds_per_tropical_year: f64,
    #[wasm_bindgen(js_name = secondsPerSiderealYear)]
    pub seconds_per_sidereal_year: f64,
    #[wasm_bindgen(js_name = nanosecondsPerMicrosecond)]
    pub nanoseconds_per_microsecond: u64,
    #[wasm_bindgen(js_name = nanosecondsPerMillisecond)]
    pub nanoseconds_per_millisecond: u64,
    #[wasm_bindgen(js_name = nanosecondsPerSecond)]
    pub nanoseconds_per_second: u64,
    #[wasm_bindgen(js_name = nanosecondsPerMinute)]
    pub nanoseconds_per_minute: u64,
    #[wasm_bindgen(js_name = nanosecondsPerHour)]
    pub nanoseconds_per_hour: u64,
    #[wasm_bindgen(js_name = nanosecondsPerDay)]
    pub nanoseconds_per_day: u64,
    #[wasm_bindgen(js_name = nanosecondsPerCentury)]
    pub nanoseconds_per_century: u64,
}

/// Returns the scientific constants of hifitime.
#[wasm_bindgen]
pub fn constants() -> Constants {
    Constants {
        jd_j1900: ht::JD_J1900,
        jd_j2000: ht::JD_J2000,
        mjd_j1900: ht::MJD_J1900,
        mjd_j2000: ht::MJD_J2000,
        et_epoch_seconds: ht::ET_EPOCH_S as f64,
        mjd_offset: ht::MJD_OFFSET,
        jd_j1900_noon: ht::JD_J1900_NOON,
        days_per_year: ht::DAYS_PER_YEAR,
        days_per_year_nld: ht::DAYS_PER_YEAR_NLD,
        days_per_century: ht::DAYS_PER_CENTURY,
        days_per_week: ht::DAYS_PER_WEEK,
        seconds_per_minute: ht::SECONDS_PER_MINUTE,
        seconds_per_hour: ht::SECONDS_PER_HOUR,
        seconds_per_day: ht::SECONDS_PER_DAY,
        seconds_per_century: ht::SECONDS_PER_CENTURY,
        seconds_per_year: ht::SECONDS_PER_YEAR,
        seconds_per_tropical_year: ht::SECONDS_PER_TROPICAL_YEAR,
        seconds_per_sidereal_year: ht::SECONDS_PER_SIDEREAL_YEAR,
        nanoseconds_per_microsecond: ht::NANOSECONDS_PER_MICROSECOND,
        nanoseconds_per_millisecond: ht::NANOSECONDS_PER_MILLISECOND,
        nanoseconds_per_second: ht::NANOSECONDS_PER_SECOND,
        nanoseconds_per_minute: ht::NANOSECONDS_PER_MINUTE,
        nanoseconds_per_hour: ht::NANOSECONDS_PER_HOUR,
        nanoseconds_per_day: ht::NANOSECONDS_PER_DAY,
        nanoseconds_per_century: ht::NANOSECONDS_PER_CENTURY,
    }
}
