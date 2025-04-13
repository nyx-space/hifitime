use crate::{Epoch, TimeScale, Unit};

/// Errors during [TimeOffset] exploitation
#[derive(Debug)]
pub enum TimeOffsetError {
    /// Timescales should not be identical when defining a new [TimeOffset]!
    IdenticalTimescales,
    /// Epoch is not expressed in one of the two supported [TimeScale]s!
    NotSupportedTimescale,
    /// Epoch should be expressed in the left-hand side [TimeScale].
    InvalidTimescale,
    /// [TimeOffset] is outdated and should not apply (difference is too large): should be updated weekly at least!
    OutdatedTimeOffset,
}

/// [TimeOffset] used in [TimeShift]ing operations
#[derive(Copy, Clone, PartialEq)]
pub struct TimeOffset {
    /// Left hand side (compared) [TimeScale]
    lhs: TimeScale,
    /// Right hand side (reference) [TimeScale]
    rhs: TimeScale,
    /// Weekly reference time (counter, nanoseconds)
    t_ref_nanos: (u32, u64),
    /// Polynomials (s, s.s⁻¹, s.s⁻²)
    polynomials: (f64, f64, f64),
}

impl TimeOffset {
    /// Define a new [TimeOffset] from this reference [Epoch] expressed
    /// in left-hand side [TimeScale] to rhs [TimeScale]
    pub fn from_reference_epoch(
        t_ref: Epoch,
        rhs: TimeScale,
        polynomials: (f64, f64, f64),
    ) -> Result<Self, TimeOffsetError> {
        if t_ref.time_scale == rhs {
            return Err(TimeOffsetError::IdenticalTimescales);
        }

        let t_ref_nanos = t_ref.to_time_of_week();

        Ok(Self {
            rhs,
            t_ref_nanos,
            polynomials,
            lhs: t_ref.time_scale,
        })
    }

    /// Define a new [TimeOffset] from reference time of week and other components
    pub fn from_reference_time_of_week(
        t_ref_nanos: (u32, u64),
        lhs: TimeScale,
        rhs: TimeScale,
        polynomials: (f64, f64, f64),
    ) -> Result<Self, TimeOffsetError> {
        if lhs == rhs {
            return Err(TimeOffsetError::IdenticalTimescales);
        }

        Ok(Self {
            lhs,
            rhs,
            t_ref_nanos,
            polynomials,
        })
    }

    /// Define a new |[TimeScale::GPST] - [TimeScale::UTC]| [TimeOffset] from time of week components
    pub fn from_gpst_utc_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(t_ref_nanos, TimeScale::GPST, TimeScale::UTC, polynomials)
            .unwrap()
    }

    /// Define a new |[TimeScale::GST] - [TimeScale::UTC]| [TimeOffset] from time of week components
    pub fn from_gst_utc_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(t_ref_nanos, TimeScale::GST, TimeScale::UTC, polynomials)
            .unwrap()
    }

    /// Define a new |[TimeScale::GST] - [TimeScale::GPST]| [TimeOffset] from time of week components
    pub fn from_gst_gpst_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(t_ref_nanos, TimeScale::GST, TimeScale::GPST, polynomials)
            .unwrap()
    }

    /// Define a new |[TimeScale::BDT] - [TimeScale::UTC]| [TimeOffset] from time of week components
    pub fn from_bdt_utc_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(t_ref_nanos, TimeScale::BDT, TimeScale::UTC, polynomials)
            .unwrap()
    }

    /// Define a new |[TimeScale::BDT] - [TimeScale::GST]| [TimeOffset] from time of week components
    pub fn from_bdt_gst_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(t_ref_nanos, TimeScale::BDT, TimeScale::GST, polynomials)
            .unwrap()
    }

    /// Define a new |[TimeScale::QZSST] - [TimeScale::GPST]| [TimeOffset] from time of week components
    pub fn from_qzsst_gpst_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(
            t_ref_nanos,
            TimeScale::QZSST,
            TimeScale::GPST,
            polynomials,
        )
        .unwrap()
    }

    /// Define a new  |[TimeScale::QZSST] - [TimeScale::UTC]|  [TimeOffset] from time of week components
    pub fn from_qzsst_utc_time_of_week(
        t_ref_nanos: (u32, u64),
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self::from_reference_time_of_week(
            t_ref_nanos,
            TimeScale::QZSST,
            TimeScale::UTC,
            polynomials,
        )
        .unwrap()
    }

    /// Returns both [TimeScale]s this [TimeOffset] allows converting to.
    pub fn supported_timescales(&self) -> (TimeScale, TimeScale) {
        (self.lhs, self.rhs)
    }

    /// Update this [TimeOffset] with new reference epoch and polynomials.
    /// NB: this should be expressed in the left-hand side [TimeScale] and we have no means
    /// to verify that.
    pub fn update_mut(&mut self, t_ref_nanos: (u32, u64), polynomials: (f64, f64, f64)) {
        self.t_ref_nanos = t_ref_nanos;
        self.polynomials = polynomials;
    }

    /// Define a new [TimeOffset] with new reference [TimeScale], while preserving other components.
    /// NB: this should be expressed in the left-hand side [TimeScale] and we do not verify it!
    pub fn with_reference_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.rhs = ts;
        s
    }

    /// Define a new [TimeOffset] with new left-hand side [TimeScale], while preserving other components.
    /// This needs to be coupled to either [Self::with_reference_time_of_week_nanos] or
    /// [Self::with_reference_epoch] to remain correct.
    pub fn with_lhs_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.lhs = ts;
        s
    }

    /// Define a new [TimeOffset] with new reference time of week (in nanoseconds), while preserving other components.
    pub fn with_reference_time_of_week_nanos(&self, t_ref_nanos: (u32, u64)) -> Self {
        let mut s = self.clone();
        s.t_ref_nanos = t_ref_nanos;
        s
    }

    /// Define a new [TimeOffset] with new reference [Epoch] with 1 ns precision.
    pub fn with_reference_epoch(&self, t_ref: Epoch) -> Result<Self, TimeOffsetError> {
        let mut s = self.clone();

        if t_ref.time_scale != self.lhs {
            return Err(TimeOffsetError::InvalidTimescale);
        }

        s.t_ref_nanos = t_ref.to_time_of_week();
        Ok(s)
    }

    /// Define a new [TimeOffset] with new polynomials, while preserving other components.
    pub fn with_polynomials(&self, polynomials: (f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.polynomials = polynomials;
        s
    }

    /// Returns the total number of nanoseconds to apply to convert this [Epoch]
    /// into either of [Self::supported_timescales].
    /// NB:
    /// - `t` must be expressed in either of [Self::supported_timescales].
    /// - `t` should fall within the reference week, otherwise this will give invalid results.
    pub fn time_correction_nanos(&self, t: Epoch) -> Result<f64, TimeOffsetError> {
        if t.time_scale != self.lhs && t.time_scale != self.rhs {
            return Err(TimeOffsetError::NotSupportedTimescale);
        }

        let (t_week, t_nanos) = t.to_time_of_week();
        let (ref_week, ref_nanos) = self.t_ref_nanos;

        // make sure this falls within a week duration (at most)
        if t_week > ref_week + 1 || ref_week > t_week + 1 {
            return Err(TimeOffsetError::OutdatedTimeOffset);
        }

        let (a0, a1, a2) = self.polynomials;
        let dt_s = (t_nanos as f64 - ref_nanos as f64) * 1.0E-9;
        let dt_s = a0 + a1 * dt_s + a2 * dt_s.powi(2);

        // support back & forth conversion
        if t.time_scale == self.rhs {
            Ok(-dt_s)
        } else {
            Ok(dt_s)
        }
    }

    /// Returns the total number of nanoseconds to apply to convert this [Epoch]
    /// into either of [Self::supported_timescales].
    /// NB:
    /// - `t` must be expressed in either of [Self::supported_timescales].
    /// - `t` should fall within the reference week, otherwise this will give invalid results.
    pub fn time_correction_seconds(&self, t: Epoch) -> Result<f64, TimeOffsetError> {
        let correction_nanos = self.time_correction_nanos(t)?;
        Ok(correction_nanos * 1.0E-9)
    }

    /// Convert this [Epoch] to desired [TimeScale], with 1 nanosecond precision,
    /// using this [TimeOffset] definitions.
    /// NB:
    /// - `t` can be originally expressed in any supported [TimeScale]
    /// - `t` should fall within the reference week, otherwise this will give invalid results.
    pub fn epoch_time_correction(&self, t: Epoch) -> Result<Epoch, TimeOffsetError> {
        let correction_nanos = self.time_correction_nanos(t)?;
        let corrected = t + correction_nanos * Unit::Nanosecond;
        // perform the swap & return
        Ok(corrected.to_time_scale(self.rhs))
    }
}

pub trait TimeShift {
    /// Convert and shift [TimeScale] using provided [TimeOffset] structure.
    fn time_offset(&self, timeoffset: TimeOffset) -> Self
    where
        Self: Sized;

    /// Convert and shift [TimeScale] using provided [TimeOffset] structure.
    fn time_offset_mut(&mut self, timeoffset: TimeOffset);
}
