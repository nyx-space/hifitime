/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use core::cmp::Ordering;
use core::str::FromStr;

use hifitime as ht;
use hifitime::efmt::{Format, Formatter};
use wasm_bindgen::prelude::*;

use crate::duration::{Duration, DurationParts};
use crate::enums::{MonthName, TimeScale, Unit, Weekday};
use crate::error::js_err;
use crate::polynomial::Polynomial;

/// A specific instant in time, stored as an exact duration since 1900-01-01
/// 00:00:00 TAI, together with the time scale used for display and for parsing.
///
/// Every arithmetic operation happens in TAI, so a conversion between time
/// scales never loses precision. Methods that return a `number` can lose
/// precision, because a JavaScript `number` holds 53 significant bits. Prefer
/// `toDurationSinceJ1900()` and `Duration.totalNanoseconds()` for exact values.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Epoch(pub(crate) ht::Epoch);

impl From<ht::Epoch> for Epoch {
    fn from(value: ht::Epoch) -> Self {
        Self(value)
    }
}

/// A Gregorian date and time in one time scale.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct GregorianDate {
    pub year: i32,
    /// Month of the year, from 1 for January to 12 for December.
    pub month: u8,
    /// Day of the month, from 1.
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: u32,
}

impl From<(i32, u8, u8, u8, u8, u8, u32)> for GregorianDate {
    fn from(
        (year, month, day, hour, minute, second, nanosecond): (i32, u8, u8, u8, u8, u8, u32),
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
        }
    }
}

/// The position of an epoch inside a GNSS week.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct TimeOfWeek {
    /// Number of weeks since the reference epoch of the time scale.
    pub week: u32,
    /// Number of nanoseconds into the week.
    pub nanoseconds: u64,
}

#[wasm_bindgen]
impl Epoch {
    /// Parses an epoch. The string may hold a Gregorian date such as
    /// `"2024-02-29T12:34:56.789 UTC"`, or a formatted value such as
    /// `"MJD 51544.5 TAI"`, `"JD 2451545.0 TT"`, or `"SEC 66.28 TDB"`.
    ///
    /// The time scale defaults to UTC when the string does not name one.
    #[wasm_bindgen(constructor)]
    pub fn new(epoch_str: &str) -> Result<Epoch, JsError> {
        Self::from_string(epoch_str)
    }

    /// Parses an epoch. It behaves like the constructor.
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(epoch_str: &str) -> Result<Epoch, JsError> {
        ht::Epoch::from_str(epoch_str).map(Self).map_err(js_err)
    }

    /// Parses an epoch with an explicit format, for example
    /// `strptime("2024-02-29 12:34:56", "%Y-%m-%d %H:%M:%S")`.
    pub fn strptime(epoch_str: &str, format_str: &str) -> Result<Epoch, JsError> {
        ht::Epoch::from_format_str(epoch_str, format_str)
            .map(Self)
            .map_err(js_err)
    }

    /// Returns the current time, read from the system clock. In a browser it
    /// uses `Date.now()` through the `web-time` shim.
    ///
    /// The system clock reports UTC, so the returned epoch is in UTC.
    pub fn now() -> Result<Epoch, JsError> {
        ht::Epoch::now().map(Self).map_err(js_err)
    }

    /// The J1900 reference epoch: 1900-01-01 at noon TAI.
    pub fn j1900() -> Epoch {
        Self(ht::J1900_REF_EPOCH)
    }

    /// The J2000 reference epoch: 2000-01-01 at noon TAI.
    pub fn j2000() -> Epoch {
        Self(ht::J2000_REF_EPOCH)
    }

    /// The reference epoch of the time scale, expressed in that time scale.
    #[wasm_bindgen(js_name = referenceEpoch)]
    pub fn reference_epoch(scale: TimeScale) -> Epoch {
        Self(ht::TimeScale::from(scale).reference_epoch())
    }

    /// Builds an epoch from a Gregorian date and time in UTC. The month is one
    /// indexed. It throws if the date does not exist.
    #[wasm_bindgen(js_name = fromGregorianUtc)]
    pub fn from_gregorian_utc(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Epoch, JsError> {
        ht::Epoch::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanosecond)
            .map(Self)
            .map_err(js_err)
    }

    /// Builds an epoch from a Gregorian date and time in TAI. The month is one
    /// indexed. It throws if the date does not exist.
    #[wasm_bindgen(js_name = fromGregorianTai)]
    pub fn from_gregorian_tai(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Epoch, JsError> {
        ht::Epoch::maybe_from_gregorian_tai(year, month, day, hour, minute, second, nanosecond)
            .map(Self)
            .map_err(js_err)
    }

    /// Builds an epoch from a Gregorian date and time in one time scale. The
    /// month is one indexed. It throws if the date does not exist.
    #[wasm_bindgen(js_name = fromGregorian)]
    pub fn from_gregorian(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        scale: TimeScale,
    ) -> Result<Epoch, JsError> {
        ht::Epoch::maybe_from_gregorian(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
            scale.into(),
        )
        .map(Self)
        .map_err(js_err)
    }

    /// Builds an epoch at midnight of the Gregorian date in one time scale.
    #[wasm_bindgen(js_name = fromGregorianAtMidnight)]
    pub fn from_gregorian_at_midnight(
        year: i32,
        month: u8,
        day: u8,
        scale: TimeScale,
    ) -> Result<Epoch, JsError> {
        Self::from_gregorian(year, month, day, 0, 0, 0, 0, scale)
    }

    /// Builds an epoch at noon of the Gregorian date in one time scale.
    #[wasm_bindgen(js_name = fromGregorianAtNoon)]
    pub fn from_gregorian_at_noon(
        year: i32,
        month: u8,
        day: u8,
        scale: TimeScale,
    ) -> Result<Epoch, JsError> {
        Self::from_gregorian(year, month, day, 12, 0, 0, 0, scale)
    }

    /// Builds an epoch from a duration since the reference epoch of the time scale.
    #[wasm_bindgen(js_name = fromDuration)]
    pub fn from_duration(duration: &Duration, scale: TimeScale) -> Epoch {
        Self(ht::Epoch::from_duration(duration.0, scale.into()))
    }

    /// Builds an epoch from the number of seconds in one time scale, counted
    /// from the reference epoch of that time scale.
    #[wasm_bindgen(js_name = fromSecondsInTimeScale)]
    pub fn from_seconds_in_time_scale(seconds: f64, scale: TimeScale) -> Epoch {
        Self(ht::Epoch::from_duration(
            ht::Duration::from_seconds(seconds),
            scale.into(),
        ))
    }

    /// Builds an epoch from a duration since 1900-01-01 00:00:00 TAI.
    #[wasm_bindgen(js_name = fromTaiDuration)]
    pub fn from_tai_duration(duration: &Duration) -> Epoch {
        Self(ht::Epoch::from_tai_duration(duration.0))
    }

    /// Builds an epoch from a century count and a nanosecond count into that
    /// century, both since 1900-01-01 00:00:00 TAI. This constructor is exact.
    #[wasm_bindgen(js_name = fromTaiParts)]
    pub fn from_tai_parts(centuries: i16, nanoseconds: u64) -> Epoch {
        Self(ht::Epoch::from_tai_parts(centuries, nanoseconds))
    }

    #[wasm_bindgen(js_name = fromTaiSeconds)]
    pub fn from_tai_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_tai_seconds(seconds))
    }

    #[wasm_bindgen(js_name = fromTaiDays)]
    pub fn from_tai_days(days: f64) -> Epoch {
        Self(ht::Epoch::from_tai_days(days))
    }

    #[wasm_bindgen(js_name = fromUtcSeconds)]
    pub fn from_utc_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_utc_seconds(seconds))
    }

    #[wasm_bindgen(js_name = fromUtcDays)]
    pub fn from_utc_days(days: f64) -> Epoch {
        Self(ht::Epoch::from_utc_days(days))
    }

    #[wasm_bindgen(js_name = fromTtSeconds)]
    pub fn from_tt_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_tt_seconds(seconds))
    }

    /// Builds an epoch from the number of ephemeris seconds since J2000.
    #[wasm_bindgen(js_name = fromEtSeconds)]
    pub fn from_et_seconds(seconds_since_j2000: f64) -> Epoch {
        Self(ht::Epoch::from_et_seconds(seconds_since_j2000))
    }

    /// Builds an epoch from the number of barycentric dynamical seconds since J2000.
    #[wasm_bindgen(js_name = fromTdbSeconds)]
    pub fn from_tdb_seconds(seconds_since_j2000: f64) -> Epoch {
        Self(ht::Epoch::from_tdb_seconds(seconds_since_j2000))
    }

    /// Builds an epoch from a Modified Julian Date in one time scale.
    #[wasm_bindgen(js_name = fromMjd)]
    pub fn from_mjd(days: f64, scale: TimeScale) -> Epoch {
        Self(ht::Epoch::from_mjd_in_time_scale(days, scale.into()))
    }

    /// Builds an epoch from a Julian Date ephemeris in one time scale.
    #[wasm_bindgen(js_name = fromJde)]
    pub fn from_jde(days: f64, scale: TimeScale) -> Epoch {
        Self(ht::Epoch::from_jde_in_time_scale(days, scale.into()))
    }

    #[wasm_bindgen(js_name = fromGpstSeconds)]
    pub fn from_gpst_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_gpst_seconds(seconds))
    }

    #[wasm_bindgen(js_name = fromGpstNanoseconds)]
    pub fn from_gpst_nanoseconds(nanoseconds: u64) -> Epoch {
        Self(ht::Epoch::from_gpst_nanoseconds(nanoseconds))
    }

    #[wasm_bindgen(js_name = fromQzsstSeconds)]
    pub fn from_qzsst_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_qzsst_seconds(seconds))
    }

    #[wasm_bindgen(js_name = fromQzsstNanoseconds)]
    pub fn from_qzsst_nanoseconds(nanoseconds: u64) -> Epoch {
        Self(ht::Epoch::from_qzsst_nanoseconds(nanoseconds))
    }

    #[wasm_bindgen(js_name = fromGstSeconds)]
    pub fn from_gst_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_gst_seconds(seconds))
    }

    #[wasm_bindgen(js_name = fromGstNanoseconds)]
    pub fn from_gst_nanoseconds(nanoseconds: u64) -> Epoch {
        Self(ht::Epoch::from_gst_nanoseconds(nanoseconds))
    }

    #[wasm_bindgen(js_name = fromBdtSeconds)]
    pub fn from_bdt_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_bdt_seconds(seconds))
    }

    #[wasm_bindgen(js_name = fromBdtNanoseconds)]
    pub fn from_bdt_nanoseconds(nanoseconds: u64) -> Epoch {
        Self(ht::Epoch::from_bdt_nanoseconds(nanoseconds))
    }

    /// Builds an epoch from a Precision Time Protocol duration, which counts
    /// from 1970-01-01 00:00:00 TAI.
    #[wasm_bindgen(js_name = fromPtpNanoseconds)]
    pub fn from_ptp_nanoseconds(nanoseconds: u64) -> Epoch {
        Self(ht::Epoch::from_ptp_nanoseconds(nanoseconds))
    }

    /// Builds an epoch from a Unix timestamp in seconds.
    #[wasm_bindgen(js_name = fromUnixSeconds)]
    pub fn from_unix_seconds(seconds: f64) -> Epoch {
        Self(ht::Epoch::from_unix_seconds(seconds))
    }

    /// Builds an epoch from a Unix timestamp in milliseconds, which is what
    /// `Date.now()` returns. A whole number of milliseconds stays exact.
    #[wasm_bindgen(js_name = fromUnixMilliseconds)]
    pub fn from_unix_milliseconds(milliseconds: f64) -> Epoch {
        // `hifitime::Epoch::from_unix_milliseconds` multiplies a float, so
        // 1709210096789 ms lands one nanosecond short. Convert the whole part
        // with integer math, and only the fraction as a float.
        let whole = milliseconds.trunc();
        let fraction = milliseconds - whole;
        let duration = ht::Duration::from_total_nanoseconds(whole as i128 * 1_000_000)
            + ht::Duration::from_nanoseconds(fraction * 1e6);
        Self(ht::Epoch::from_unix_duration(duration))
    }

    /// Builds an epoch from a Unix timestamp in nanoseconds. This constructor is
    /// exact.
    #[wasm_bindgen(js_name = fromUnixNanoseconds)]
    pub fn from_unix_nanoseconds(nanoseconds: i128) -> Epoch {
        Self(ht::Epoch::from_unix_duration(
            ht::Duration::from_total_nanoseconds(nanoseconds),
        ))
    }

    /// Builds an epoch from a JavaScript `Date`. The result has millisecond
    /// precision, because that is the precision of `Date`.
    #[wasm_bindgen(js_name = fromDate)]
    pub fn from_date(date: &js_sys::Date) -> Epoch {
        Self::from_unix_milliseconds(date.get_time())
    }

    /// Builds an epoch from a position inside a GNSS week.
    #[wasm_bindgen(js_name = fromTimeOfWeek)]
    pub fn from_time_of_week(week: u32, nanoseconds: u64, scale: TimeScale) -> Epoch {
        Self(ht::Epoch::from_time_of_week(
            week,
            nanoseconds,
            scale.into(),
        ))
    }

    /// Builds an epoch from a day of the year, where 1 January is day 1.
    #[wasm_bindgen(js_name = fromDayOfYear)]
    pub fn from_day_of_year(year: i32, days: f64, scale: TimeScale) -> Epoch {
        Self(ht::Epoch::from_day_of_year(year, days, scale.into()))
    }

    /// The time scale used to display and to parse this epoch.
    #[wasm_bindgen(getter, js_name = timeScale)]
    pub fn time_scale(&self) -> TimeScale {
        self.0.time_scale.into()
    }

    /// The year in the time scale of this epoch.
    #[wasm_bindgen(getter)]
    pub fn year(&self) -> i32 {
        self.0.year()
    }

    /// The name of the month.
    #[wasm_bindgen(getter, js_name = monthName)]
    pub fn month_name(&self) -> MonthName {
        self.0.month_name().into()
    }

    /// The day of the month, from 1.
    #[wasm_bindgen(getter, js_name = dayOfMonth)]
    pub fn day_of_month(&self) -> u8 {
        self.0.day_of_month()
    }

    /// The day of the year, where 1 January is day 1. The fraction of the value
    /// is the fraction of the day.
    #[wasm_bindgen(getter, js_name = dayOfYear)]
    pub fn day_of_year(&self) -> f64 {
        self.0.day_of_year()
    }

    /// The hour of the day, from 0 to 23.
    #[wasm_bindgen(getter)]
    pub fn hours(&self) -> u32 {
        self.0.hours() as u32
    }

    /// The minute of the hour, from 0 to 59.
    #[wasm_bindgen(getter)]
    pub fn minutes(&self) -> u32 {
        self.0.minutes() as u32
    }

    /// The second of the minute, from 0 to 59, or 60 during a leap second.
    #[wasm_bindgen(getter)]
    pub fn seconds(&self) -> u32 {
        self.0.seconds() as u32
    }

    /// The milliseconds into the current second.
    #[wasm_bindgen(getter)]
    pub fn milliseconds(&self) -> u32 {
        self.0.milliseconds() as u32
    }

    /// The microseconds into the current millisecond.
    #[wasm_bindgen(getter)]
    pub fn microseconds(&self) -> u32 {
        self.0.microseconds() as u32
    }

    /// The nanoseconds into the current microsecond.
    #[wasm_bindgen(getter)]
    pub fn nanoseconds(&self) -> u32 {
        self.0.nanoseconds() as u32
    }

    /// Returns this epoch in another time scale. The instant does not change.
    #[wasm_bindgen(js_name = toTimeScale)]
    pub fn to_time_scale(&self, scale: TimeScale) -> Epoch {
        Self(self.0.to_time_scale(scale.into()))
    }

    /// Returns the exact duration since the reference epoch of the time scale.
    #[wasm_bindgen(js_name = toDurationInTimeScale)]
    pub fn to_duration_in_time_scale(&self, scale: TimeScale) -> Duration {
        self.0.to_duration_in_time_scale(scale.into()).into()
    }

    /// Returns the number of seconds since the reference epoch of the time scale.
    #[wasm_bindgen(js_name = toSecondsInTimeScale)]
    pub fn to_seconds_in_time_scale(&self, scale: TimeScale) -> f64 {
        self.0.to_duration_in_time_scale(scale.into()).to_seconds()
    }

    /// Returns the exact duration since 1900-01-01 00:00:00 TAI.
    #[wasm_bindgen(js_name = toDurationSinceJ1900)]
    pub fn to_duration_since_j1900(&self) -> Duration {
        self.0.to_duration_since_j1900().into()
    }

    /// Returns the exact duration since 1900-01-01 00:00:00 TAI.
    #[wasm_bindgen(js_name = toTaiDuration)]
    pub fn to_tai_duration(&self) -> Duration {
        self.0.to_tai_duration().into()
    }

    /// Returns the exact century count and nanosecond count since
    /// 1900-01-01 00:00:00 TAI.
    #[wasm_bindgen(js_name = toTaiParts)]
    pub fn to_tai_parts(&self) -> DurationParts {
        self.0.to_tai_parts().into()
    }

    #[wasm_bindgen(js_name = toTaiSeconds)]
    pub fn to_tai_seconds(&self) -> f64 {
        self.0.to_tai_seconds()
    }

    #[wasm_bindgen(js_name = toTaiDays)]
    pub fn to_tai_days(&self) -> f64 {
        self.0.to_tai_days()
    }

    #[wasm_bindgen(js_name = toUtcSeconds)]
    pub fn to_utc_seconds(&self) -> f64 {
        self.0.to_utc_seconds()
    }

    #[wasm_bindgen(js_name = toUtcDays)]
    pub fn to_utc_days(&self) -> f64 {
        self.0.to_utc_days()
    }

    #[wasm_bindgen(js_name = toTtSeconds)]
    pub fn to_tt_seconds(&self) -> f64 {
        self.0.to_tt_seconds()
    }

    /// Returns the number of Julian centuries of Terrestrial Time since J2000.
    /// Many astrodynamics formulations use this value.
    #[wasm_bindgen(js_name = toTtCenturiesSinceJ2000)]
    pub fn to_tt_centuries_since_j2000(&self) -> f64 {
        self.0.to_tt_centuries_j2k()
    }

    #[wasm_bindgen(js_name = toEtSeconds)]
    pub fn to_et_seconds(&self) -> f64 {
        self.0.to_et_seconds()
    }

    /// Returns the number of days of ephemeris time since J2000.
    #[wasm_bindgen(js_name = toEtDaysSinceJ2000)]
    pub fn to_et_days_since_j2000(&self) -> f64 {
        self.0.to_et_days_since_j2000()
    }

    #[wasm_bindgen(js_name = toTdbSeconds)]
    pub fn to_tdb_seconds(&self) -> f64 {
        self.0.to_tdb_seconds()
    }

    /// Returns the number of days of barycentric dynamical time since J2000.
    #[wasm_bindgen(js_name = toTdbDaysSinceJ2000)]
    pub fn to_tdb_days_since_j2000(&self) -> f64 {
        self.0.to_tdb_days_since_j2000()
    }

    /// Returns the number of Julian centuries of barycentric dynamical time
    /// since J2000.
    #[wasm_bindgen(js_name = toTdbCenturiesSinceJ2000)]
    pub fn to_tdb_centuries_since_j2000(&self) -> f64 {
        self.0.to_tdb_centuries_since_j2000()
    }

    #[wasm_bindgen(js_name = toTcgSeconds)]
    pub fn to_tcg_seconds(&self) -> f64 {
        self.0.to_tcg_seconds()
    }

    #[wasm_bindgen(js_name = toTcbSeconds)]
    pub fn to_tcb_seconds(&self) -> f64 {
        self.0.to_tcb_seconds()
    }

    #[wasm_bindgen(js_name = toMjdTaiDays)]
    pub fn to_mjd_tai_days(&self) -> f64 {
        self.0.to_mjd_tai_days()
    }

    #[wasm_bindgen(js_name = toMjdUtcDays)]
    pub fn to_mjd_utc_days(&self) -> f64 {
        self.0.to_mjd_utc_days()
    }

    #[wasm_bindgen(js_name = toMjdTtDays)]
    pub fn to_mjd_tt_days(&self) -> f64 {
        self.0.to_mjd_tt_days()
    }

    #[wasm_bindgen(js_name = toJdeTaiDays)]
    pub fn to_jde_tai_days(&self) -> f64 {
        self.0.to_jde_tai_days()
    }

    #[wasm_bindgen(js_name = toJdeUtcDays)]
    pub fn to_jde_utc_days(&self) -> f64 {
        self.0.to_jde_utc_days()
    }

    #[wasm_bindgen(js_name = toJdeTtDays)]
    pub fn to_jde_tt_days(&self) -> f64 {
        self.0.to_jde_tt_days()
    }

    #[wasm_bindgen(js_name = toJdeEtDays)]
    pub fn to_jde_et_days(&self) -> f64 {
        self.0.to_jde_et_days()
    }

    #[wasm_bindgen(js_name = toJdeTdbDays)]
    pub fn to_jde_tdb_days(&self) -> f64 {
        self.0.to_jde_tdb_days()
    }

    #[wasm_bindgen(js_name = toGpstSeconds)]
    pub fn to_gpst_seconds(&self) -> f64 {
        self.0.to_gpst_seconds()
    }

    /// Returns the number of nanoseconds since the GPS reference epoch. It
    /// throws when this epoch precedes that reference epoch.
    #[wasm_bindgen(js_name = toGpstNanoseconds)]
    pub fn to_gpst_nanoseconds(&self) -> Result<u64, JsError> {
        self.0.to_gpst_nanoseconds().map_err(js_err)
    }

    #[wasm_bindgen(js_name = toQzsstSeconds)]
    pub fn to_qzsst_seconds(&self) -> f64 {
        self.0.to_qzsst_seconds()
    }

    /// Returns the number of nanoseconds since the QZSS reference epoch. It
    /// throws when this epoch precedes that reference epoch.
    #[wasm_bindgen(js_name = toQzsstNanoseconds)]
    pub fn to_qzsst_nanoseconds(&self) -> Result<u64, JsError> {
        self.0.to_qzsst_nanoseconds().map_err(js_err)
    }

    #[wasm_bindgen(js_name = toGstSeconds)]
    pub fn to_gst_seconds(&self) -> f64 {
        self.0.to_gst_seconds()
    }

    /// Returns the number of nanoseconds since the Galileo reference epoch. It
    /// throws when this epoch precedes that reference epoch.
    #[wasm_bindgen(js_name = toGstNanoseconds)]
    pub fn to_gst_nanoseconds(&self) -> Result<u64, JsError> {
        self.0.to_gst_nanoseconds().map_err(js_err)
    }

    #[wasm_bindgen(js_name = toBdtSeconds)]
    pub fn to_bdt_seconds(&self) -> f64 {
        self.0.to_bdt_seconds()
    }

    /// Returns the number of nanoseconds since the BeiDou reference epoch. It
    /// throws when this epoch precedes that reference epoch.
    #[wasm_bindgen(js_name = toBdtNanoseconds)]
    pub fn to_bdt_nanoseconds(&self) -> Result<u64, JsError> {
        self.0.to_bdt_nanoseconds().map_err(js_err)
    }

    #[wasm_bindgen(js_name = toUnixSeconds)]
    pub fn to_unix_seconds(&self) -> f64 {
        self.0.to_unix_seconds()
    }

    /// Returns the Unix timestamp in milliseconds. The result is a `number`, so
    /// it can carry a rounding error of a fraction of a microsecond. Use
    /// `toUnixNanoseconds()` or `toDate()` when that matters.
    #[wasm_bindgen(js_name = toUnixMilliseconds)]
    pub fn to_unix_milliseconds(&self) -> f64 {
        self.0.to_unix_milliseconds()
    }

    /// Returns the exact Unix timestamp in nanoseconds.
    #[wasm_bindgen(js_name = toUnixNanoseconds)]
    pub fn to_unix_nanoseconds(&self) -> i128 {
        self.0.to_unix_duration().total_nanoseconds()
    }

    #[wasm_bindgen(js_name = toUnixDays)]
    pub fn to_unix_days(&self) -> f64 {
        self.0.to_unix_days()
    }

    /// Returns the exact duration since 1970-01-01 00:00:00 UTC.
    #[wasm_bindgen(js_name = toUnixDuration)]
    pub fn to_unix_duration(&self) -> Duration {
        self.0.to_unix_duration().into()
    }

    /// Returns a JavaScript `Date`. The result has millisecond precision, so it
    /// drops the microseconds and the nanoseconds of this epoch.
    #[wasm_bindgen(js_name = toDate)]
    pub fn to_date(&self) -> js_sys::Date {
        let nanoseconds = self.to_unix_nanoseconds();
        let milliseconds = nanoseconds.div_euclid(1_000_000) as f64;
        js_sys::Date::new(&JsValue::from_f64(milliseconds))
    }

    /// Returns the position of this epoch inside its GNSS week.
    #[wasm_bindgen(js_name = toTimeOfWeek)]
    pub fn to_time_of_week(&self) -> TimeOfWeek {
        let (week, nanoseconds) = self.0.to_time_of_week();
        TimeOfWeek { week, nanoseconds }
    }

    /// Returns the Gregorian date and time in the time scale of this epoch.
    #[wasm_bindgen(js_name = toGregorian)]
    pub fn to_gregorian(&self) -> GregorianDate {
        self.0.to_gregorian(self.0.time_scale).into()
    }

    /// Returns the Gregorian date and time in the requested time scale.
    #[wasm_bindgen(js_name = toGregorianInTimeScale)]
    pub fn to_gregorian_in_time_scale(&self, scale: TimeScale) -> GregorianDate {
        self.0.to_gregorian(scale.into()).into()
    }

    /// Formats this epoch in the given time scale, for example
    /// `"2024-02-29T12:34:56.789000000 UTC"`.
    #[wasm_bindgen(js_name = toGregorianString)]
    pub fn to_gregorian_string(&self, scale: TimeScale) -> String {
        self.0.to_gregorian_str(scale.into())
    }

    /// Formats this epoch as an ISO 8601 string, with microseconds and without
    /// the name of the time scale, for example `"2024-02-29T12:34:56.789000"`.
    #[wasm_bindgen(js_name = toIsoformat)]
    pub fn to_isoformat(&self) -> String {
        self.0.to_isoformat()
    }

    /// Formats this epoch as an RFC 3339 string.
    #[wasm_bindgen(js_name = toRfc3339)]
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Formats this epoch with an explicit format, for example
    /// `strftime("%Y-%m-%d %H:%M:%S")`.
    pub fn strftime(&self, format_str: &str) -> Result<String, JsError> {
        let format = Format::from_str(format_str).map_err(js_err)?;
        Ok(Formatter::new(self.0, format).to_string())
    }

    /// Formats this epoch in its own time scale.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Returns the string form of this epoch, so `JSON.stringify` works.
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> String {
        self.0.to_string()
    }

    /// Returns the number of leap seconds between TAI and UTC at this epoch,
    /// counting only the announcements of the IERS.
    #[wasm_bindgen(js_name = leapSecondsIers)]
    pub fn leap_seconds_iers(&self) -> i32 {
        self.0.leap_seconds_iers()
    }

    /// Returns the number of leap seconds between TAI and UTC at this epoch, or
    /// `undefined` if this epoch precedes the first leap second.
    ///
    /// Set `iersOnly` to false to include the extrapolated leap seconds of NAIF.
    #[wasm_bindgen(js_name = leapSeconds)]
    pub fn leap_seconds(&self, iers_only: bool) -> Option<f64> {
        self.0.leap_seconds(iers_only)
    }

    /// Returns the exact duration since the start of the year of this epoch.
    #[wasm_bindgen(js_name = durationInYear)]
    pub fn duration_in_year(&self) -> Duration {
        self.0.duration_in_year().into()
    }

    /// Adds a duration to this epoch.
    pub fn add(&self, duration: &Duration) -> Epoch {
        Self(self.0 + duration.0)
    }

    /// Adds a value of one unit to this epoch, for example
    /// `addUnit(1.5, Unit.Hour)`.
    #[wasm_bindgen(js_name = addUnit)]
    pub fn add_unit(&self, value: f64, unit: Unit) -> Epoch {
        Self(self.0 + ht::Unit::from(unit) * value)
    }

    /// Subtracts a duration from this epoch.
    pub fn sub(&self, duration: &Duration) -> Epoch {
        Self(self.0 - duration.0)
    }

    /// Returns the exact duration from `other` to this epoch. The result is
    /// negative when this epoch precedes `other`.
    #[wasm_bindgen(js_name = durationSince)]
    pub fn duration_since(&self, other: &Epoch) -> Duration {
        (self.0 - other.0).into()
    }

    /// Returns the earlier of the two epochs.
    pub fn min(&self, other: &Epoch) -> Epoch {
        Self(self.0.min(other.0))
    }

    /// Returns the later of the two epochs.
    pub fn max(&self, other: &Epoch) -> Epoch {
        Self(self.0.max(other.0))
    }

    /// Rounds this epoch down to the previous multiple of `step`.
    pub fn floor(&self, step: &Duration) -> Epoch {
        Self(self.0.floor(step.0))
    }

    /// Rounds this epoch up to the next multiple of `step`.
    pub fn ceil(&self, step: &Duration) -> Epoch {
        Self(self.0.ceil(step.0))
    }

    /// Rounds this epoch to the closest multiple of `step`.
    pub fn round(&self, step: &Duration) -> Epoch {
        Self(self.0.round(step.0))
    }

    /// Compares two epochs. It returns `-1`, `0`, or `1`, so it can be passed to
    /// `Array.prototype.sort`.
    pub fn compare(&self, other: &Epoch) -> i32 {
        match self.0.cmp(&other.0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// Returns true if both epochs are the same instant. The time scale does not
    /// take part in the comparison.
    pub fn equals(&self, other: &Epoch) -> bool {
        self.0 == other.0
    }

    /// Returns a copy of this epoch.
    #[wasm_bindgen(js_name = clone)]
    pub fn duplicate(&self) -> Epoch {
        *self
    }

    /// The day of the week in the time scale of this epoch.
    pub fn weekday(&self) -> Weekday {
        self.0.weekday().into()
    }

    /// The day of the week in UTC.
    #[wasm_bindgen(js_name = weekdayUtc)]
    pub fn weekday_utc(&self) -> Weekday {
        self.0.weekday_utc().into()
    }

    /// Returns the next occurrence of the day of the week, at the same time of day.
    pub fn next(&self, weekday: Weekday) -> Epoch {
        Self(self.0.next(weekday.into()))
    }

    /// Returns the next occurrence of the day of the week, at midnight.
    #[wasm_bindgen(js_name = nextWeekdayAtMidnight)]
    pub fn next_weekday_at_midnight(&self, weekday: Weekday) -> Epoch {
        Self(self.0.next_weekday_at_midnight(weekday.into()))
    }

    /// Returns the next occurrence of the day of the week, at noon.
    #[wasm_bindgen(js_name = nextWeekdayAtNoon)]
    pub fn next_weekday_at_noon(&self, weekday: Weekday) -> Epoch {
        Self(self.0.next_weekday_at_noon(weekday.into()))
    }

    /// Returns the previous occurrence of the day of the week, at the same time
    /// of day.
    pub fn previous(&self, weekday: Weekday) -> Epoch {
        Self(self.0.previous(weekday.into()))
    }

    /// Returns the previous occurrence of the day of the week, at midnight.
    #[wasm_bindgen(js_name = previousWeekdayAtMidnight)]
    pub fn previous_weekday_at_midnight(&self, weekday: Weekday) -> Epoch {
        Self(self.0.previous_weekday_at_midnight(weekday.into()))
    }

    /// Returns the previous occurrence of the day of the week, at noon.
    #[wasm_bindgen(js_name = previousWeekdayAtNoon)]
    pub fn previous_weekday_at_noon(&self, weekday: Weekday) -> Epoch {
        Self(self.0.previous_weekday_at_noon(weekday.into()))
    }

    /// Returns this epoch with a new time of day. It removes the subsecond part.
    /// During a leap second the seconds may reach 60.
    #[wasm_bindgen(js_name = withHms)]
    pub fn with_hms(&self, hours: u32, minutes: u32, seconds: u32) -> Epoch {
        Self(
            self.0
                .with_hms(hours.into(), minutes.into(), seconds.into()),
        )
    }

    /// Returns this epoch with a new time of day. It keeps the subsecond part.
    #[wasm_bindgen(js_name = withHmsStrict)]
    pub fn with_hms_strict(&self, hours: u32, minutes: u32, seconds: u32) -> Epoch {
        Self(
            self.0
                .with_hms_strict(hours.into(), minutes.into(), seconds.into()),
        )
    }

    /// Returns this epoch with the hours, the minutes, and the seconds of
    /// `other`. It keeps the subsecond part of this epoch.
    #[wasm_bindgen(js_name = withHmsFrom)]
    pub fn with_hms_from(&self, other: &Epoch) -> Epoch {
        Self(self.0.with_hms_from(other.0))
    }

    /// Returns this epoch with the full time of day of `other`, down to the
    /// nanosecond.
    #[wasm_bindgen(js_name = withTimeFrom)]
    pub fn with_time_from(&self, other: &Epoch) -> Epoch {
        Self(self.0.with_time_from(other.0))
    }

    /// Converts to another time scale with an interpolation polynomial, as
    /// published by a GNSS constellation.
    ///
    /// Set `forward` to true when the polynomial describes
    /// `thisTimeScale - target`, and to false for the opposite direction.
    #[wasm_bindgen(js_name = preciseTimescaleConversion)]
    pub fn precise_timescale_conversion(
        &self,
        forward: bool,
        reference_epoch: &Epoch,
        polynomial: &Polynomial,
        target: TimeScale,
    ) -> Result<Epoch, JsError> {
        self.0
            .precise_timescale_conversion(forward, reference_epoch.0, polynomial.0, target.into())
            .map(Self)
            .map_err(js_err)
    }
}
