extern crate hifitime;

use hifitime::datetime::{Datetime, FixedOffset};

#[test]
fn tz() {
    let santa_ktz = Datetime::with_offset(
        2017,
        12,
        25,
        00,
        00,
        00,
        00,
        FixedOffset::west_with_hours(10),
    ).expect("Santa failed");
    assert_eq!(format!("{}", santa_ktz), "2017-12-25T00:00:00+10:00");
    let santa_wtz = Datetime::with_offset(
        2017,
        12,
        25,
        00,
        00,
        00,
        00,
        FixedOffset::east_with_hours(10),
    ).expect("Santa failed");
    assert_eq!(format!("{}", santa_wtz), "2017-12-25T00:00:00-10:00");
    assert_eq!(
        format!("{}", santa_wtz.to_utc()),
        "2017-12-24T14:00:00+00:00"
    );
}
