#[cfg(feature = "std")]
extern crate core;

use hifitime::Weekday;

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
