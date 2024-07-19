use super::{Duration, Epoch, TimeSeries, Weekday};
use crate::parser::Token;

#[cfg(kani)]
mod kani_harnesses {
    use super::*;
    #[kani::proof]
    fn kani_harness_value_ok() {
        let val: i32 = kani::any();
        let callee: Token = kani::any();
        callee.value_ok(val);
    }

    #[kani::proof]
    fn kani_harness_gregorian_position() {
        let callee: Token = kani::any();
        callee.gregorian_position();
    }

    #[kani::proof]
    fn kani_harness_advance_with() {
        let ending_char: char = kani::any();
        let mut callee: Token = kani::any();
        callee.advance_with(ending_char);
    }

    #[kani::proof]
    fn kani_harness_is_numeric() {
        let callee: Token = kani::any();
        callee.is_numeric();
    }

    #[kani::proof]
    fn kani_harness_TimeSeries_exclusive() {
        let start: Epoch = kani::any();
        let end: Epoch = kani::any();
        let step: Duration = kani::any();
        TimeSeries::exclusive(start, end, step);
    }

    #[kani::proof]
    fn kani_harness_TimeSeries_inclusive() {
        let start: Epoch = kani::any();
        let end: Epoch = kani::any();
        let step: Duration = kani::any();
        TimeSeries::inclusive(start, end, step);
    }

    #[kani::proof]
    fn kani_harness_to_c89_weekday() {
        let callee: Weekday = kani::any();
        callee.to_c89_weekday();
    }
}
