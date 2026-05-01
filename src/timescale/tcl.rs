/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use crate::{Duration, TimeUnits};

/// Mean fractional rate of TCL relative to TT.
///
/// Paper option iii removes this secular TCL-TT drift when defining TL.
/// The paper gives approximately 6.8e-10, i.e. about 58.7 us/day.
pub(crate) const TCL_MINUS_TT_MEAN_RATE: f64 = 6.8e-10;

/// TL option iii:
///
/// ```text
///     TL = TCL + Δf * (TCL - TCL0) + const0
/// ```
/// If:
/// ```text
///     dTCL / dTT = 1 + k
/// ```
/// then choosing:
/// ```text
///     1 + Δf = 1 / (1 + k)
///
/// gives:
/// ```text
///     dTL / dTT = 1
/// ```
/// so TL has no secular drift relative to TT.
pub(crate) const TL_DELTA_F: f64 = -TCL_MINUS_TT_MEAN_RATE / (1.0 + TCL_MINUS_TT_MEAN_RATE);

/// Recommended experimental convention:
/// ```text
///     TL = TCL = TT at T0
/// ```
/// with:
/// ```text
///     T0 = 1977-01-01T00:00:00 TAI
///        = 1977-01-01T00:00:32.184 TT
/// ```
pub(crate) const TL_CONST0_S: f64 = 0.0;

#[inline]
pub fn tt_since_t77_to_tcl_since_t77(tt_since_t77: Duration) -> Duration {
    // Mean TCL model:
    //
    //     TCL - TCL0 = (TT - TT0) * (1 + k)
    //
    // Do not factor this into tt_since_t77 * (1.0 + k), matching the
    // style used for TCG/TCB to reduce avoidable rounding noise.
    tt_since_t77 + tt_since_t77 * TCL_MINUS_TT_MEAN_RATE
}

#[inline]
pub fn tcl_since_t77_to_tt_since_t77(tcl_since_t77: Duration) -> Duration {
    // Inverse of:
    //
    //     TCL = TT * (1 + k)
    //
    // so:
    //
    //     TT = TCL / (1 + k)
    //        = TCL * (1 - k / (1 + k))
    tcl_since_t77 - tcl_since_t77 * (TCL_MINUS_TT_MEAN_RATE / (1.0 + TCL_MINUS_TT_MEAN_RATE))
}

#[inline]
pub fn tcl_since_t77_to_tl_since_t77(tcl_since_t77: Duration) -> Duration {
    // Option iii:
    //
    //     TL = TCL + Δf * (TCL - TCL0) + const0
    //
    // with const0 = 0 and TCL0 = T0.
    tcl_since_t77 + tcl_since_t77 * TL_DELTA_F + TL_CONST0_S.seconds()
}

#[inline]
pub fn tl_since_t77_to_tcl_since_t77(tl_since_t77: Duration) -> Duration {
    // Since TL = TCL / (1 + k), the inverse is:
    //
    //     TCL = TL * (1 + k)
    //
    // const0 is zero in this convention.
    let tl_minus_const0 = tl_since_t77 - TL_CONST0_S.seconds();
    tl_minus_const0 + tl_minus_const0 * TCL_MINUS_TT_MEAN_RATE
}

#[cfg(test)]
mod ut_tcl {
    use super::*;
    use crate::{Epoch, TimeScale, TimeUnits};

    /// This test uses the private prime_epoch_offset, hence its definition here.
    #[test]
    fn tcl_accumulates_mean_drift_from_tt() {
        let tt = Epoch::from_gregorian_at_midnight(2024, 2, 29, TimeScale::TT);
        let tcl = tt.to_time_scale(TimeScale::TCL);

        let tt_since_t77 = tt.duration - TimeScale::TCL.prime_epoch_offset();
        let expected = tt_since_t77 * TCL_MINUS_TT_MEAN_RATE;

        let actual = tcl.duration - tt_since_t77;

        assert!(
            (actual - expected).abs() <= 1.nanoseconds(),
            "actual={actual}, expected={expected}"
        );
    }
}
