#[cfg(feature = "std")]
extern crate core;

use hifitime::Weekday;

#[cfg(feature = "std")]
use core::f64::EPSILON;
#[cfg(not(feature = "std"))]
use std::f64::EPSILON;

#[test]
fn test_basic_ops() {
    assert_eq!(Weekday::default(), Weekday::Monday);
    let weekday = Weekday::default();
    for i in 1..24 {
        // test (+) wrapping
        let add = weekday + i;
        let expected: Weekday = i.rem_euclid(Weekday::MAX.into()).into();
        assert_eq!(add, expected);
    }
}
