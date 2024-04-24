/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
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
