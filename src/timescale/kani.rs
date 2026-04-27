/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use super::TimeScale;
use kani::Arbitrary;

impl Arbitrary for TimeScale {
    #[inline(always)]
    fn any() -> Self {
        let ts_u8: u8 = kani::any();
        Self::from(ts_u8)
    }
}

#[kani::proof]
fn formal_time_scale() {
    let _time_scale: TimeScale = kani::any();
}

#[kani::proof]
#[kani::stub_verified(crate::duration::Duration::decompose)]
fn kani_harness_gregorian_epoch_offset() {
    let callee: crate::TimeScale = kani::any();
    let _ = callee.gregorian_epoch_offset();
}

#[cfg(kani)]
mod kani_harnesses {
    use super::*;
    #[kani::proof]
    fn kani_harness_formatted_len() {
        let callee: TimeScale = kani::any();
        let _ = callee.formatted_len();
    }

    #[kani::proof]
    fn kani_harness_is_gnss() {
        let callee: TimeScale = kani::any();
        let _ = callee.is_gnss();
    }

    #[kani::proof]
    fn kani_harness_reference_epoch() {
        let callee: TimeScale = kani::any();
        let _ = callee.reference_epoch();
    }

    #[kani::proof]
    fn kani_harness_prime_epoch_offset() {
        let callee: TimeScale = kani::any();
        let _ = callee.prime_epoch_offset();
    }
}
