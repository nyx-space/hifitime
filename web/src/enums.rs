/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

//! The enumerations. A TypeScript enumeration holds no method, so each helper
//! is a free function.

use core::str::FromStr;

use hifitime as ht;
use wasm_bindgen::prelude::*;

use crate::error::js_err;

/// Declares the conversions between a wasm-bindgen enumeration and the
/// hifitime enumeration of the same name.
macro_rules! mirror_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        impl From<$name> for ht::$name {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => ht::$name::$variant,)+
                }
            }
        }

        impl From<ht::$name> for $name {
            #[allow(unreachable_patterns)]
            fn from(value: ht::$name) -> Self {
                match value {
                    $(ht::$name::$variant => $name::$variant,)+
                    // `hifitime::TimeScale` is `non_exhaustive`. A new variant
                    // must be added to this crate before it can cross the
                    // boundary.
                    other => unreachable!("{other:?} has no TypeScript binding"),
                }
            }
        }
    };
}

/// The time scale of an [`crate::epoch::Epoch`].
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimeScale {
    /// International Atomic Time, the internal representation of every epoch.
    TAI = 0,
    /// Terrestrial Time.
    TT = 1,
    /// Ephemeris Time, as defined by SPICE.
    ET = 2,
    /// Barycentric Dynamical Time.
    TDB = 3,
    /// Coordinated Universal Time.
    UTC = 4,
    /// GPS Time.
    GPST = 5,
    /// Galileo System Time.
    GST = 6,
    /// BeiDou Time.
    BDT = 7,
    /// Quasi-Zenith Satellite System Time.
    QZSST = 8,
    /// Geocentric Coordinate Time.
    TCG = 9,
    /// Barycentric Coordinate Time.
    TCB = 10,
    /// EXPERIMENTAL Lunar Time.
    TL = 11,
    /// EXPERIMENTAL Lunar Coordinate Time.
    TCL = 12,
}

mirror_enum!(TimeScale {
    TAI,
    TT,
    ET,
    TDB,
    UTC,
    GPST,
    GST,
    BDT,
    QZSST,
    TCG,
    TCB,
    TL,
    TCL
});

/// Time unit, used to convert a [`crate::duration::Duration`] to a number.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    Nanosecond = 0,
    Microsecond = 1,
    Millisecond = 2,
    Second = 3,
    Minute = 4,
    Hour = 5,
    Day = 6,
    Week = 7,
    /// 36525 days, the number of days per century in the Julian calendar.
    Century = 8,
}

mirror_enum!(Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Century
});

/// Day of the week. Monday is zero.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

mirror_enum!(Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
});

/// Name of the month. January is zero, as in the hifitime Rust API.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MonthName {
    January = 0,
    February = 1,
    March = 2,
    April = 3,
    May = 4,
    June = 5,
    July = 6,
    August = 7,
    September = 8,
    October = 9,
    November = 10,
    December = 11,
}

mirror_enum!(MonthName {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December
});

/// Parses a time scale from its name, for example `"UTC"` or `"GPST"`.
#[wasm_bindgen(js_name = parseTimeScale)]
pub fn parse_time_scale(name: &str) -> Result<TimeScale, JsError> {
    ht::TimeScale::from_str(name)
        .map(Into::into)
        .map_err(js_err)
}

/// Returns the name of the time scale, for example `"UTC"`.
#[wasm_bindgen(js_name = timeScaleName)]
pub fn time_scale_name(scale: TimeScale) -> String {
    ht::TimeScale::from(scale).to_string()
}

/// Returns true if the time scale belongs to a global navigation satellite system.
#[wasm_bindgen(js_name = isGnssTimeScale)]
pub fn is_gnss_time_scale(scale: TimeScale) -> bool {
    ht::TimeScale::from(scale).is_gnss()
}

/// Parses a day of the week from its name, for example `"Monday"` or `"Mon"`.
#[wasm_bindgen(js_name = parseWeekday)]
pub fn parse_weekday(name: &str) -> Result<Weekday, JsError> {
    ht::Weekday::from_str(name).map(Into::into).map_err(js_err)
}

/// Returns the name of the day of the week, for example `"Monday"`.
#[wasm_bindgen(js_name = weekdayName)]
pub fn weekday_name(weekday: Weekday) -> String {
    ht::Weekday::from(weekday).to_string()
}

/// Returns the number of the month, from 1 for January to 12 for December.
#[wasm_bindgen(js_name = monthNumber)]
pub fn month_number(month: MonthName) -> u8 {
    month as u8 + 1
}

/// Builds a month from its number, from 1 for January to 12 for December.
#[wasm_bindgen(js_name = monthFromNumber)]
pub fn month_from_number(month_number: u8) -> Result<MonthName, JsError> {
    if !(1..=12).contains(&month_number) {
        return Err(JsError::new(&format!(
            "month number {month_number} is not between 1 and 12"
        )));
    }
    Ok(ht::MonthName::from(month_number).into())
}

/// Returns the name of the month, for example `"January"`.
#[wasm_bindgen(js_name = monthName)]
pub fn month_name(month: MonthName) -> String {
    ht::MonthName::from(month).to_string()
}

/// Returns the number of seconds in one unit.
#[wasm_bindgen(js_name = unitInSeconds)]
pub fn unit_in_seconds(unit: Unit) -> f64 {
    ht::Unit::from(unit).in_seconds()
}
