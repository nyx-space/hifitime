#[cfg(feature = "std")]
extern crate core;

use hifitime::{Duration, Epoch, Unit, Weekday};

#[test]
fn test_basic_ops() {
    assert_eq!(Weekday::default(), Weekday::Monday);

    let monday = Weekday::default();

    assert_eq!(monday - 1, Weekday::Sunday);
    assert_eq!(monday - 2, Weekday::Saturday);
    assert_eq!(monday - 3, Weekday::Friday);
    assert_eq!(monday - 4, Weekday::Thursday);
    assert_eq!(monday - 5, Weekday::Wednesday);
    assert_eq!(monday - 6, Weekday::Tuesday);
    assert_eq!(monday - 7, monday);
    assert_eq!(monday - 8, Weekday::Sunday);
    assert_eq!(monday - 9, Weekday::Saturday);
    assert_eq!(monday - 13, Weekday::Tuesday);
    assert_eq!(monday - 14, monday);
    assert_eq!(monday - 15, Weekday::Sunday);

    let i: i8 = -1;
    let weekday: Weekday = i.into();
    assert_eq!(weekday, Weekday::Sunday);
    let i: i8 = -2;
    let weekday: Weekday = i.into();
    assert_eq!(weekday, Weekday::Saturday);
}

#[test]
fn test_weekday_differences() {
    let monday = Weekday::Monday;

    for day_num in 0..15_u8 {
        let day = Weekday::from(day_num);
        let neg_delta: Duration = monday - day;
        let pos_delta: Duration = day - monday;
        // Check reciprocity
        if day_num % 7 == 0 {
            assert_eq!(pos_delta + neg_delta, Duration::ZERO);
        } else {
            assert_eq!(pos_delta + neg_delta, 7 * Unit::Day);
        }
        // Check actual value
        assert_eq!(neg_delta, i64::from(day_num % 7) * Unit::Day);
    }

    // Start in the middle of the week
    for day_num in 0..15_u8 {
        let day = Weekday::from(day_num);
        let neg_delta: Duration = Weekday::Wednesday - day;
        let pos_delta: Duration = day - Weekday::Wednesday;
        // Check reciprocity
        if day_num % 7 == 2 {
            assert_eq!(pos_delta + neg_delta, Duration::ZERO);
        } else {
            assert_eq!(pos_delta + neg_delta, 7 * Unit::Day);
        }
        // Check actual value
        if day_num % 7 <= 2 {
            assert_eq!(pos_delta, i64::from(2 - day_num % 7) * Unit::Day);
        } else {
            assert_eq!(neg_delta, i64::from(day_num % 7 - 2) * Unit::Day);
        }
    }
}

#[test]
fn test_next() {
    let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    assert_eq!(epoch.weekday_utc(), Weekday::Saturday);
    assert_eq!(
        epoch.next(Weekday::Sunday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 3)
    );
    assert_eq!(
        epoch.next(Weekday::Monday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 4)
    );
    assert_eq!(
        epoch.next(Weekday::Tuesday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 5)
    );
    assert_eq!(
        epoch.next(Weekday::Wednesday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 6)
    );
    assert_eq!(
        epoch.next(Weekday::Thursday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 7)
    );
    assert_eq!(
        epoch.next(Weekday::Friday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 8)
    );
    assert_eq!(
        epoch.next(Weekday::Saturday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 9)
    );
}

#[test]
fn test_previous() {
    let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    assert_eq!(
        epoch.previous(Weekday::Friday),
        Epoch::from_gregorian_utc_at_midnight(1988, 1, 1)
    );
    assert_eq!(
        epoch.previous(Weekday::Thursday),
        Epoch::from_gregorian_utc_at_midnight(1987, 12, 31)
    );
    assert_eq!(
        epoch.previous(Weekday::Wednesday),
        Epoch::from_gregorian_utc_at_midnight(1987, 12, 30)
    );
    assert_eq!(
        epoch.previous(Weekday::Tuesday),
        Epoch::from_gregorian_utc_at_midnight(1987, 12, 29)
    );
    assert_eq!(
        epoch.previous(Weekday::Monday),
        Epoch::from_gregorian_utc_at_midnight(1987, 12, 28)
    );
    assert_eq!(
        epoch.previous(Weekday::Sunday),
        Epoch::from_gregorian_utc_at_midnight(1987, 12, 27)
    );
    assert_eq!(
        epoch.previous(Weekday::Saturday),
        Epoch::from_gregorian_utc_at_midnight(1987, 12, 26)
    );
}