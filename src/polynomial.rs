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

impl From<(f64, f64, f64)> for Polynomial {
    /// Converts (f64, f64, f64) triplet, oftentimes
    /// noted (a0, a1, a2) as (offset (s), drift (s.s⁻¹), drift change (s.s⁻²))
    /// to [Polynomial] structure, that allows precise [TimeScale] translation.
    fn from(triplet: (f64, f64, f64)) -> Self {
        Self {
            constant: Duration::from_seconds(triplet.0),
            rate: Duration::from_seconds(triplet.1),
            accel: Duration::from_seconds(triplet.2),
        }
    }
}

impl From<Polynomial> for (f64, f64, f64) {
    /// Converts [Polynomial] to (f64, f64, f64) triplet, oftentimes
    /// noted (a0, a1, a2) as (offset (s), drift (s.s⁻¹), drift change (s.s⁻²)).
    fn from(polynomial: Polynomial) -> Self {
        (
            polynomial.constant.to_seconds(),
            polynomial.rate.to_seconds(),
            polynomial.accel.to_seconds(),
        )
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Polynomial {
    /// Calculate the correction (as [Duration] once again) from [Self] and given
    /// the interpolation time interval
    /// :type time_interval: Duration
    /// :rtype: Duration
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
    /// :type constant: Duration
    /// :rtype: Polynomial
    #[pyo3(name = "from_constant_offset")]
    #[classmethod]
    pub fn py_from_constant_offset(_cls: &Bound<'_, PyType>, constant: Duration) -> Self {
        Self {
            constant,
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from a static offset expressed in nanoseconds
    /// :type nanos: float
    /// :rtype: Polynomial
    #[pyo3(name = "from_constant_offset_nanoseconds")]
    #[classmethod]
    pub fn py_from_constant_offset_nanoseconds(_cls: &Bound<'_, PyType>, nanos: f64) -> Self {
        Self {
            constant: Duration::from_nanoseconds(nanos),
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomial] structure from both static offset and rate of change:
    /// :type constant: Duration
    /// :type rate: Duration
    /// :rtype: Polynomial
    #[pyo3(name = "from_offset_and_rate")]
    #[classmethod]
    pub fn py_from_offset_and_rate(
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

    /// Create a [Polynomial] structure from a static offset and drift, in nanoseconds and nanoseconds.s⁻¹
    /// :type offset_ns: float
    /// :type drift_ns_s: float
    /// :rtype: Polynomial
    #[pyo3(name = "from_offset_rate_nanoseconds")]
    #[classmethod]
    pub fn py_from_offset_rate_nanoseconds(
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
