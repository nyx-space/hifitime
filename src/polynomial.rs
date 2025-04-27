//! Polynomial Duration wrapper used in interpolation processes
use crate::Duration;

use core::fmt;

#[cfg(not(feature = "std"))]
use num_traits::Float;

#[cfg(doc)]
use crate::TimeScale;

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

/// Interpolation [Polynomial] used for example in [TimeScale]
/// maintenance, precise monitoring or conversions.
///
/// (Python documentation hints)
/// :type constant: Duration
/// :type rate: Duration
/// :type accel: Duration
/// :rtype: Polynomial
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "hifitime"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polynomial {
    /// Constant offset [Duration], regardless of the interpolation interval
    pub constant: Duration,
    /// Rate or drift in seconds per second (s.s⁻¹) expressed as [Duration] for convenience.
    /// It is a linear scaling factor of the interpolation interval.
    pub rate: Duration,
    /// Acceleration or drift change in s.s⁻², expressed as [Duration] for convenience.
    /// It is a quadratic scaling of the interpolation interval.
    pub accel: Duration,
}

#[cfg_attr(feature = "python", pymethods)]
impl Polynomial {
    /// Calculate the correction (as [Duration] once again) from [Self] and given
    /// the interpolation time interval
    pub fn correction_duration(&self, time_interval: Duration) -> Duration {
        let dt_s = time_interval.to_seconds();
        let (a0, a1, a2) = (
            self.constant.to_seconds(),
            self.rate.to_seconds(),
            self.accel.to_seconds(),
        );
        Duration::from_seconds(a0 + a1 * dt_s + a2 * dt_s.powi(2))
    }
}

#[cfg(not(feature = "python"))]
impl Polynomial {
    /// Create a [Polynomial] structure that is only made of a static offset
    pub fn from_constant_offset(constant: Duration) -> Self {
        Self {
            constant,
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from a static offset expressed in nanoseconds
    pub fn from_constant_offset_nanoseconds(nanos: f64) -> Self {
        Self {
            constant: Duration::from_nanoseconds(nanos),
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from both static offset and rate of change:
    pub fn from_offset_and_rate(constant: Duration, rate: Duration) -> Self {
        Self {
            constant,
            rate,
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from a static offset and drift,
    /// in nanoseconds and nanoseconds.s⁻¹
    pub fn from_offset_rate_nanoseconds(offset_ns: f64, drift_ns_s: f64) -> Self {
        Self {
            constant: Duration::from_nanoseconds(offset_ns),
            rate: Duration::from_nanoseconds(drift_ns_s),
            accel: Default::default(),
        }
    }
}

impl fmt::Display for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "offset={} rate={} accel={}",
            self.constant, self.rate, self.accel
        )
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Polynomial {
    /// Create a [Polynomial] structure that is only made of a static offset
    #[classmethod]
    pub fn from_constant_offset(_cls: &Bound<'_, PyType>, constant: Duration) -> Self {
        Self {
            constant,
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from a static offset expressed in nanoseconds
    #[classmethod]
    pub fn from_constant_offset_nanoseconds(_cls: &Bound<'_, PyType>, nanos: f64) -> Self {
        Self {
            constant: Duration::from_nanoseconds(nanos),
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from both static offset and rate of change:
    #[classmethod]
    pub fn from_offset_and_rate(
        _cls: &Bound<'_, PyType>,
        constant: Duration,
        rate: Duration,
    ) -> Self {
        Self {
            constant,
            rate,
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from a static offset and drift,
    /// in nanoseconds and nanoseconds.s⁻¹
    #[classmethod]
    pub fn from_offset_rate_nanoseconds(
        _cls: &Bound<'_, PyType>,
        offset_ns: f64,
        drift_ns_s: f64,
    ) -> Self {
        Self {
            constant: Duration::from_nanoseconds(offset_ns),
            rate: Duration::from_nanoseconds(drift_ns_s),
            accel: Default::default(),
        }
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }

    #[cfg(feature = "python")]
    fn __eq__(&self, other: Self) -> bool {
        *self == other
    }
}
