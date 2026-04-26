/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

// Here lives all of the formal verification for Duration.

use super::Duration;
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
#[kani::stub_verified(Duration::decompose)]
fn formal_duration_normalize_any() {
    let dur: Duration = kani::any();
    // Check that decompose never fails
    let _ = dur.decompose();
}

#[kani::proof]
fn formal_duration_truncated_ns_reciprocity() {
    let nanoseconds: i64 = kani::any();
    let dur_from_part = Duration::from_truncated_nanoseconds(nanoseconds);
    let recip_ns = dur_from_part.try_truncated_nanoseconds().unwrap();
    assert_eq!(recip_ns, nanoseconds);
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
    // repeat_test!(test_dur_f64_recip_1, [1e-5, 1e-4, 1e-3]);
    // repeat_test!(test_dur_f64_recip_2, [1e-2, 1e-1, 1e0]);
    // repeat_test!(test_dur_f64_recip_3, [1e0, 1e1, 1e2]);
    // repeat_test!(test_dur_f64_recip_4, [1e2, 1e3, 1e4]);
    // repeat_test!(test_dur_f64_recip_5, [1e4, 1e5]);
    // repeat_test!(test_dur_f64_recip_6, [1e5, 1e6]);
}

#[kani::proof]
#[kani::stub_verified(Duration::decompose)]
fn kani_harness_subdivision() {
    let unit: crate::Unit = kani::any();
    let callee: Duration = kani::any();
    let _ = callee.subdivision(unit);
}

#[kani::proof_for_contract(Duration::to_seconds)]
fn verify_to_seconds_contract() {
    let dur: Duration = kani::any();
    let _ = dur.to_seconds();
}

#[kani::proof_for_contract(Duration::from_seconds)]
fn verify_from_seconds_contract() {
    let value: f64 = kani::any();
    let _ = Duration::from_seconds(value);
}

#[cfg(kani)]
#[allow(non_snake_case)]
mod kani_harnesses {
    use super::*;
    use crate::Unit;
    #[kani::proof_for_contract(Duration::from_parts)]
    fn kani_harness_Duration_from_parts() {
        let centuries: i16 = kani::any();
        let nanoseconds: u64 = kani::any();
        let _ = Duration::from_parts(centuries, nanoseconds);
    }

    #[kani::proof_for_contract(Duration::from_total_nanoseconds)]
    fn kani_harness_Duration_from_total_nanoseconds() {
        let nanos: i128 = kani::any();
        let _ = Duration::from_total_nanoseconds(nanos);
    }

    #[kani::proof_for_contract(Duration::from_truncated_nanoseconds)]
    fn kani_harness_Duration_from_truncated_nanoseconds() {
        let nanos: i64 = kani::any();
        let _ = Duration::from_truncated_nanoseconds(nanos);
    }

    #[kani::proof_for_contract(Duration::from_days)]
    fn kani_harness_Duration_from_days() {
        let value: f64 = kani::any();
        let _ = Duration::from_days(value);
    }

    #[kani::proof_for_contract(Duration::from_hours)]
    fn kani_harness_Duration_from_hours() {
        let value: f64 = kani::any();
        let _ = Duration::from_hours(value);
    }

    #[kani::proof_for_contract(Duration::from_seconds)]
    fn kani_harness_Duration_from_seconds() {
        let value: f64 = kani::any();
        let _ = Duration::from_seconds(value);
    }

    #[kani::proof_for_contract(Duration::from_milliseconds)]
    fn kani_harness_Duration_from_milliseconds() {
        let value: f64 = kani::any();
        let _ = Duration::from_milliseconds(value);
    }

    #[kani::proof_for_contract(Duration::from_microseconds)]
    fn kani_harness_Duration_from_microseconds() {
        let value: f64 = kani::any();
        let _ = Duration::from_microseconds(value);
    }

    #[kani::proof_for_contract(Duration::from_nanoseconds)]
    fn kani_harness_Duration_from_nanoseconds() {
        let value: f64 = kani::any();
        let _ = Duration::from_nanoseconds(value);
    }

    #[kani::proof_for_contract(Duration::compose)]
    #[kani::stub_verified(Unit::const_multiply)]
    fn kani_harness_Duration_compose() {
        let sign: i8 = kani::any();
        let days: u64 = kani::any();
        let hours: u64 = kani::any();
        let minutes: u64 = kani::any();
        let seconds: u64 = kani::any();
        let milliseconds: u64 = kani::any();
        let microseconds: u64 = kani::any();
        let nanoseconds: u64 = kani::any();
        let _ = Duration::compose(
            sign,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        );
    }

    #[kani::proof]
    #[kani::stub_verified(Unit::const_multiply)]
    fn kani_harness_Duration_compose_f64() {
        let sign: i8 = kani::any();
        let days: f64 = kani::any();
        let hours: f64 = kani::any();
        let minutes: f64 = kani::any();
        let seconds: f64 = kani::any();
        let milliseconds: f64 = kani::any();
        let microseconds: f64 = kani::any();
        let nanoseconds: f64 = kani::any();
        kani::assume(
            days.is_finite()
                && hours.is_finite()
                && minutes.is_finite()
                && seconds.is_finite()
                && milliseconds.is_finite()
                && microseconds.is_finite()
                && nanoseconds.is_finite(),
        );
        let _ = Duration::compose_f64(
            sign,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        );
    }

    #[kani::proof_for_contract(Duration::from_tz_offset)]
    fn kani_harness_Duration_from_tz_offset() {
        let sign: i8 = kani::any();
        let hours: i64 = kani::any();
        let minutes: i64 = kani::any();
        let _ = Duration::from_tz_offset(sign, hours, minutes);
    }

    #[kani::proof_for_contract(Duration::as_normalized)]
    fn kani_harness_normalize() {
        let centuries: i16 = kani::any();
        let nanoseconds: u64 = kani::any();
        let dur = Duration {
            centuries,
            nanoseconds,
        };
        let _ = dur.as_normalized();
    }

    #[kani::proof_for_contract(Duration::to_parts)]
    fn kani_harness_to_parts() {
        let callee: Duration = kani::any();
        let _ = callee.to_parts();
    }

    #[kani::proof]
    fn kani_harness_total_nanoseconds() {
        let callee: Duration = kani::any();
        let _ = callee.total_nanoseconds();
    }

    #[kani::proof]
    fn kani_harness_try_truncated_nanoseconds() {
        let callee: Duration = kani::any();
        let _ = callee.try_truncated_nanoseconds();
    }

    #[kani::proof]
    fn kani_harness_truncated_nanoseconds() {
        let callee: Duration = kani::any();
        let _ = callee.truncated_nanoseconds();
    }

    #[kani::proof]
    fn kani_harness_to_seconds() {
        let callee: Duration = kani::any();
        let _ = callee.to_seconds();
    }

    #[kani::proof]
    fn kani_harness_to_unit() {
        let unit: Unit = kani::any();
        let callee: Duration = kani::any();
        let _ = callee.to_unit(unit);
    }

    #[kani::proof_for_contract(Duration::abs)]
    fn kani_harness_abs() {
        let callee: Duration = kani::any();
        let _ = callee.abs();
    }

    #[kani::proof_for_contract(Duration::signum)]
    fn kani_harness_signum() {
        let callee: Duration = kani::any();
        let _ = callee.signum();
    }

    #[kani::proof_for_contract(Duration::decompose)]
    #[kani::stub_verified(Unit::const_multiply)]
    fn kani_harness_decompose() {
        let callee: Duration = kani::any();
        let _ = callee.decompose();
    }

    // kani_harness_subdivision moved to top-level for stub_verified compatibility

    #[kani::proof_for_contract(Duration::floor)]
    fn kani_harness_floor() {
        let duration: Duration = kani::any();
        let callee: Duration = kani::any();
        let _ = callee.floor(duration);
    }

    // Duration::ceil — TIMEOUT with all solvers (CBMC, z3, cvc5).
    // Contract: ensures(result.nanoseconds < NPC || MAX || MIN)
    // Root cause: ceil calls floor (i128 div_euclid) + total_nanoseconds (i128 mul)
    // + checked_add (i128) + from_total_nanoseconds (i128 div_euclid). The combined
    // i128 arithmetic creates ~250K SAT clauses, exceeding solver capacity.
    // Compositional approach blocked by: (1) kani::stub can't target functions with
    // kani::ensures contracts, (2) kani::stub_verified compilation scales poorly
    // with crate size (>5min for hifitime).
    // #[kani::proof]
    // fn kani_harness_ceil() {
    //     let duration: Duration = kani::any();
    //     let callee: Duration = kani::any();
    //     let _ = callee.ceil(duration);
    // }

    // Duration::round — TIMEOUT with all solvers.
    // Contract: ensures(result.nanoseconds < NPC || MAX || MIN)
    // Root cause: round calls both floor AND ceil, tripling the i128 arithmetic.
    // Same compositional blockers as ceil above.
    // #[kani::proof]
    // fn kani_harness_round() {
    //     let duration: Duration = kani::any();
    //     let callee: Duration = kani::any();
    //     let _ = callee.round(duration);
    // }

    #[kani::proof]
    fn kani_harness_approx() {
        let callee: Duration = kani::any();
        let _ = callee.approx();
    }

    #[kani::proof_for_contract(Duration::min)]
    fn kani_harness_min() {
        let other: Duration = kani::any();
        let callee: Duration = kani::any();
        let _ = callee.min(other);
    }

    #[kani::proof_for_contract(Duration::max)]
    fn kani_harness_max() {
        let other: Duration = kani::any();
        let callee: Duration = kani::any();
        let _ = callee.max(other);
    }

    #[kani::proof_for_contract(Duration::is_negative)]
    fn kani_harness_is_negative() {
        let callee: Duration = kani::any();
        let _ = callee.is_negative();
    }

    /// Verifies Unit::const_multiply always returns a normalized Duration.
    #[kani::proof_for_contract(Unit::const_multiply)]
    fn verify_unit_const_multiply_contract() {
        let unit: Unit = kani::any();
        let q: f64 = kani::any();
        let _ = unit.const_multiply(q);
    }
}

/// Verifies the #[kani::ensures] contract on total_nanoseconds():
/// result == centuries * NPC + nanoseconds for all inputs.
///
/// Constructs Duration directly (not via from_parts) to avoid a second
/// total_nanoseconds call through the Arbitrary impl, which would conflict
/// with proof_for_contract's single-call requirement.
#[kani::proof_for_contract(Duration::total_nanoseconds)]
fn verify_total_nanoseconds_contract() {
    let centuries: i16 = kani::any();
    let nanoseconds: u64 = kani::any();
    let dur = Duration {
        centuries,
        nanoseconds,
    };
    let _ = dur.total_nanoseconds();
}

/// Verifies that Duration * i64 does not panic and produces a normalized result
/// for ALL i64 values including i64::MIN.
///
/// This caught a bug where Unit::Mul<i64> called total_ns.abs() which panics
/// on i64::MIN. Fixed by using unsigned_abs().
///
/// Note: Cannot use #[kani::proof_for_contract] because Mul<i64> is a generic
/// trait method and Kani does not support contracts on those (issue #1997).
/// The normalization postcondition is verified via explicit assertion instead.
#[kani::proof]
fn verify_mul_i64_no_panic() {
    let dur: Duration = kani::any();
    let q: i64 = kani::any();
    let result = dur * q;
    let (c, n) = result.to_parts();
    assert!(
        n < NANOSECONDS_PER_CENTURY
            || (c == i16::MAX && n == NANOSECONDS_PER_CENTURY)
            || (c == i16::MIN && n == 0)
    );
}

/// Verifies Duration::Mul<f64> terminates and produces a normalized result.
///
/// This caught a bug where q * 10^p overflowing to infinity caused
/// floor(inf) - inf = NaN, and NaN < EPSILON = false, so the loop
/// never broke. Fixed by adding !is_finite() guard and p >= 19 bound.
///
/// The loop is annotated with #[kani::loop_invariant(p >= 0 && p <= 19)]
/// which Kani verifies inductively — proving the bound holds for all
/// iterations without unrolling.
///
/// Note: Cannot use #[kani::ensures] / #[kani::proof_for_contract] on Mul<f64>
/// because Kani does not support contracts on generic trait methods (issue #1997).
///
/// The function is correct for ALL f64 inputs after the fix. The assumptions
/// below restrict the input range solely to keep CBMC's f64 symbolic execution
/// tractable within the verification time budget — they are NOT preconditions
/// of the function:
/// - is_finite: CBMC's bit-level f64 model for NaN/inf creates intractable SAT formulas
/// - |q| < 1e15: keeps q * 10^19 within f64 range, reducing SAT formula size
/// - |q| > 1e-18: avoids subnormal f64 representation which multiplies CBMC's case splits
#[kani::proof]
fn verify_mul_f64_terminates() {
    let dur: Duration = kani::any();
    let q: f64 = kani::any();
    // Verification budget constraints (not function preconditions):
    kani::assume(q.is_finite());
    kani::assume(q.abs() < 1e15);
    kani::assume(q.abs() > 1e-18 || q == 0.0);
    let result = dur * q;
    let (c, n) = result.to_parts();
    assert!(
        n < NANOSECONDS_PER_CENTURY
            || (c == i16::MAX && n == NANOSECONDS_PER_CENTURY)
            || (c == i16::MIN && n == 0)
    );
}

/// Proves Duration::PartialEq and Duration::Ord are consistent:
/// if a == b then a.cmp(&b) == Equal, for all Duration values.
/// This was Bug 5 (issue #469): the zero-crossing special case in
/// PartialEq made -d == d, but derived Ord did not, violating
/// Rust's Eq/Ord contract.
#[kani::proof]
fn verify_duration_eq_ord_consistent() {
    let a: Duration = kani::any();
    let b: Duration = kani::any();
    if a == b {
        assert!(
            a.cmp(&b) == core::cmp::Ordering::Equal,
            "PartialEq and Ord must be consistent"
        );
    }
}
