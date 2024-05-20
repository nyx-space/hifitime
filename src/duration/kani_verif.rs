/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the formal verification for Duration.

use super::{Duration, DurationError, EpochError};
use crate::NANOSECONDS_PER_CENTURY;

use kani::Arbitrary;

impl Arbitrary for Duration {
    #[inline(always)]
    fn any() -> Self {
        let centuries: i16 = kani::any();
        let nanoseconds: u64 = kani::any();

        Duration::from_parts(centuries, nanoseconds)
    }
}

#[kani::proof]
fn formal_duration_normalize_any() {
    let dur: Duration = kani::any();
    // Check that decompose never fails
    let _ = dur.decompose();
}

#[kani::proof]
fn formal_duration_truncated_ns_reciprocity() {
    let nanoseconds: i64 = kani::any();
    let dur_from_part = Duration::from_truncated_nanoseconds(nanoseconds);

    let u_ns = dur_from_part.nanoseconds;
    let centuries = dur_from_part.centuries;
    if centuries <= -3 || centuries >= 3 {
        // Then it does not fit on a i64, so this function should return an error
        assert_eq!(
            dur_from_part.try_truncated_nanoseconds(),
            Err(EpochError::Duration {
                source: DurationError::Overflow,
            })
        );
    } else if centuries == -1 {
        // If we are negative by just enough that the centuries is negative, then the truncated seconds
        // should be the unsigned nanoseconds wrapped by the number of nanoseconds per century.

        let expect_rslt = -((NANOSECONDS_PER_CENTURY - u_ns) as i64);

        let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();
        assert_eq!(recip_ns, expect_rslt);
    } else if centuries < 0 {
        // We fit on a i64 but we need to account for the number of nanoseconds wrapped to the negative centuries.

        let nanos = u_ns.rem_euclid(NANOSECONDS_PER_CENTURY);
        let expect_rslt = i64::from(centuries) * NANOSECONDS_PER_CENTURY as i64 + nanos as i64;

        let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();
        assert_eq!(recip_ns, expect_rslt);
    } else {
        // Positive duration but enough to fit on an i64.
        let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();

        assert_eq!(recip_ns, nanoseconds);
    }
}

mod tests {
    use super::*;

    macro_rules! repeat_test {
        ($test_name:ident, $bounds:expr) => {
            #[kani::proof]
            fn $test_name() {
                for pair in $bounds.windows(2) {
                    let seconds: f64 = kani::any();

                    kani::assume(seconds > pair[0]);
                    kani::assume(seconds < pair[1]);

                    if seconds.is_finite() {
                        let big_seconds = seconds * 1e9;
                        let floored = big_seconds.floor();
                        // Remove the sub nanoseconds -- but this can lead to rounding errors!
                        let truncated_ns = floored * 1e-9;

                        let duration: Duration = Duration::from_seconds(truncated_ns);
                        let truncated_out = duration.to_seconds();
                        let floored_out = truncated_out * 1e9;

                        // So we check that the data times 1e9 matches the rounded data
                        if floored != floored_out {
                            let floored_out_bits = floored_out.to_bits();
                            let floored_bits = floored.to_bits();

                            // Allow for ONE bit error on the LSB
                            if floored_out_bits > floored_bits {
                                assert_eq!(floored_out_bits - floored_bits, 1);
                            } else {
                                assert_eq!(floored_bits - floored_out_bits, 1);
                            }
                        } else {
                            assert_eq!(floored_out, floored);
                        }
                    }
                }
            }
        };
    }

    repeat_test!(test_dur_f64_recip_0, [1e-9, 1e-8, 1e-7, 1e-6, 1e-5]);
    repeat_test!(test_dur_f64_recip_1, [1e-5, 1e-4, 1e-3]);
    // repeat_test!(test_dur_f64_recip_2, [1e-2, 1e-1, 1e0]);
    // repeat_test!(test_dur_f64_recip_3, [1e0, 1e1, 1e2]);
    // repeat_test!(test_dur_f64_recip_4, [1e2, 1e3, 1e4]);
    // repeat_test!(test_dur_f64_recip_5, [1e4, 1e5]);
    // repeat_test!(test_dur_f64_recip_6, [1e5, 1e6]);
}
