//! Polynomial Duration wrapper used in interpolation processes
use crate::Duration;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Polynomials {
    /// Constant offset [Duration], regardless of the interpolation interval
    pub constant: Duration,
    /// Rate or drift in seconds per second (s.s⁻¹) expressed as [Duration] for convenience.
    /// It is a linear scaling factor of the interpolation interval.
    pub rate: Duration,
    /// Acceleration or drift change in s.s⁻², expressed as [Duration] for convenience.
    /// It is a quadratic scaling of the interpolation interval.
    pub accel: Duration,
}

impl Polynomials {
    /// Create a [Polynomials] structure that is only made of a static offset
    pub fn from_constant_offset(constant: Duration) -> Self {
        Self {
            constant,
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomials] structure from a static offset expressed in nanoseconds
    pub fn from_constant_offset_nanoseconds(nanos: f64) -> Self {
        Self {
            constant: Duration::from_nanoseconds(nanos),
            rate: Default::default(),
            accel: Default::default(),
        }
    }

    /// Create a [Polynomials] structure from both static offset and rate of change:
    pub fn from_offset_and_rate(constant: Duration, rate: Duration) -> Self {
        Self {
            constant,
            rate,
            accel: Default::default(),
        }
    }

    /// Create a [Polynomials] structure from a static offset and drift,
    /// in nanoseconds and nanoseconds.s⁻¹
    pub fn from_offset_rate_nanoseconds(offset_ns: f64, drift_ns_s: f64) -> Self {
        Self {
            constant: Duration::from_nanoseconds(offset_ns),
            rate: Duration::from_nanoseconds(drift_ns_s),
            accel: Default::default(),
        }
    }
}
