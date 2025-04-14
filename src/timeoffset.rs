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

/// [TimeOffset] allows to describe the behavior of a tracked [TimeScale]
/// with resepect to a reference [TimeScale] as typically done in [TimeScale] maintenance
/// or monitoring. For example, |GPST-UTC| when referencing [TimeScale::GPST]
/// to [TimeScale::UTC] timescale. It allows precise conversion between both [TimeScale]s
/// and works both ways.
/// The reference and conversion instants are encoded as [Epoch]s which limits their description
/// to 1 nanosecond accuracy. The conversion methods:
/// [Self::time_correction_nanos] and [Self::time_correction_seconds] are not limited to 1 ns.
#[derive(Copy, Clone, PartialEq)]
pub struct TimeOffset {
    /// Left hand side (compared) [TimeScale]
    compared_lhs: TimeScale,
    /// Right hand side (reference) [TimeScale]
    reference_rhs: TimeScale,
    /// Weekly reference time (counter, nanoseconds)
    ref_epoch_tow_nanos: (u32, u64),
    /// Polynomials (s, s.s⁻¹, s.s⁻²)
    polynomials: (f64, f64, f64),
}

impl TimeOffset {
    /// Define a new [TimeOffset].
    /// ## Input
    /// - ref_epoch: Reference [Epoch] expressed in left-hand side [TimeScale]
    /// - reference_ts: [TimeScale] to which left-hand side is referenced to.
    /// - polynomials: for interpolation calculations
    pub fn from_reference_epoch(
        ref_epoch: Epoch,
        reference_ts: TimeScale,
        polynomials: (f64, f64, f64),
    ) -> Result<Self, TimeOffsetError> {
        if ref_epoch.time_scale == reference_ts {
            // illegal / invalid operation
            return Err(TimeOffsetError::IdenticalTimescales);
        }

        let ref_epoch_tow_nanos = ref_epoch.to_time_of_week();

        Ok(Self {
            ref_epoch_tow_nanos,
            polynomials,
            reference_rhs: reference_ts,
            compared_lhs: ref_epoch.time_scale,
        })
    }

    /// Define a new [TimeOffset].
    /// ## Input
    /// - ref_epoch_tow_nanos: reference [Epoch::to_time_of_week]
    /// expressed in left-hand side [TimeScale]
    /// - compared_lhs: left-hand side [TimeScale]
    /// - reference_rhs: reference [TimeScale]
    /// - polynomials: for interpolation calculations
    pub fn from_reference_time_of_week(
        ref_epoch_tow_nanos: (u32, u64),
        compared_lhs: TimeScale,
        reference_rhs: TimeScale,
        polynomials: (f64, f64, f64),
    ) -> Result<Self, TimeOffsetError> {
        if compared_lhs == reference_rhs {
            // illegal / invalid operation
            return Err(TimeOffsetError::IdenticalTimescales);
        }

        Ok(Self {
            compared_lhs,
            reference_rhs,
            ref_epoch_tow_nanos,
            polynomials,
        })
    }

    /// Returns both [TimeScale]s this [TimeOffset] supports. Meaning that
    /// it allows conversion to either one.
    pub fn supported_timescales(&self) -> (TimeScale, TimeScale) {
        (self.compared_lhs, self.reference_rhs)
    }

    /// Update this [TimeOffset] with new reference epoch and polynomials.
    /// ## Input
    /// - ref_epoch_tow_nanos: reference [Epoch::to_time_of_week] that needs to
    /// remain expressed in left-hand side [TimeScale] for this structure to remain correct.
    /// - polynomials: updated polynomial terms used in interpolation.
    pub fn update_mut(&mut self, ref_epoch_tow_nanos: (u32, u64), polynomials: (f64, f64, f64)) {
        self.ref_epoch_tow_nanos = ref_epoch_tow_nanos;
        self.polynomials = polynomials;
    }

    /// Define a new [TimeOffset] with new reference [TimeScale], while preserving other components.
    /// For example, this would be [TimeScale::UTC] in the |[TimeScale::GPST] - [TimeScale::UTC]| tracking.
    /// Modifying one of the [TimeScale]s will require modification of the polynomial terms or reference epoch
    /// for this structure to remain correct.
    pub fn with_reference_timescale(mut self, ts: TimeScale) -> Self {
        self.reference_rhs = ts;
        self
    }

    /// Define a new [TimeOffset] with new left-hand side [TimeScale], while preserving other components.
    /// For example, this would be [TimeScale::GPST] in the |[TimeScale::GPST] - [TimeScale::UTC]| tracking.
    /// Modifying one of the [TimeScale]s will require modification of the polynomial terms or reference epoch
    /// for this structure to remain correct.
    pub fn with_comparison_timescale(mut self, ts: TimeScale) -> Self {
        self.compared_lhs = ts;
        self
    }

    /// Define a new [TimeOffset] with new reference [Epoch::to_time_of_week] (in nanoseconds), while preserving other components.
    /// This most likely should be tied to a polynoliam terms update: [Self::with_polynomials].
    /// If new reference [Epoch] is expressed in the current left-hand side [TimeScale] the structure remains correct.
    /// Otherwise, you should modify the left-hand side [TimeScale] with one of the prooposed methods for this structure to remain valid.
    pub fn with_reference_time_of_week_nanos(mut self, ref_epoch_tow_nanos: (u32, u64)) -> Self {
        self.ref_epoch_tow_nanos = ref_epoch_tow_nanos;
        self
    }

    /// Define a new [TimeOffset] with new reference [Epoch] with 1 ns precision.
    /// This most likely should be tied to a polynoliam terms update: [Self::with_polynomials].
    /// [Epoch] needs to be expressed in left-hand side [TimeScale] for this operation to be valid.
    pub fn with_reference_epoch(mut self, ref_epoch: Epoch) -> Result<Self, TimeOffsetError> {
        if ref_epoch.time_scale != self.compared_lhs {
            return Err(TimeOffsetError::InvalidTimescale);
        }

        self.ref_epoch_tow_nanos = ref_epoch.to_time_of_week();
        Ok(self)
    }

    /// Define a new [TimeOffset] with new polynomials, while preserving other components.
    pub fn with_polynomials(mut self, polynomials: (f64, f64, f64)) -> Self {
        self.polynomials = polynomials;
        self
    }

    /// Returns the total number of nanoseconds to apply to convert this [Epoch] to other [TimeScale].
    /// ## Input
    /// - t: interpolation instant expressed as [Epoch] with 1ns accuracy.
    /// It needs to be expressed in either of [Self::supported_timescales] for this operation to be valid.
    /// The correction is calculated for the other supported [TimeScale].
    pub fn time_correction_nanos(&self, t: Epoch) -> Result<f64, TimeOffsetError> {
        if t.time_scale != self.compared_lhs && t.time_scale != self.reference_rhs {
            return Err(TimeOffsetError::NotSupportedTimescale);
        }

        let (t_week, t_nanos) = t.to_time_of_week();
        let (ref_week, ref_nanos) = self.ref_epoch_tow_nanos;

        // make sure this falls within a week duration (at most)
        if t_week > ref_week + 1 || ref_week > t_week + 1 {
            return Err(TimeOffsetError::OutdatedTimeOffset);
        }

        let (a0, a1, a2) = self.polynomials;
        let dt_s = (t_nanos as f64 - ref_nanos as f64) * 1.0E-9;
        let dt_s = a0 + a1 * dt_s + a2 * dt_s.powi(2);

        // support back & forth conversion
        if t.time_scale == self.reference_rhs {
            Ok(-dt_s * 1.0E9)
        } else {
            Ok(dt_s * 1.0E9)
        }
    }

    /// Returns the total number of nanoseconds to apply to convert this [Epoch] to other [TimeScale].
    /// ## Input
    /// - t: interpolation instant expressed as [Epoch] with 1ns accuracy.
    /// It needs to be either of [Self::supported_timescales] for this operation to be valid.
    /// The correction is calculated for the other supported [TimeScale].
    pub fn time_correction_seconds(&self, t: Epoch) -> Result<f64, TimeOffsetError> {
        let correction_nanos = self.time_correction_nanos(t)?;
        Ok(correction_nanos * 1.0E-9)
    }

    /// Convert provided [Epoch] expressed in either of [Self::supported_timescales],
    /// to other supported [TimeScale]. This operation has a 1 ns accuracy.
    /// ## Input
    /// - t: interpolation instant expressed as [Epoch] with 1ns accuracy.
    /// It needs to be either of [Self::supported_timescales] for this operation to be valid.
    /// The correction is calculated for the other supported [TimeScale].
    pub fn epoch_time_correction(&self, t: Epoch) -> Result<Epoch, TimeOffsetError> {
        let correction_nanos = self.time_correction_nanos(t)?;
        let corrected = t + correction_nanos * Unit::Nanosecond;
        // perform the swap & return
        Ok(corrected.to_time_scale(self.reference_rhs))
    }
}

#[cfg(test)]
mod test {
    use crate::{Epoch, TimeOffset, TimeScale, Unit};

    #[test]
    fn test_1ns_time_offset() {
        // Tests the TimeOffset API with values slightly above hifitime precision.
        let polynomials = (1E-9, 0.0, 0.0);

        let known_timescales = [
            TimeScale::UTC,
            TimeScale::TAI,
            TimeScale::GPST,
            TimeScale::GST,
            TimeScale::BDT,
            TimeScale::QZSST,
        ];

        for ref_ts in known_timescales.iter() {
            for lhs_ts in known_timescales.iter() {
                // random t_ref in LHS timescale
                let t_ref = Epoch::from_gregorian(2020, 1, 1, 0, 0, 0, 0, *lhs_ts);

                // create valid TimeOffset
                let time_offset = TimeOffset::from_reference_epoch(t_ref, *ref_ts, polynomials);

                if ref_ts != lhs_ts {
                    // valid use case
                    let time_offset = time_offset.unwrap();

                    ///////////////////////////////////////
                    // 1. some time LATER within that week
                    ///////////////////////////////////////
                    let instant = t_ref + 1.0 * Unit::Day;

                    // this is a simple case of a static offset
                    let dt_s = time_offset.time_correction_seconds(instant).unwrap();
                    assert_eq!(dt_s, polynomials.0);

                    let dt_nanos = time_offset.time_correction_nanos(instant).unwrap();
                    assert_eq!(dt_nanos, polynomials.0 * 1E9);

                    // Test that conversion did work
                    let converted = time_offset.epoch_time_correction(instant).unwrap();

                    assert_eq!(
                        converted.time_scale, *ref_ts,
                        "epoch_time_correction did not translate timescale!"
                    );

                    // this is a simple case of a static offset
                    let dt = (converted - instant).to_seconds();
                    assert_eq!(dt, polynomials.0);

                    /////////////////////////////////////////////////////////
                    // 2. some time BEFORE within that week (works both ways)
                    /////////////////////////////////////////////////////////
                    let instant = t_ref - 1.0 * Unit::Day;

                    // this is a simple case of a static offset
                    let dt_s = time_offset.time_correction_seconds(instant).unwrap();
                    assert_eq!(dt_s, polynomials.0); // same static offset

                    let dt_nanos = time_offset.time_correction_nanos(instant).unwrap();
                    assert_eq!(dt_nanos, polynomials.0 * 1E9); // same static offset

                    // Test that conversion did work
                    let converted = time_offset.epoch_time_correction(instant).unwrap();

                    assert_eq!(
                        converted.time_scale, *ref_ts,
                        "epoch_time_correction did not translate timescale!"
                    );

                    // this is a simple case of a static offset
                    let dt = (converted - instant).to_seconds();
                    assert_eq!(dt, polynomials.0);
                } else {
                    // invalid use case
                    assert!(time_offset.is_err());
                }
            }
        }
    }

    #[test]
    fn test_1ns_time_offset_drift() {
        // Tests the TimeOffset API with values slightly above hifitime precision.
        let (a0, a1, a2) = (1E-9, 1E-10, 1E-15);

        let known_timescales = [
            TimeScale::UTC,
            TimeScale::TAI,
            TimeScale::GPST,
            TimeScale::GST,
            TimeScale::BDT,
            TimeScale::QZSST,
        ];

        for ref_ts in known_timescales.iter() {
            for lhs_ts in known_timescales.iter() {
                // random t_ref in LHS timescale
                let t_ref = Epoch::from_gregorian(2020, 1, 1, 0, 0, 0, 0, *lhs_ts);

                // create valid TimeOffset
                let time_offset = TimeOffset::from_reference_epoch(t_ref, *ref_ts, (a0, a1, a2));

                if ref_ts != lhs_ts {
                    // valid use case
                    let time_offset = time_offset.unwrap();

                    // some time later within that week
                    let instant = t_ref + 1.0 * Unit::Day;

                    // Time offset + drift so time difference is integrated
                    let interval_s = (instant - t_ref).to_seconds();
                    let expected_s = a0 + a1 * interval_s.powi(2) + a2 * interval_s.powi(2);

                    let dt_s = time_offset.time_correction_seconds(instant).unwrap();
                    assert!((dt_s - expected_s) < 1E-9);

                    // Test that conversion did work
                    let converted = time_offset.epoch_time_correction(instant).unwrap();

                    assert_eq!(
                        converted.time_scale, *ref_ts,
                        "epoch_time_correction did not translate timescale!"
                    );

                    // // Time offset only, so time difference does not impact
                    // // and both timescales are offset by a static a0 value
                    // let dt = (converted - instant).to_seconds();
                    // assert_eq!(dt, polynomials.0);
                } else {
                    // invalid use case
                    assert!(time_offset.is_err());
                }
            }
        }
    }

    #[test]
    fn test_sub_nano_time_offset() {
        // for what it's worth.. tests that
        // Epoch is not translated if a0 < 1ns which is below Epoch
        // precision (for all timescales).
        let polynomials = (1E-10, 0.0, 0.0);

        let known_timescales = [
            TimeScale::UTC,
            TimeScale::TAI,
            TimeScale::GPST,
            TimeScale::GST,
            TimeScale::BDT,
            TimeScale::QZSST,
        ];

        for ref_ts in known_timescales.iter() {
            for lhs_ts in known_timescales.iter() {
                // random t_ref in LHS timescale
                let t_ref = Epoch::from_gregorian(2020, 1, 1, 0, 0, 0, 0, *lhs_ts);

                // create valid TimeOffset
                let time_offset = TimeOffset::from_reference_epoch(t_ref, *ref_ts, polynomials);

                if ref_ts != lhs_ts {
                    // valid use case
                    let time_offset = time_offset.unwrap();

                    // some time later within that week
                    let instant = t_ref + 1.0 * Unit::Day;

                    // API should work
                    let _ = time_offset.time_correction_seconds(instant).unwrap();

                    // Test that conversion did work
                    let converted = time_offset.epoch_time_correction(instant).unwrap();

                    assert_eq!(
                        converted.time_scale, *ref_ts,
                        "epoch_time_correction did not translate timescale!"
                    );

                    // Epoch should remain the same because a0 is below current Hifitime precision
                    let initial_gregorian = instant.to_gregorian_utc();
                    let converted_gregorian = converted.to_gregorian_utc();
                    assert_eq!(initial_gregorian, converted_gregorian);
                } else {
                    // invalid use case
                    assert!(time_offset.is_err());
                }
            }
        }
    }
}
