extern crate hifitime;
use hifitime::{ParsingError, TimeScale};
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
    ];
    for value in values {
        let (descriptor, expected) = value;
        let ts = TimeScale::from_str(descriptor);
        assert_eq!(ts.is_ok(), true);
        let ts = ts.unwrap();
        assert_eq!(ts, expected);
        // test to_str()/format()
        assert_eq!(format!("{}", ts), descriptor);
        // test format(0x)
        let expected: &str = match ts {
            TimeScale::GPST => "GPS",
            TimeScale::GST => "GAL",
            TimeScale::BDT => "BDS",
            TimeScale::QZSST => "QZSS",
            _ => descriptor, // untouched
        };
        assert_eq!(format!("{:x}", ts), expected);
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
}

#[test]
fn test_default() {
    assert_eq!(TimeScale::default(), TimeScale::TAI);
}
