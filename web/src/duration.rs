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
use wasm_bindgen::prelude::*;

use crate::enums::Unit;
use crate::error::js_err;

/// An exact duration with nanosecond precision, valid for 32,768 centuries in
/// either direction.
///
/// The internal representation is a century count plus a nanosecond count into
/// that century. No method loses precision unless it returns a `number`, because
/// a JavaScript `number` holds 53 significant bits. Use `totalNanoseconds()`,
/// which returns a `bigint`, when the full precision matters.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Duration(pub(crate) ht::Duration);

impl From<ht::Duration> for Duration {
    fn from(value: ht::Duration) -> Self {
        Self(value)
    }
}

/// A duration broken down into calendar-like components, all of them positive.
/// Read `sign` to know the direction of the duration.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct DurationComponents {
    /// `-1` for a negative duration, `0` for a zero duration, `+1` otherwise.
    pub sign: i8,
    pub days: f64,
    pub hours: f64,
    pub minutes: f64,
    pub seconds: f64,
    pub milliseconds: f64,
    pub microseconds: f64,
    pub nanoseconds: f64,
}

/// The century count and the nanosecond count into that century.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct DurationParts {
    /// Number of centuries. It is negative for a negative duration.
    pub centuries: i16,
    /// Number of nanoseconds into the century.
    pub nanoseconds: u64,
}

impl From<(i16, u64)> for DurationParts {
    fn from((centuries, nanoseconds): (i16, u64)) -> Self {
        Self {
            centuries,
            nanoseconds,
        }
    }
}

#[wasm_bindgen]
impl Duration {
    /// Parses a duration, for example `"1 d"`, `"10.598 min"`, or `"3 h 15 min"`.
    #[wasm_bindgen(constructor)]
    pub fn new(duration_str: &str) -> Result<Duration, JsError> {
        Self::from_string(duration_str)
    }

    /// Parses a duration, for example `"1 d"` or `"10.598 min"`.
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(duration_str: &str) -> Result<Duration, JsError> {
        ht::Duration::from_str(duration_str)
            .map(Self)
            .map_err(js_err)
    }

    /// A zero duration.
    pub fn zero() -> Duration {
        Self(ht::Duration::ZERO)
    }

    /// One nanosecond, the smallest positive duration.
    pub fn epsilon() -> Duration {
        Self(ht::Duration::EPSILON)
    }

    /// The largest duration that this representation supports.
    #[wasm_bindgen(js_name = maxValue)]
    pub fn max_value() -> Duration {
        Self(ht::Duration::MAX)
    }

    /// The largest negative duration that this representation supports.
    #[wasm_bindgen(js_name = minValue)]
    pub fn min_value() -> Duration {
        Self(ht::Duration::MIN)
    }

    /// One nanosecond, the smallest positive duration.
    #[wasm_bindgen(js_name = minPositive)]
    pub fn min_positive() -> Duration {
        Self(ht::Duration::MIN_POSITIVE)
    }

    /// Minus one nanosecond, the smallest negative duration.
    #[wasm_bindgen(js_name = minNegative)]
    pub fn min_negative() -> Duration {
        Self(ht::Duration::MIN_NEGATIVE)
    }

    /// Builds a duration from a century count and a nanosecond count into that
    /// century. This constructor is exact.
    #[wasm_bindgen(js_name = fromParts)]
    pub fn from_parts(centuries: i16, nanoseconds: u64) -> Duration {
        Self(ht::Duration::from_parts(centuries, nanoseconds))
    }

    /// Builds a duration from a total nanosecond count. This constructor is exact.
    #[wasm_bindgen(js_name = fromTotalNanoseconds)]
    pub fn from_total_nanoseconds(nanoseconds: i128) -> Duration {
        Self(ht::Duration::from_total_nanoseconds(nanoseconds))
    }

    /// Builds an exact duration from its components. Set `sign` to a negative
    /// number for a negative duration.
    ///
    /// Each component takes a `number`, not a `bigint`, because none of them can
    /// exceed the 53 significant bits of a JavaScript `number`.
    pub fn compose(
        sign: i8,
        days: f64,
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> Duration {
        Self(ht::Duration::compose_f64(
            sign,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        ))
    }

    /// Builds the offset of a time zone, for example `fromTzOffset(-1, 5, 30)`
    /// for `UTC-05:30`.
    #[wasm_bindgen(js_name = fromTzOffset)]
    pub fn from_tz_offset(sign: i8, hours: i32, minutes: i32) -> Duration {
        Self(ht::Duration::from_tz_offset(
            sign,
            hours.into(),
            minutes.into(),
        ))
    }

    /// Builds a duration from a value and a unit.
    #[wasm_bindgen(js_name = fromUnit)]
    pub fn from_unit(value: f64, unit: Unit) -> Duration {
        Self(value * ht::Unit::from(unit))
    }

    #[wasm_bindgen(js_name = fromCenturies)]
    pub fn from_centuries(value: f64) -> Duration {
        Self(value * ht::Unit::Century)
    }

    #[wasm_bindgen(js_name = fromWeeks)]
    pub fn from_weeks(value: f64) -> Duration {
        Self(value * ht::Unit::Week)
    }

    #[wasm_bindgen(js_name = fromDays)]
    pub fn from_days(value: f64) -> Duration {
        Self(ht::Duration::from_days(value))
    }

    #[wasm_bindgen(js_name = fromHours)]
    pub fn from_hours(value: f64) -> Duration {
        Self(ht::Duration::from_hours(value))
    }

    #[wasm_bindgen(js_name = fromMinutes)]
    pub fn from_minutes(value: f64) -> Duration {
        Self(value * ht::Unit::Minute)
    }

    #[wasm_bindgen(js_name = fromSeconds)]
    pub fn from_seconds(value: f64) -> Duration {
        Self(ht::Duration::from_seconds(value))
    }

    #[wasm_bindgen(js_name = fromMilliseconds)]
    pub fn from_milliseconds(value: f64) -> Duration {
        Self(ht::Duration::from_milliseconds(value))
    }

    #[wasm_bindgen(js_name = fromMicroseconds)]
    pub fn from_microseconds(value: f64) -> Duration {
        Self(ht::Duration::from_microseconds(value))
    }

    #[wasm_bindgen(js_name = fromNanoseconds)]
    pub fn from_nanoseconds(value: f64) -> Duration {
        Self(ht::Duration::from_nanoseconds(value))
    }

    /// Returns the exact total number of nanoseconds of this duration.
    #[wasm_bindgen(js_name = totalNanoseconds)]
    pub fn total_nanoseconds(&self) -> i128 {
        self.0.total_nanoseconds()
    }

    /// Returns the total number of nanoseconds as a 64 bit integer. It throws if
    /// the duration does not fit in 64 bits, which happens beyond 292 years.
    #[wasm_bindgen(js_name = truncatedNanoseconds)]
    pub fn truncated_nanoseconds(&self) -> Result<i64, JsError> {
        self.0.try_truncated_nanoseconds().map_err(js_err)
    }

    /// Returns the century count and the nanosecond count into that century.
    #[wasm_bindgen(js_name = toParts)]
    pub fn to_parts(&self) -> DurationParts {
        self.0.to_parts().into()
    }

    /// Returns this duration in the requested unit. The result is a `number`, so
    /// it can lose precision.
    #[wasm_bindgen(js_name = toUnit)]
    pub fn to_unit(&self, unit: Unit) -> f64 {
        self.0.to_unit(unit.into())
    }

    #[wasm_bindgen(js_name = toCenturies)]
    pub fn to_centuries(&self) -> f64 {
        self.0.to_unit(ht::Unit::Century)
    }

    #[wasm_bindgen(js_name = toWeeks)]
    pub fn to_weeks(&self) -> f64 {
        self.0.to_unit(ht::Unit::Week)
    }

    #[wasm_bindgen(js_name = toDays)]
    pub fn to_days(&self) -> f64 {
        self.0.to_unit(ht::Unit::Day)
    }

    #[wasm_bindgen(js_name = toHours)]
    pub fn to_hours(&self) -> f64 {
        self.0.to_unit(ht::Unit::Hour)
    }

    #[wasm_bindgen(js_name = toMinutes)]
    pub fn to_minutes(&self) -> f64 {
        self.0.to_unit(ht::Unit::Minute)
    }

    #[wasm_bindgen(js_name = toSeconds)]
    pub fn to_seconds(&self) -> f64 {
        self.0.to_seconds()
    }

    #[wasm_bindgen(js_name = toMilliseconds)]
    pub fn to_milliseconds(&self) -> f64 {
        self.0.to_unit(ht::Unit::Millisecond)
    }

    #[wasm_bindgen(js_name = toMicroseconds)]
    pub fn to_microseconds(&self) -> f64 {
        self.0.to_unit(ht::Unit::Microsecond)
    }

    #[wasm_bindgen(js_name = toNanoseconds)]
    pub fn to_nanoseconds(&self) -> f64 {
        self.0.to_unit(ht::Unit::Nanosecond)
    }

    /// Breaks this duration into days, hours, minutes, and smaller components.
    pub fn decompose(&self) -> DurationComponents {
        let (sign, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) =
            self.0.decompose();
        DurationComponents {
            sign,
            days: days as f64,
            hours: hours as f64,
            minutes: minutes as f64,
            seconds: seconds as f64,
            milliseconds: milliseconds as f64,
            microseconds: microseconds as f64,
            nanoseconds: nanoseconds as f64,
        }
    }

    /// Returns the part of this duration that is smaller than the given unit,
    /// or `undefined` if the unit is not supported.
    pub fn subdivision(&self, unit: Unit) -> Option<Duration> {
        self.0.subdivision(unit.into()).map(Self)
    }

    /// Returns `-1` for a negative duration, `0` for a zero duration, `+1` otherwise.
    pub fn signum(&self) -> i8 {
        self.0.signum()
    }

    /// Returns true if this duration is negative.
    #[wasm_bindgen(js_name = isNegative)]
    pub fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// Returns the absolute value of this duration.
    pub fn abs(&self) -> Duration {
        Self(self.0.abs())
    }

    /// Returns this duration with the opposite sign.
    pub fn neg(&self) -> Duration {
        Self(-self.0)
    }

    /// Adds two durations. It saturates at the largest supported duration.
    pub fn add(&self, other: &Duration) -> Duration {
        Self(self.0 + other.0)
    }

    /// Subtracts a duration. It saturates at the largest supported duration.
    pub fn sub(&self, other: &Duration) -> Duration {
        Self(self.0 - other.0)
    }

    /// Multiplies this duration by a factor.
    pub fn mul(&self, factor: f64) -> Duration {
        Self(self.0 * factor)
    }

    /// Divides this duration by a divisor.
    pub fn div(&self, divisor: f64) -> Duration {
        Self(self.0 / divisor)
    }

    /// Returns the smaller of the two durations.
    pub fn min(&self, other: &Duration) -> Duration {
        Self(self.0.min(other.0))
    }

    /// Returns the larger of the two durations.
    pub fn max(&self, other: &Duration) -> Duration {
        Self(self.0.max(other.0))
    }

    /// Rounds this duration down to the previous multiple of `step`.
    pub fn floor(&self, step: &Duration) -> Duration {
        Self(self.0.floor(step.0))
    }

    /// Rounds this duration up to the next multiple of `step`.
    pub fn ceil(&self, step: &Duration) -> Duration {
        Self(self.0.ceil(step.0))
    }

    /// Rounds this duration to the closest multiple of `step`.
    pub fn round(&self, step: &Duration) -> Duration {
        Self(self.0.round(step.0))
    }

    /// Rounds this duration to a human-friendly value, for example 15 minutes.
    pub fn approx(&self) -> Duration {
        Self(self.0.approx())
    }

    /// Compares two durations. It returns `-1`, `0`, or `1`, so it can be passed
    /// to `Array.prototype.sort`.
    pub fn compare(&self, other: &Duration) -> i32 {
        match self.0.cmp(&other.0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// Returns true if both durations are the same.
    pub fn equals(&self, other: &Duration) -> bool {
        self.0 == other.0
    }

    /// Returns a copy of this duration.
    #[wasm_bindgen(js_name = clone)]
    pub fn duplicate(&self) -> Duration {
        *self
    }

    /// Returns a readable form of this duration, for example `"1 day 3 h"`.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Returns the string form of this duration, so `JSON.stringify` works.
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> String {
        self.0.to_string()
    }
}
