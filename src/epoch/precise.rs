/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

//! Conversions between [Epoch] and [PreciseEpoch], and accessors that only make sense
//! for any [GenericEpoch].

use super::{Epoch, GenericEpoch, PreciseEpoch};
use crate::{DurationTrait, PreciseDuration, TimeScale};

impl<D: DurationTrait> GenericEpoch<D> {
    /// Builds this epoch from a nanosecond [Epoch], in the same time scale.
    /// This conversion is exact (no time scale conversion is performed).
    #[must_use]
    pub fn from_epoch(epoch: Epoch) -> Self {
        Self {
            duration: D::from(epoch.duration),
            time_scale: epoch.time_scale,
        }
    }

    /// Converts this epoch into a nanosecond [Epoch], in the same time scale,
    /// flooring the sub-nanosecond part (cf. [GenericEpoch::subnanoseconds]).
    #[must_use]
    pub fn to_epoch(&self) -> Epoch {
        Epoch {
            duration: self.duration.to_duration(),
            time_scale: self.time_scale,
        }
    }

    /// Returns the number of zeptoseconds (1e-21 s) within the current nanosecond, in the time scale of this epoch.
    /// This is always zero for an [Epoch], and in `[0; 1e12)` for a [PreciseEpoch].
    #[must_use]
    pub fn subnanoseconds(&self) -> u64 {
        self.duration.subnanoseconds()
    }

    /// Converts this epoch into an epoch with another internal representation, in the same time scale.
    /// Converting to a lower precision floors to that precision.
    #[must_use]
    pub fn to_precision<D2: DurationTrait>(&self) -> GenericEpoch<D2> {
        let mut out = GenericEpoch::<D2>::from_epoch(self.to_epoch());
        out.duration = out.duration.add_subnanoseconds(self.subnanoseconds());
        out
    }
}

impl PreciseEpoch {
    /// Converts this epoch into a nanosecond [Epoch], in the same time scale,
    /// rounding to the closest nanosecond (ties away from zero).
    #[must_use]
    pub fn to_epoch_rounded(&self) -> Epoch {
        Epoch {
            duration: self.duration.to_duration_rounded(),
            time_scale: self.time_scale,
        }
    }

    /// Initializes a precise epoch from its total number of zeptoseconds since the reference epoch of the provided time scale.
    #[must_use]
    pub const fn from_total_zeptoseconds(zeptoseconds: i128, time_scale: TimeScale) -> Self {
        Self {
            duration: PreciseDuration::from_total_zeptoseconds(zeptoseconds),
            time_scale,
        }
    }

    /// Returns the total number of zeptoseconds of this epoch since the reference epoch of the provided time scale.
    #[must_use]
    pub fn to_zeptoseconds_in_time_scale(&self, time_scale: TimeScale) -> i128 {
        self.to_duration_in_time_scale(time_scale)
            .total_zeptoseconds()
    }
}

impl From<Epoch> for PreciseEpoch {
    /// Exact conversion, in the same time scale.
    fn from(epoch: Epoch) -> Self {
        Self::from_epoch(epoch)
    }
}

impl From<PreciseEpoch> for Epoch {
    /// Floors the precise epoch to the nanosecond, in the same time scale.
    fn from(epoch: PreciseEpoch) -> Self {
        epoch.to_epoch()
    }
}
