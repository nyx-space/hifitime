/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the implementations that are only built with the pyhon feature.

use super::{Duration, Unit};

use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyType;
use std::str::FromStr;

#[pymethods]
impl Duration {
    #[must_use]
    /// Returns the centuries and nanoseconds of this duration
    /// NOTE: These items are not public to prevent incorrect durations from being created by modifying the values of the structure directly.
    #[pyo3(name = "to_parts")]
    pub const fn py_to_parts(&self) -> (i16, u64) {
        (self.centuries, self.nanoseconds)
    }

    /// Returns the total nanoseconds in a signed 128 bit integer
    #[pyo3(name = "total_nanoseconds")]
    pub fn py_total_nanoseconds(&self) -> i128 {
        self.total_nanoseconds()
    }

    /// Returns this duration in seconds f64.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    #[pyo3(name = "to_seconds")]
    pub fn py_to_seconds(&self) -> f64 {
        self.to_seconds()
    }

    #[pyo3(name = "to_unit")]
    pub fn py_to_unit(&self, unit: Unit) -> f64 {
        self.to_unit(unit)
    }

    /// Returns the absolute value of this duration
    #[pyo3(name = "abs")]
    pub fn py_abs(&self) -> Self {
        self.abs()
    }

    /// Returns the sign of this duration
    /// + 0 if the number is zero
    /// + 1 if the number is positive
    /// + -1 if the number is negative
    #[pyo3(name = "signum")]
    pub const fn py_signum(&self) -> i8 {
        self.signum()
    }

    /// Decomposes a Duration in its sign, days, hours, minutes, seconds, ms, us, ns
    #[pyo3(name = "decompose")]
    pub fn py_decompose(&self) -> (i8, u64, u64, u64, u64, u64, u64, u64) {
        self.decompose()
    }

    /// Floors this duration to the closest duration from the bottom
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.floor(1.hours()), 2.hours());
    /// assert_eq!(two_hours_three_min.floor(30.minutes()), 2.hours());
    /// // This is zero because we floor by a duration longer than the current duration, rounding it down
    /// assert_eq!(two_hours_three_min.floor(4.hours()), 0.hours());
    /// assert_eq!(two_hours_three_min.floor(1.seconds()), two_hours_three_min);
    /// assert_eq!(two_hours_three_min.floor(1.hours() + 1.minutes()), 2.hours() + 2.minutes());
    /// assert_eq!(two_hours_three_min.floor(1.hours() + 5.minutes()), 1.hours() + 5.minutes());
    /// ```
    #[pyo3(name = "floor")]
    pub fn py_floor(&self, duration: Self) -> Self {
        self.floor(duration)
    }

    /// Ceils this duration to the closest provided duration
    ///
    /// This simply floors then adds the requested duration
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.ceil(1.hours()), 3.hours());
    /// assert_eq!(two_hours_three_min.ceil(30.minutes()), 2.hours() + 30.minutes());
    /// assert_eq!(two_hours_three_min.ceil(4.hours()), 4.hours());
    /// assert_eq!(two_hours_three_min.ceil(1.seconds()), two_hours_three_min + 1.seconds());
    /// assert_eq!(two_hours_three_min.ceil(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
    /// ```
    #[pyo3(name = "ceil")]
    pub fn py_ceil(&self, duration: Self) -> Self {
        self.ceil(duration)
    }

    /// Rounds this duration to the closest provided duration
    ///
    /// This performs both a `ceil` and `floor` and returns the value which is the closest to current one.
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// let two_hours_three_min = 2.hours() + 3.minutes();
    /// assert_eq!(two_hours_three_min.round(1.hours()), 2.hours());
    /// assert_eq!(two_hours_three_min.round(30.minutes()), 2.hours());
    /// assert_eq!(two_hours_three_min.round(4.hours()), 4.hours());
    /// assert_eq!(two_hours_three_min.round(1.seconds()), two_hours_three_min);
    /// assert_eq!(two_hours_three_min.round(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
    /// ```
    #[pyo3(name = "round")]
    pub fn py_round(&self, duration: Self) -> Self {
        self.round(duration)
    }

    /// Rounds this duration to the largest units represented in this duration.
    ///
    /// This is useful to provide an approximate human duration. Under the hood, this function uses `round`,
    /// so the "tipping point" of the rounding is half way to the next increment of the greatest unit.
    /// As shown below, one example is that 35 hours and 59 minutes rounds to 1 day, but 36 hours and 1 minute rounds
    /// to 2 days because 2 days is closer to 36h 1 min than 36h 1 min is to 1 day.
    ///
    /// # Example
    ///
    /// ```
    /// use hifitime::{Duration, TimeUnits};
    ///
    /// assert_eq!((2.hours() + 3.minutes()).approx(), 2.hours());
    /// assert_eq!((24.hours() + 3.minutes()).approx(), 1.days());
    /// assert_eq!((35.hours() + 59.minutes()).approx(), 1.days());
    /// assert_eq!((36.hours() + 1.minutes()).approx(), 2.days());
    /// assert_eq!((47.hours() + 3.minutes()).approx(), 2.days());
    /// assert_eq!((49.hours() + 3.minutes()).approx(), 2.days());
    /// ```
    #[pyo3(name = "approx")]
    pub fn py_approx(&self) -> Self {
        self.approx()
    }

    /// Returns the minimum of the two durations.
    ///
    /// ```
    /// use hifitime::TimeUnits;
    ///
    /// let d0 = 20.seconds();
    /// let d1 = 21.seconds();
    ///
    /// assert_eq!(d0, d1.min(d0));
    /// assert_eq!(d0, d0.min(d1));
    /// ```
    #[pyo3(name = "min")]
    pub fn py_min(&self, other: Self) -> Self {
        *(self.min(&other))
    }

    /// Returns the maximum of the two durations.
    ///
    /// ```
    /// use hifitime::TimeUnits;
    ///
    /// let d0 = 20.seconds();
    /// let d1 = 21.seconds();
    ///
    /// assert_eq!(d1, d1.max(d0));
    /// assert_eq!(d1, d0.max(d1));
    /// ```
    #[pyo3(name = "max")]
    pub fn py_max(&self, other: Self) -> Self {
        *(self.max(&other))
    }

    /// Returns whether this is a negative or positive duration.
    #[pyo3(name = "is_negative")]
    pub fn py_is_negative(&self) -> bool {
        self.is_negative()
    }

    #[new]
    fn new_py(string_repr: String) -> PyResult<Self> {
        match Self::from_str(&string_repr) {
            Ok(d) => Ok(d),
            Err(e) => Err(PyErr::from(e)),
        }
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self} @ {self:p}")
    }

    fn __add__(&self, other: Self) -> Duration {
        *self + other
    }

    fn __sub__(&self, other: Self) -> Duration {
        *self - other
    }

    fn __mul__(&self, other: f64) -> Duration {
        *self * other
    }

    fn __div__(&self, other: f64) -> Duration {
        *self / other
    }

    fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Lt => *self < other,
            CompareOp::Le => *self <= other,
            CompareOp::Eq => *self == other,
            CompareOp::Ne => *self != other,
            CompareOp::Gt => *self > other,
            CompareOp::Ge => *self >= other,
        }
    }

    fn __getnewargs__(&self) -> Result<(String,), PyErr> {
        Ok((format!("{self}"),))
    }

    // Python constructors

    #[classmethod]
    #[pyo3(name = "ZERO")]
    fn zero(_cls: &Bound<'_, PyType>) -> Duration {
        Duration::ZERO
    }

    #[classmethod]
    #[pyo3(name = "EPSILON")]
    fn epsilon(_cls: &Bound<'_, PyType>) -> Duration {
        Duration::EPSILON
    }

    #[classmethod]
    #[pyo3(name = "MAX")]
    fn py_from_max(_cls: &Bound<'_, PyType>) -> Duration {
        Duration::MAX
    }

    #[classmethod]
    #[pyo3(name = "MIN")]
    fn py_from_min(_cls: &Bound<'_, PyType>) -> Duration {
        Duration::MIN
    }

    #[classmethod]
    #[pyo3(name = "MIN_POSITIVE")]
    fn min_positive(_cls: &Bound<'_, PyType>) -> Duration {
        Duration::MIN_POSITIVE
    }

    #[classmethod]
    #[pyo3(name = "MIN_NEGATIVE")]
    fn min_negative(_cls: &Bound<'_, PyType>) -> Duration {
        Duration::MIN_NEGATIVE
    }

    #[classmethod]
    #[pyo3(name = "from_parts")]
    /// Create a normalized duration from its parts
    fn py_from_parts(_cls: &Bound<'_, PyType>, centuries: i16, nanoseconds: u64) -> Self {
        Self::from_parts(centuries, nanoseconds)
    }

    /// Creates a new duration from its parts
    #[allow(clippy::too_many_arguments)]
    #[classmethod]
    #[pyo3(name = "from_all_parts")]
    fn py_from_all_parts(
        _cls: &Bound<'_, PyType>,
        sign: i8,
        days: u64,
        hours: u64,
        minutes: u64,
        seconds: u64,
        milliseconds: u64,
        microseconds: u64,
        nanoseconds: u64,
    ) -> Self {
        Self::compose(
            sign,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )
    }

    #[classmethod]
    #[pyo3(name = "from_total_nanoseconds")]
    fn py_from_total_nanoseconds(_cls: &Bound<'_, PyType>, nanos: i128) -> Self {
        Self::from_total_nanoseconds(nanos)
    }
}
