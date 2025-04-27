use hifitime::{
    prelude::{Duration, Epoch, TimeScale, Unit},
    Polynomial,
};

use std::str::FromStr;

// When the polynomial model is (a0, 0, 0) the epoch stirring results
// in an a0 static offset, at all times.
#[test]
fn epoch_stirring_static_offset() {
    let (a0, _, _) = (1.0, 0.0, 0.0);

    // Random reference epoch in a given timescale
    let t_ref_gpst = Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap();

    let polynomial = Polynomial::from_constant_offset_nanoseconds(a0);

    // Same date time, many timescales
    let t_gst = Epoch::from_str("2020-01-01T00:00:00 GST").unwrap();
    let t_gpst = Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap();

    // Testing positive offset from REF.
    // "now" is past polynomial publication.
    for offset_from_ref in [
        Duration::ZERO,
        10.0 * Unit::Nanosecond,
        10.0 * Unit::Second,
        1.0 * Unit::Minute,
        1.0 * Unit::Hour,
        1.0 * Unit::Day,
    ] {
        // Test forward transform: |GST->GPST|
        // Starting from same date time, different timescale: apply positive offset
        let t_gst = t_gst + offset_from_ref;

        let t_gst_gpst = t_gst
            .precise_timescale_conversion(true, t_ref_gpst, polynomial, TimeScale::GPST)
            .unwrap();

        assert_eq!(
            t_gst_gpst.time_scale,
            TimeScale::GPST,
            "Timescale conversion did not work!"
        );

        // Test (forward) time delta
        let dt = t_gst_gpst - t_ref_gpst;

        // What ever the time offset might be, the stirring should only produce an a0 offset
        assert_eq!(dt, offset_from_ref - a0 * Unit::Nanosecond); // (-)!

        // Linear operation: reciprocity applies at all times
        let reciprocal = t_gst_gpst
            .precise_timescale_conversion(false, t_ref_gpst, polynomial, TimeScale::GST)
            .unwrap();

        assert_eq!(reciprocal, t_gst);

        // Test backward transform: |GST<-GPST|
        // Starting from same date time, different timescale: apply positive offset
        let t_gpst = t_gpst + offset_from_ref;

        let t_gpst_gst = t_gpst
            .precise_timescale_conversion(false, t_ref_gpst, polynomial, TimeScale::GST)
            .unwrap();

        assert_eq!(
            t_gpst_gst.time_scale,
            TimeScale::GST,
            "Timescale conversion did not work!"
        );

        // Test (backward) time delta
        let dt = t_gpst_gst - t_ref_gpst;

        // What ever the time offset might be, the stirring should only produce an a0 offset
        assert_eq!(dt, offset_from_ref + a0 * Unit::Nanosecond); // (+)!

        // Linear operation: reciprocity applies at all times
        let reciprocal = t_gpst_gst
            .precise_timescale_conversion(true, t_ref_gpst, polynomial, TimeScale::GPST)
            .unwrap();

        assert_eq!(reciprocal, t_gpst);
    }

    // Testing negative offset from REF.
    // "now" is before polynomial publication, which is also valid and should be supported.
    for offset_from_ref in [
        Duration::ZERO,
        10.0 * Unit::Nanosecond,
        10.0 * Unit::Second,
        1.0 * Unit::Minute,
        1.0 * Unit::Hour,
        1.0 * Unit::Day,
    ] {
        // Test forward transform: |GST->GPST|
        // Starting from same date time, different timescale: apply negative offset
        let t_gst = t_gst - offset_from_ref;

        let t_gst_gpst = t_gst
            .precise_timescale_conversion(true, t_ref_gpst, polynomial, TimeScale::GPST)
            .unwrap();

        assert_eq!(
            t_gst_gpst.time_scale,
            TimeScale::GPST,
            "Timescale conversion did not work!"
        );

        // Test (forward) time delta
        let dt = t_gst_gpst - t_ref_gpst;

        // What ever the time offset might be, the stirring should only produce an a0 offset
        assert_eq!(dt, -(a0 * Unit::Nanosecond + offset_from_ref));

        // Linear operation: reciprocity applies at all times
        let reciprocal = t_gst_gpst
            .precise_timescale_conversion(false, t_ref_gpst, polynomial, TimeScale::GST)
            .unwrap();

        assert_eq!(reciprocal, t_gst);

        // Test backward transform: |GST<-GPST|
        // Starting from same date time, different timescale: apply positive offset
        let t_gpst = t_gpst - offset_from_ref;

        let t_gpst_gst = t_gpst
            .precise_timescale_conversion(false, t_ref_gpst, polynomial, TimeScale::GST)
            .unwrap();

        assert_eq!(
            t_gpst_gst.time_scale,
            TimeScale::GST,
            "Timescale conversion did not work!"
        );

        // Test (backward) time delta
        let dt = t_gpst_gst - t_ref_gpst;

        // What ever the time offset might be, the stirring should only produce an a0 offset
        assert_eq!(dt, -(a0 * Unit::Nanosecond - offset_from_ref));

        // Linear operation: reciprocity applies at all times
        let reciprocal = t_gpst_gst
            .precise_timescale_conversion(true, t_ref_gpst, polynomial, TimeScale::GPST)
            .unwrap();

        assert_eq!(reciprocal, t_gpst);
    }
}

// When the polynomial model is (a0, a1, 0) the epoch stirring results
// in an a0 static offset + a linear scale of the time interpolation interval.
#[test]
fn epoch_stirring_offset_drift() {
    let (a0, a1, _) = (1.0, 1.0, 0.0);

    // Random reference epoch in a given timescale
    let t_ref_gpst = Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap();

    let polynomial = Polynomial::from_offset_and_rate(
        Duration::from_nanoseconds(a0),
        Duration::from_nanoseconds(a1),
    );

    // Same date time, many timescales
    let t_gst = Epoch::from_str("2020-01-01T00:00:00 GST").unwrap();
    let t_gpst = Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap();

    // Testing positive offset from REF.
    // "now" is past polynomial publication.
    for offset_from_ref in [
        Duration::ZERO,
        10.0 * Unit::Nanosecond,
        10.0 * Unit::Second,
        1.0 * Unit::Minute,
        1.0 * Unit::Hour,
        1.0 * Unit::Day,
    ] {
        // Test forward transform: |GST->GPST|
        // Starting from same date time, different timescale: apply positive offset
        let t_gst = t_gst + offset_from_ref;

        let t_gst_gpst = t_gst
            .precise_timescale_conversion(true, t_ref_gpst, polynomial, TimeScale::GPST)
            .unwrap();

        assert_eq!(
            t_gst_gpst.time_scale,
            TimeScale::GPST,
            "Timescale conversion did not work!"
        );

        // Test (forward) time delta
        let dt = t_gst_gpst - t_ref_gpst;

        let expected = offset_from_ref - a0 * Unit::Nanosecond;
        let expected = expected - a1 * 1E-9 * Unit::Second * dt.to_seconds();
        assert!(dt.total_nanoseconds() - expected.total_nanoseconds() < 1);

        // Linear operation: reciprocity applies at all times
        let reciprocal = t_gst_gpst
            .precise_timescale_conversion(false, t_ref_gpst, polynomial, TimeScale::GST)
            .unwrap();

        assert!(reciprocal.duration.total_nanoseconds() - t_gst.duration.total_nanoseconds() < 1);

        // Test backward transform: |GST<-GPST|
        // Starting from same date time, different timescale: apply positive offset
        let t_gpst = t_gpst - offset_from_ref;

        let t_gpst_gst = t_gpst
            .precise_timescale_conversion(false, t_ref_gpst, polynomial, TimeScale::GST)
            .unwrap();

        assert_eq!(
            t_gpst_gst.time_scale,
            TimeScale::GST,
            "Timescale conversion did not work!"
        );

        // Test (backward) time delta
        let dt = t_gpst_gst - t_ref_gpst;

        let expected = offset_from_ref + a0 * Unit::Nanosecond;
        let expected = expected + a1 * 1E-9 * Unit::Second * dt.to_seconds();
        assert!(dt.total_nanoseconds() - expected.total_nanoseconds() < 1);

        // Linear operation: reciprocity applies at all times
        let reciprocal = t_gpst_gst
            .precise_timescale_conversion(true, t_ref_gpst, polynomial, TimeScale::GPST)
            .unwrap();

        assert_eq!(reciprocal, t_gpst);
    }
}
