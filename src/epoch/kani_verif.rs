/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2017-onwards Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::epoch::system_time::duration_since_unix_epoch;
use crate::leap_seconds::LeapSecond;
use crate::{Duration, Epoch, TimeScale, Unit, Weekday, TT_OFFSET_MS};

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

#[cfg(kani)]
mod kani_harnesses {
    use super::*;
    use crate::*;
    #[kani::proof]
    fn kani_harness_Epoch_compute_gregorian() {
        let duration: Duration = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::compute_gregorian(duration, time_scale);
    }

    #[kani::proof]
    fn kani_harness_to_gregorian_str() {
        let time_scale: TimeScale = kani::any();
        let callee: Epoch = kani::any();
        callee.to_gregorian_str(time_scale);
    }

    #[kani::proof]
    fn kani_harness_to_gregorian_utc() {
        let callee: Epoch = kani::any();
        callee.to_gregorian_utc();
    }

    #[kani::proof]
    fn kani_harness_to_gregorian_tai() {
        let callee: Epoch = kani::any();
        callee.to_gregorian_tai();
    }

    #[kani::proof]
    fn kani_harness_Epoch_maybe_from_gregorian_tai() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        Epoch::maybe_from_gregorian_tai(year, month, day, hour, minute, second, nanos);
    }

    #[kani::proof]
    fn kani_harness_Epoch_maybe_from_gregorian() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::maybe_from_gregorian(year, month, day, hour, minute, second, nanos, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_tai() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        Epoch::from_gregorian_tai(year, month, day, hour, minute, second, nanos);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_tai_at_midnight() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        Epoch::from_gregorian_tai_at_midnight(year, month, day);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_tai_at_noon() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        Epoch::from_gregorian_tai_at_noon(year, month, day);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_tai_hms() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        Epoch::from_gregorian_tai_hms(year, month, day, hour, minute, second);
    }

    #[kani::proof]
    fn kani_harness_Epoch_maybe_from_gregorian_utc() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        Epoch::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanos);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_utc() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        Epoch::from_gregorian_utc(year, month, day, hour, minute, second, nanos);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_utc_at_midnight() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        Epoch::from_gregorian_utc_at_midnight(year, month, day);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_utc_at_noon() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        Epoch::from_gregorian_utc_at_noon(year, month, day);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_utc_hms() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        Epoch::from_gregorian_utc_hms(year, month, day, hour, minute, second);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_gregorian(year, month, day, hour, minute, second, nanos, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_at_midnight() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_gregorian_at_midnight(year, month, day, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_at_noon() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_gregorian_at_noon(year, month, day, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gregorian_hms() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_gregorian_hms(year, month, day, hour, minute, second, time_scale);
    }

    #[kani::proof]
    fn kani_harness_is_gregorian_valid() {
        let year: i32 = kani::any();
        let month: u8 = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let nanos: u32 = kani::any();
        is_gregorian_valid(year, month, day, hour, minute, second, nanos);
    }

    #[kani::proof]
    fn kani_harness_LeapSecond_new() {
        let timestamp_tai_s: f64 = kani::any();
        let delta_at: f64 = kani::any();
        let announced: bool = kani::any();
        LeapSecond::new(timestamp_tai_s, delta_at, announced);
    }

    #[kani::proof]
    fn kani_harness_to_time_scale() {
        let ts: TimeScale = kani::any();
        let callee: Epoch = kani::any();
        callee.to_time_scale(ts);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tai_duration() {
        let duration: Duration = kani::any();
        Epoch::from_tai_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_duration() {
        let duration: Duration = kani::any();
        let ts: TimeScale = kani::any();
        Epoch::from_duration(duration, ts);
    }

    #[kani::proof]
    fn kani_harness_to_duration_since_j1900() {
        let callee: Epoch = kani::any();
        callee.to_duration_since_j1900();
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tai_parts() {
        let centuries: i16 = kani::any();
        let nanoseconds: u64 = kani::any();
        Epoch::from_tai_parts(centuries, nanoseconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tai_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_tai_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tai_days() {
        let days: f64 = kani::any();
        Epoch::from_tai_days(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_utc_duration() {
        let duration: Duration = kani::any();
        Epoch::from_utc_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_utc_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_utc_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_utc_days() {
        let days: f64 = kani::any();
        Epoch::from_utc_days(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gpst_duration() {
        let duration: Duration = kani::any();
        Epoch::from_gpst_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_qzsst_duration() {
        let duration: Duration = kani::any();
        Epoch::from_qzsst_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gst_duration() {
        let duration: Duration = kani::any();
        Epoch::from_gst_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_bdt_duration() {
        let duration: Duration = kani::any();
        Epoch::from_bdt_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_tai() {
        let days: f64 = kani::any();
        Epoch::from_mjd_tai(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_in_time_scale() {
        let days: f64 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_mjd_in_time_scale(days, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_utc() {
        let days: f64 = kani::any();
        Epoch::from_mjd_utc(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_gpst() {
        let days: f64 = kani::any();
        Epoch::from_mjd_gpst(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_qzsst() {
        let days: f64 = kani::any();
        Epoch::from_mjd_qzsst(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_gst() {
        let days: f64 = kani::any();
        Epoch::from_mjd_gst(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_mjd_bdt() {
        let days: f64 = kani::any();
        Epoch::from_mjd_bdt(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_tai() {
        let days: f64 = kani::any();
        Epoch::from_jde_tai(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_in_time_scale() {
        let days: f64 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_jde_in_time_scale(days, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_utc() {
        let days: f64 = kani::any();
        Epoch::from_jde_utc(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_gpst() {
        let days: f64 = kani::any();
        Epoch::from_jde_gpst(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_qzsst() {
        let days: f64 = kani::any();
        Epoch::from_jde_qzsst(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_gst() {
        let days: f64 = kani::any();
        Epoch::from_jde_gst(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_bdt() {
        let days: f64 = kani::any();
        Epoch::from_jde_bdt(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tt_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_tt_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tt_duration() {
        let duration: Duration = kani::any();
        Epoch::from_tt_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_et_seconds() {
        let seconds_since_j2000: f64 = kani::any();
        Epoch::from_et_seconds(seconds_since_j2000);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_et_duration() {
        let duration_since_j2000: Duration = kani::any();
        Epoch::from_et_duration(duration_since_j2000);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tdb_seconds() {
        let seconds_j2000: f64 = kani::any();
        Epoch::from_tdb_seconds(seconds_j2000);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_tdb_duration() {
        let duration_since_j2000: Duration = kani::any();
        Epoch::from_tdb_duration(duration_since_j2000);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_et() {
        let days: f64 = kani::any();
        Epoch::from_jde_et(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_jde_tdb() {
        let days: f64 = kani::any();
        Epoch::from_jde_tdb(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gpst_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_gpst_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gpst_days() {
        let days: f64 = kani::any();
        Epoch::from_gpst_days(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gpst_nanoseconds() {
        let nanoseconds: u64 = kani::any();
        Epoch::from_gpst_nanoseconds(nanoseconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_qzsst_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_qzsst_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_qzsst_days() {
        let days: f64 = kani::any();
        Epoch::from_qzsst_days(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_qzsst_nanoseconds() {
        let nanoseconds: u64 = kani::any();
        Epoch::from_qzsst_nanoseconds(nanoseconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gst_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_gst_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gst_days() {
        let days: f64 = kani::any();
        Epoch::from_gst_days(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_gst_nanoseconds() {
        let nanoseconds: u64 = kani::any();
        Epoch::from_gst_nanoseconds(nanoseconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_bdt_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_bdt_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_bdt_days() {
        let days: f64 = kani::any();
        Epoch::from_bdt_days(days);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_bdt_nanoseconds() {
        let nanoseconds: u64 = kani::any();
        Epoch::from_bdt_nanoseconds(nanoseconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_unix_duration() {
        let duration: Duration = kani::any();
        Epoch::from_unix_duration(duration);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_unix_seconds() {
        let seconds: f64 = kani::any();
        Epoch::from_unix_seconds(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_unix_milliseconds() {
        let millisecond: f64 = kani::any();
        Epoch::from_unix_milliseconds(millisecond);
    }

    #[kani::proof]
    fn kani_harness_Epoch_delta_et_tai() {
        let seconds: f64 = kani::any();
        Epoch::delta_et_tai(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_inner_g() {
        let seconds: f64 = kani::any();
        Epoch::inner_g(seconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_time_of_week() {
        let week: u32 = kani::any();
        let nanoseconds: u64 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_time_of_week(week, nanoseconds, time_scale);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_time_of_week_utc() {
        let week: u32 = kani::any();
        let nanoseconds: u64 = kani::any();
        Epoch::from_time_of_week_utc(week, nanoseconds);
    }

    #[kani::proof]
    fn kani_harness_Epoch_from_day_of_year() {
        let year: i32 = kani::any();
        let days: f64 = kani::any();
        let time_scale: TimeScale = kani::any();
        Epoch::from_day_of_year(year, days, time_scale);
    }

    #[kani::proof]
    fn kani_harness_min() {
        let other: Epoch = kani::any();
        let callee: Epoch = kani::any();
        callee.min(other);
    }

    #[kani::proof]
    fn kani_harness_max() {
        let other: Epoch = kani::any();
        let callee: Epoch = kani::any();
        callee.max(other);
    }

    #[kani::proof]
    fn kani_harness_floor() {
        let duration: Duration = kani::any();
        let callee: Epoch = kani::any();
        callee.floor(duration);
    }

    #[kani::proof]
    fn kani_harness_ceil() {
        let duration: Duration = kani::any();
        let callee: Epoch = kani::any();
        callee.ceil(duration);
    }

    #[kani::proof]
    fn kani_harness_round() {
        let duration: Duration = kani::any();
        let callee: Epoch = kani::any();
        callee.round(duration);
    }

    #[kani::proof]
    fn kani_harness_to_time_of_week() {
        let callee: Epoch = kani::any();
        callee.to_time_of_week();
    }

    #[kani::proof]
    fn kani_harness_weekday_in_time_scale() {
        let time_scale: TimeScale = kani::any();
        let callee: Epoch = kani::any();
        callee.weekday_in_time_scale(time_scale);
    }

    #[kani::proof]
    fn kani_harness_weekday() {
        let callee: Epoch = kani::any();
        callee.weekday();
    }

    #[kani::proof]
    fn kani_harness_weekday_utc() {
        let callee: Epoch = kani::any();
        callee.weekday_utc();
    }

    #[kani::proof]
    fn kani_harness_next() {
        let weekday: Weekday = kani::any();
        let callee: Epoch = kani::any();
        callee.next(weekday);
    }

    #[kani::proof]
    fn kani_harness_next_weekday_at_midnight() {
        let weekday: Weekday = kani::any();
        let callee: Epoch = kani::any();
        callee.next_weekday_at_midnight(weekday);
    }

    #[kani::proof]
    fn kani_harness_next_weekday_at_noon() {
        let weekday: Weekday = kani::any();
        let callee: Epoch = kani::any();
        callee.next_weekday_at_noon(weekday);
    }

    #[kani::proof]
    fn kani_harness_previous() {
        let weekday: Weekday = kani::any();
        let callee: Epoch = kani::any();
        callee.previous(weekday);
    }

    #[kani::proof]
    fn kani_harness_previous_weekday_at_midnight() {
        let weekday: Weekday = kani::any();
        let callee: Epoch = kani::any();
        callee.previous_weekday_at_midnight(weekday);
    }

    #[kani::proof]
    fn kani_harness_previous_weekday_at_noon() {
        let weekday: Weekday = kani::any();
        let callee: Epoch = kani::any();
        callee.previous_weekday_at_noon(weekday);
    }

    #[kani::proof]
    fn kani_harness_duration_since_unix_epoch() {
        duration_since_unix_epoch();
    }

    #[kani::proof]
    fn kani_harness_Epoch_now() {
        Epoch::now();
    }

    #[kani::proof]
    fn kani_harness_with_hms() {
        let hours: u64 = kani::any();
        let minutes: u64 = kani::any();
        let seconds: u64 = kani::any();
        let callee: Epoch = kani::any();
        callee.with_hms(hours, minutes, seconds);
    }

    #[kani::proof]
    fn kani_harness_with_hms_from() {
        let other: Epoch = kani::any();
        let callee: Epoch = kani::any();
        callee.with_hms_from(other);
    }

    #[kani::proof]
    fn kani_harness_with_time_from() {
        let other: Epoch = kani::any();
        let callee: Epoch = kani::any();
        callee.with_time_from(other);
    }

    #[kani::proof]
    fn kani_harness_with_hms_strict() {
        let hours: u64 = kani::any();
        let minutes: u64 = kani::any();
        let seconds: u64 = kani::any();
        let callee: Epoch = kani::any();
        callee.with_hms_strict(hours, minutes, seconds);
    }

    #[kani::proof]
    fn kani_harness_with_hms_strict_from() {
        let other: Epoch = kani::any();
        let callee: Epoch = kani::any();
        callee.with_hms_strict_from(other);
    }
}
