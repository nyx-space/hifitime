use std::fmt;

use super::instant::Instant;

/// Offset is an alias of Instant. It contains the same kind of information, but is used in a
/// very different context
pub type Offset = Instant;

#[derive(Debug)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a Timezone `new` receives 60 seconds and there are only 59 seconds in the provided date
    /// time then a Carry is returned as the Result.
    Carry,
}


impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Errors::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
        }
    }
}

/// quorem returns a tuple of the quotient and the remainder a numerator and a denominator.
pub fn quorem(numerator: f64, denominator: f64) -> (i32, f64) {
    if numerator < 0.0 || denominator < 0.0 {
        panic!("quorem only supports positive numbers");
    }
    if denominator == 0.0 {
        panic!("cannot divide by zero");
    }
    (
        (numerator / denominator).floor() as i32,
        (numerator % denominator),
    )
}

#[test]
fn quorem_nominal_test() {
    assert_eq!(::utils::quorem(24.0, 6.0), (4, 0.0));
    assert_eq!(::utils::quorem(25.0, 6.0), (4, 1.0));
    assert_eq!(::utils::quorem(6.0, 6.0), (1, 0.0));
    assert_eq!(::utils::quorem(5.0, 6.0), (0, 5.0));
    assert_eq!(::utils::quorem(3540.0, 3600.0), (0, 3540.0));
    assert_eq!(::utils::quorem(3540.0, 60.0), (59, 0.0));
}

#[test]
#[should_panic]
fn quorem_negative_num_test() {
    assert_eq!(::utils::quorem(-24.0, 6.0), (4, 0.0));
}

#[test]
#[should_panic]
fn quorem_negative_den_test() {
    assert_eq!(::utils::quorem(24.0, -6.0), (4, 0.0));
}

#[test]
#[should_panic]
fn quorem_negative_numden_test() {
    // A valid argument could be made that this test should work, but there is no situation in
    // this library where two negative numbers should be considered a valid input.
    assert_eq!(::utils::quorem(-24.0, -6.0), (4, 0.0));
}
