/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{Duration, Epoch, TimeScale, Unit, TT_OFFSET_MS};

#[kani::proof]
fn formal_epoch_reciprocity_tai() {
    let duration: Duration = kani::any();

    // TAI
    let time_scale: TimeScale = TimeScale::TAI;
    let epoch: Epoch = Epoch::from_duration(duration, time_scale);
    assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_tai_seconds(seconds);
    }

    let days: f64 = kani::any();
    if days.is_finite() {
        let _ = Epoch::from_tai_days(days);
    }
}

#[kani::proof]
fn formal_epoch_reciprocity_tt() {
    let duration: Duration = kani::any();

    // TT -- Check valid within bounds of (MIN + TT Offset) and (MAX - TT Offset)
    if duration > Duration::MIN + TT_OFFSET_MS * Unit::Millisecond
        && duration < Duration::MAX - TT_OFFSET_MS * Unit::Millisecond
    {
        let time_scale: TimeScale = TimeScale::TT;
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_tt_seconds(seconds);
    }
    // No TT Days initializer
}

// Skip ET, kani chokes on the Newton Raphson loop.

// Skip TDB
//
// #[kani::proof]
#[test]
fn formal_epoch_reciprocity_tdb() {
    // let duration: Duration = kani::any();
    let duration = Duration::from_parts(19510, 3155759999999997938);

    // TDB
    let ts_offset = TimeScale::TDB.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        // We guard TDB from durations that are would hit the MIN or the MAX.
        // TDB is centered on J2000 but the Epoch is on J1900. So on initialization, we offset by one century and twelve hours.
        // If the duration is too close to the Duration bounds, then the TDB initialization and retrieval will fail (because the bounds will have been hit).

        let time_scale: TimeScale = TimeScale::TDB;
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        let out_duration = epoch.to_duration_in_time_scale(time_scale);
        assert_eq!((out_duration - duration).to_seconds(), 0.0);
        assert_eq!(out_duration.centuries, duration.centuries);
        assert_eq!(out_duration.nanoseconds, duration.nanoseconds);
        let error = (out_duration.nanoseconds - duration.nanoseconds) as f64;
        assert!(error.abs() < 500_000.0, "error: {}", error);
    }
}

// Skip UTC, kani chokes on the leap seconds counting.

#[kani::proof]
#[test]
fn formal_epoch_reciprocity_gpst() {
    let duration: Duration = kani::any();

    // GPST
    let time_scale: TimeScale = TimeScale::GPST;
    let ts_offset = TimeScale::GPST.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_gpst_seconds(seconds);
    }

    let _ = Epoch::from_gpst_nanoseconds(kani::any());
}

#[kani::proof]
fn formal_epoch_reciprocity_gst() {
    let duration: Duration = kani::any();

    // GST
    let time_scale: TimeScale = TimeScale::GST;
    let ts_offset = TimeScale::GST.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_gst_seconds(seconds);
    }

    let days: f64 = kani::any();
    if days.is_finite() {
        let _ = Epoch::from_gst_days(days);
    }

    let _ = Epoch::from_gst_nanoseconds(kani::any());
}

#[kani::proof]
fn formal_epoch_reciprocity_bdt() {
    let duration: Duration = kani::any();

    // BDT
    let time_scale: TimeScale = TimeScale::BDT;
    let ts_offset = TimeScale::BDT.reference_epoch() - TimeScale::TAI.reference_epoch();
    if duration > Duration::MIN + ts_offset && duration < Duration::MAX - ts_offset {
        let epoch: Epoch = Epoch::from_duration(duration, time_scale);
        assert_eq!(epoch.to_duration_in_time_scale(time_scale), duration);
    }

    // Check that no error occurs on initialization
    let seconds: f64 = kani::any();
    if seconds.is_finite() {
        let _ = Epoch::from_bdt_seconds(seconds);
    }

    let days: f64 = kani::any();
    if days.is_finite() {
        let _ = Epoch::from_bdt_days(days);
    }

    let _ = Epoch::from_bdt_nanoseconds(kani::any());
}

#[kani::proof]
fn formal_epoch_julian() {
    let days: f64 = kani::any();

    if days.is_finite() {
        // The initializers will fail on subnormal days.
        let _ = Epoch::from_mjd_bdt(days);
        let _ = Epoch::from_mjd_gpst(days);
        let _ = Epoch::from_mjd_gst(days);
        let _ = Epoch::from_mjd_tai(days);
        let _ = Epoch::from_jde_bdt(days);
        let _ = Epoch::from_jde_gpst(days);
        let _ = Epoch::from_jde_gst(days);
        let _ = Epoch::from_jde_tai(days);
        let _ = Epoch::from_jde_et(days);
        let _ = Epoch::from_jde_tai(days);
    }
}
