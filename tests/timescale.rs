extern crate hifitime;
use hifitime::{
    Duration, Epoch, ParsingError, TimeScale, BDT_REF_EPOCH, GPST_REF_EPOCH, GST_REF_EPOCH,
    QZSST_REF_EPOCH,
};
use std::str::FromStr;

#[test]
fn test_from_str() {
    let values = vec![
        ("TAI", TimeScale::TAI),
        ("UTC", TimeScale::UTC),
        ("TT", TimeScale::TT),
        ("TDB", TimeScale::TDB),
        ("GPST", TimeScale::GPST),
        ("GST", TimeScale::GST),
        ("BDT", TimeScale::BDT),
        ("QZSST", TimeScale::QZSST),
        ("IRNSST", TimeScale::IRNSST),
    ];
    for value in values {
        let (descriptor, expected) = value;
        let ts = TimeScale::from_str(descriptor);
        assert!(ts.is_ok());
        let ts = ts.unwrap();
        assert_eq!(ts, expected);
        // test to_str()/format()
        assert_eq!(format!("{ts}"), descriptor);
        // test format(0x)
        let expected: &str = match ts {
            TimeScale::GPST => "GPS",
            TimeScale::GST => "GAL",
            TimeScale::BDT => "BDS",
            TimeScale::QZSST => "QZSS",
            TimeScale::IRNSST => "IRN",
            _ => descriptor, // untouched
        };
        assert_eq!(format!("{ts:x}"), expected);
    }
}

#[test]
fn test_from_rinex_format() {
    /*
     * Test (GNSS) timescales identification
     * from standard 3 letter constellation code
     */
    assert_eq!(TimeScale::from_str("GPS"), Ok(TimeScale::GPST));
    assert_eq!(TimeScale::from_str("GAL"), Ok(TimeScale::GST));
    assert_eq!(TimeScale::from_str("BDS"), Ok(TimeScale::BDT));
    assert_eq!(TimeScale::from_str("QZSS"), Ok(TimeScale::QZSST));
    assert_eq!(TimeScale::from_str("IRN"), Ok(TimeScale::IRNSST));
    assert_eq!(TimeScale::from_str("IRNSST"), Ok(TimeScale::IRNSST));
    // Check error
    assert_eq!(TimeScale::from_str("FAK"), Err(ParsingError::TimeSystem));
}

#[test]
fn test_is_gnss() {
    let ts = TimeScale::GPST;
    assert!(ts.is_gnss());
    let ts = TimeScale::GST;
    assert!(ts.is_gnss());
    let ts = TimeScale::UTC;
    assert!(!ts.is_gnss());
    let ts = TimeScale::TAI;
    assert!(!ts.is_gnss());
    let ts = TimeScale::QZSST;
    assert!(ts.is_gnss());
    let ts = TimeScale::IRNSST;
    assert!(ts.is_gnss());
    assert!(!ts.uses_leap_seconds());
}

#[test]
fn test_default() {
    assert_eq!(TimeScale::default(), TimeScale::TAI);
}

#[test]
fn gh_294_reference_epoch_constants_match_time_scale() {
    // The reference epoch constants are the TAI representation of the same instant as the time
    // scale's own reference epoch, which has a zero duration in that time scale.
    for (time_scale, constant) in [
        (TimeScale::GPST, GPST_REF_EPOCH),
        (TimeScale::QZSST, QZSST_REF_EPOCH),
        (TimeScale::GST, GST_REF_EPOCH),
        (TimeScale::BDT, BDT_REF_EPOCH),
    ] {
        let reference_epoch = time_scale.reference_epoch();
        assert_eq!(reference_epoch.duration, Duration::ZERO, "{time_scale}");
        assert_eq!(reference_epoch.time_scale, time_scale);
        assert_eq!(
            reference_epoch.to_tai_duration(),
            constant.to_tai_duration(),
            "{time_scale}"
        );
        // A duration measured from a time scale's reference epoch must be zero there.
        assert_eq!(
            Epoch::from_duration(Duration::ZERO, time_scale),
            reference_epoch,
            "{time_scale}"
        );
    }
}
