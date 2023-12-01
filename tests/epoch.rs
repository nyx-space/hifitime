#[cfg(feature = "std")]
extern crate core;

use hifitime::{
    is_gregorian_valid, Duration, Epoch, Errors, ParsingErrors, TimeScale, TimeUnits, Unit,
    Weekday, BDT_REF_EPOCH, DAYS_GPS_TAI_OFFSET, GPST_REF_EPOCH, GST_REF_EPOCH, J1900_OFFSET,
    J1900_REF_EPOCH, J2000_OFFSET, MJD_OFFSET, SECONDS_BDT_TAI_OFFSET, SECONDS_GPS_TAI_OFFSET,
    SECONDS_GST_TAI_OFFSET, SECONDS_PER_DAY,
};

use hifitime::efmt::{Format, Formatter};

#[cfg(feature = "std")]
use core::f64::EPSILON;
#[cfg(not(feature = "std"))]
use std::f64::EPSILON;

#[test]
fn test_const_ops() {
    // Tests that multiplying a constant with a unit returns the correct number in that same unit
    let mjd_offset = MJD_OFFSET * Unit::Day;
    assert!((mjd_offset.to_unit(Unit::Day) - MJD_OFFSET).abs() < f64::EPSILON);
    let j2000_offset = J2000_OFFSET * Unit::Day;
    assert!((j2000_offset.to_unit(Unit::Day) - J2000_OFFSET).abs() < f64::EPSILON);
}

#[allow(clippy::float_equality_without_abs)]
#[test]
fn utc_epochs() {
    assert!(Epoch::from_mjd_tai(J1900_OFFSET).to_tai_seconds() < EPSILON);
    assert!((Epoch::from_mjd_tai(J1900_OFFSET).to_mjd_tai_days() - J1900_OFFSET).abs() < EPSILON);

    // Tests are chronological dates.
    // All of the following examples are cross validated against NASA HEASARC,
    // referred to as "X-Val" for "cross validation."

    // X-Val: 03 January 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=1&d2=03&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_199_333_568.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 1, 3, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 28 February 1938 00:00:00 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=02&d2=28&y2=1938&h1=0&i1=0&s1=0&h2=0&i2=0&s2=0
    let this_epoch = Epoch::from_tai_seconds(1_204_156_800.0);
    let epoch_utc =
        Epoch::maybe_from_gregorian_utc(1938, 2, 28, 00, 00, 00, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // 28 February 1938 23:59:59 (no X-Val: took the next test and subtracted one second)
    let this_epoch = Epoch::from_tai_seconds(1_204_243_199.0);
    let epoch_utc =
        Epoch::maybe_from_gregorian_utc(1938, 2, 28, 23, 59, 59, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 01 March 1938 00:00:00 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=3&d2=01&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_204_243_200.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 3, 1, 00, 00, 00, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 31 March 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=03&d2=31&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_206_850_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 3, 31, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 24 June 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=6&d2=24&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_214_194_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 6, 24, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 31 August 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=8&d2=31&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_220_069_568.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1938, 8, 31, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 31 December 1938 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=12&d2=31&y2=1938&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_230_610_368.0);
    let epoch_utc =
        Epoch::maybe_from_gregorian_utc(1938, 12, 31, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 January 1939 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=01&d2=1&y2=1939&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_230_696_768.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1939, 1, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 March 1939 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=3&d2=1&y2=1939&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_235_794_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1939, 3, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    // X-Val: 01 March 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=3&d2=1&y2=1940&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_267_416_768.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 3, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 February 1939 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=1&y2=1939&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_233_375_168.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1939, 2, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 01 February 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=01&y2=1940&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_264_911_168.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 2, 1, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 28 February 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=28&y2=1940&h1=0&i1=0&s1=0&h2=4&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_267_243_968.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 2, 28, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // X-Val: 29 February 1940 04:12:48 - https://www.timeanddate.com/date/durationresult.html?m1=1&d1=1&y1=1900&m2=2&d2=29&y2=1940&h1=0&i1=0&s1=0&h2=04&i2=12&s2=48
    let this_epoch = Epoch::from_tai_seconds(1_267_330_368.0);
    let epoch_utc = Epoch::maybe_from_gregorian_utc(1940, 2, 29, 4, 12, 48, 0).expect("init epoch");
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");

    // Test the specific leap second times
    let epoch_from_tai_secs = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    assert!(epoch_from_tai_secs.to_tai_seconds() - 2_272_060_800.0 < EPSILON);
    let epoch_from_tai_greg = Epoch::from_tai_seconds(2_272_060_800.0);
    assert_eq!(epoch_from_tai_greg, epoch_from_tai_secs, "Incorrect epoch");

    // Check that second leap second happens
    let epoch_from_utc_greg = Epoch::from_gregorian_utc_hms(1972, 6, 30, 23, 59, 59);
    let epoch_from_utc_greg1 = Epoch::from_gregorian_utc_hms(1972, 7, 1, 0, 0, 0);
    assert!(
        (epoch_from_utc_greg1.to_tai_seconds() - epoch_from_utc_greg.to_tai_seconds() - 2.0).abs()
            < EPSILON
    );

    // Just prior to the 2017 leap second, there should be an offset of 36 seconds between UTC and TAI
    let this_epoch = Epoch::from_tai_seconds(3_692_217_599.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 23);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    assert!(this_epoch.to_tai_seconds() - epoch_utc.to_utc_seconds() - 36.0 < EPSILON);

    // Just after to the 2017 leap second, there should be an offset of 37 seconds between UTC and TAI
    let this_epoch = Epoch::from_tai_seconds(3_692_217_600.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 24);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    assert!(this_epoch.to_tai_seconds() - epoch_utc.to_utc_seconds() - 37.0 < EPSILON);

    let mut this_epoch = Epoch::from_tai_seconds(3_692_217_600.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 24);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    this_epoch += Unit::Second * 3600.0;
    assert_eq!(
        this_epoch,
        Epoch::from_gregorian_utc_hms(2017, 1, 1, 0, 59, 23),
        "Incorrect epoch when adding an hour across leap second"
    );
    this_epoch -= Unit::Hour;
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch after sub");
    // Revert and then subassign with duration
    this_epoch += Unit::Hour;
    this_epoch -= 1 * Unit::Hour;
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch after sub");

    let this_epoch = Epoch::from_gregorian_tai_at_midnight(2020, 1, 1);
    assert!((this_epoch.to_jde_tai_days() - 2_458_849.5).abs() < EPSILON)
}

#[allow(clippy::float_equality_without_abs)]
#[test]
fn utc_tai() {
    // General note: TAI "ahead" of UTC means that there are _less_ TAI seconds since epoch for a given date
    // than there are seconds for that UTC epoch: the same TAI time happens _before_ that UTC time.

    // flp = first leap second
    let flp_from_secs_tai = Epoch::from_tai_seconds(2_272_060_800.0);
    let flp_from_greg_tai = Epoch::from_gregorian_tai_at_midnight(1972, 1, 1);
    assert_eq!(flp_from_secs_tai, flp_from_greg_tai);
    // Right after the discontinuity, UTC time should be ten seconds behind TAI, i.e. TAI is ten seconds ahead of UTC
    // In other words, the following date times are equal:
    assert_eq!(
        Epoch::from_gregorian_tai_hms(1972, 1, 1, 0, 0, 10),
        Epoch::from_gregorian_utc_at_midnight(1972, 1, 1),
        "UTC discontinuity failed"
    );
    // Noon UTC after the first leap second is in fact ten seconds _after_ noon TAI.
    // Hence, there are as many TAI seconds since Epoch between UTC Noon and TAI Noon + 10s.
    assert!(
        Epoch::from_gregorian_utc_at_noon(1972, 1, 1)
            > Epoch::from_gregorian_tai_at_noon(1972, 1, 1),
        "TAI is not ahead of UTC (via PartialEq) at noon after first leap second"
    );
    assert_eq!(
        Epoch::from_gregorian_utc_at_noon(1972, 1, 1)
            .min(Epoch::from_gregorian_tai_at_noon(1972, 1, 1)),
        Epoch::from_gregorian_tai_at_noon(1972, 1, 1),
        "TAI is not ahead of UTC (via PartialEq) at noon after first leap second"
    );
    assert_eq!(
        Epoch::from_gregorian_utc_at_noon(1972, 1, 1)
            .max(Epoch::from_gregorian_tai_at_noon(1972, 1, 1)),
        Epoch::from_gregorian_utc_at_noon(1972, 1, 1),
        "TAI is not ahead of UTC (via PartialEq) at noon after first leap second"
    );

    assert!(
        flp_from_secs_tai.to_tai_seconds() > flp_from_secs_tai.to_utc_seconds(),
        "TAI is not ahead of UTC (via function call)"
    );
    assert!(
        (flp_from_secs_tai.to_tai_seconds() - flp_from_secs_tai.to_utc_seconds() - 10.0) < EPSILON,
        "TAI is not ahead of UTC"
    );

    // Check that all of the TAI/UTC time differences are of 37.0 as of today.
    let epoch_utc = Epoch::from_gregorian_utc_hms(2019, 8, 1, 20, 10, 23);
    let epoch_tai = Epoch::from_gregorian_tai_hms(2019, 8, 1, 20, 10, 23);
    assert!(epoch_tai < epoch_utc, "TAI is not ahead of UTC");
    let delta: Duration = epoch_utc - epoch_tai - Unit::Second * 37.0;
    assert!(delta < Unit::Nanosecond, "TAI is not ahead of UTC");
    assert!(
        (epoch_utc.to_tai_seconds() - epoch_tai.to_tai_seconds() - 37.0).abs() < EPSILON,
        "TAI is not ahead of UTC"
    );
    assert!(
        (epoch_utc.to_utc_seconds() - epoch_tai.to_utc_seconds() - 37.0).abs() < EPSILON,
        "TAI is not ahead of UTC"
    );

    // Test from_utc_seconds and from_utc_days. Any effects from leap seconds
    // should have no bearing when testing two "from UTC" methods.
    assert_eq!(
        Epoch::from_gregorian_utc_at_midnight(1972, 1, 1),
        Epoch::from_utc_seconds(2_272_060_800.0)
    );
    assert_eq!(
        Epoch::from_gregorian_utc_hms(1972, 1, 1, 0, 0, 10),
        Epoch::from_utc_seconds(2_272_060_810.0)
    );
    assert_eq!(
        Epoch::from_gregorian_utc_at_midnight(1972, 1, 1),
        Epoch::from_utc_days(26297.0)
    );

    let now = Epoch::from_gregorian_tai_hms(2019, 8, 24, 3, 49, 9);
    assert!(
        now.to_tai_seconds() > now.to_utc_seconds(),
        "TAI is not ahead of UTC"
    );
    assert!((now.to_tai_seconds() - now.to_utc_seconds() - 37.0).abs() < EPSILON);
    assert!(
        now.to_tai_seconds() > now.to_gpst_seconds(),
        "TAI is not ahead of GPS Time"
    );
}

#[test]
fn julian_epoch() {
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let nist_j1900 = Epoch::from_tai_days(0.0);
    assert!((nist_j1900.to_mjd_tai_days() - 15_020.0).abs() < EPSILON);
    assert!((nist_j1900.to_jde_tai_days() - 2_415_020.5).abs() < EPSILON);
    let mjd = Epoch::from_gregorian_utc_at_midnight(1900, 1, 1);
    assert!((mjd.to_mjd_tai_days() - 15_020.0).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+12%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j1900 = Epoch::from_tai_days(0.5);
    assert!((j1900.to_mjd_tai_days() - 15_020.5).abs() < EPSILON);
    assert!((j1900.to_jde_tai_days() - 2_415_021.0).abs() < EPSILON);
    let mjd = Epoch::from_gregorian_utc_at_noon(1900, 1, 1);
    assert!((mjd.to_mjd_tai_days() - 15_020.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-08+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let mjd = Epoch::from_gregorian_utc_at_midnight(1900, 1, 8);
    assert!((mjd.to_mjd_tai_days() - 15_027.0).abs() < EPSILON);
    assert!((mjd.to_jde_tai_days() - 2_415_027.5).abs() < EPSILON);
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1980-01-06+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let gps_std_epoch = Epoch::from_gregorian_tai_at_midnight(1980, 1, 6);
    assert!((gps_std_epoch.to_mjd_tai_days() - 44_244.0).abs() < EPSILON);
    assert!((gps_std_epoch.to_jde_tai_days() - 2_444_244.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2000-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j2000 = Epoch::from_gregorian_tai_at_midnight(2000, 1, 1);
    assert!((j2000.to_mjd_tai_days() - 51_544.0).abs() < EPSILON);
    assert!((j2000.to_jde_tai_days() - 2_451_544.5).abs() < EPSILON);

    assert!(
        Epoch::from_gregorian_tai_at_midnight(2000, 1, 1)
            < Epoch::from_gregorian_utc_at_midnight(2000, 1, 1),
        "TAI not ahead of UTC on J2k"
    );

    assert_eq!(
        (Epoch::from_gregorian_utc_at_midnight(2000, 1, 1)
            - Epoch::from_gregorian_tai_at_midnight(2000, 1, 1)),
        Unit::Second * 32.0
    );

    let j2000 = Epoch::from_gregorian_utc_at_midnight(2000, 1, 1);
    assert!((j2000.to_mjd_utc_days() - 51_544.0).abs() < EPSILON);
    assert!((j2000.to_jde_utc_days() - 2_451_544.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2002-02-07+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let jd020207 = Epoch::from_gregorian_tai_at_midnight(2002, 2, 7);
    assert!((jd020207.to_mjd_tai_days() - 52_312.0).abs() < EPSILON);
    assert!((jd020207.to_jde_tai_days() - 2_452_312.5).abs() < EPSILON);

    // Test leap seconds and Julian at the same time
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // NOTE: Precision of HEASARC is less than hifitime, hence the last four digit difference
    // HEASARC reports 57203.99998843 but hifitime computes 57203.99998842592 (three additional)
    // significant digits.
    assert!(
        (Epoch::from_gregorian_tai_hms(2015, 6, 30, 23, 59, 59).to_mjd_tai_days()
            - 57_203.999_988_425_92)
            .abs()
            < EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (Epoch::from_gregorian_tai_hms(2015, 6, 30, 23, 59, 60).to_mjd_tai_days()
            - 57_203.999_988_425_92)
            .abs()
            < EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (Epoch::from_gregorian_tai_at_midnight(2015, 7, 1).to_mjd_tai_days() - 57_204.0).abs()
            < EPSILON,
        "Incorrect Post July 2015 leap second MJD computed"
    );
}

#[test]
fn datetime_invalid_dates() {
    assert!(!is_gregorian_valid(2001, 2, 29, 22, 8, 47, 0));
    assert!(!is_gregorian_valid(2016, 12, 31, 23, 59, 61, 0));
    assert!(!is_gregorian_valid(2015, 6, 30, 23, 59, 61, 0));
}

#[test]
fn gpst() {
    let ref_gps = Epoch::from_gregorian_utc_at_midnight(1980, 01, 06);

    // Test 1sec into GPS timescale
    let gnss = Epoch::from_gpst_seconds(1.0);
    assert_eq!(gnss, ref_gps + 1.0 * Unit::Second);

    // Test 1+1/2 day into GPS timescale
    let gnss = Epoch::from_gpst_days(1.5);
    assert_eq!(gnss, ref_gps + 1.5 * Unit::Day);

    let now = Epoch::from_gregorian_tai_hms(2019, 8, 24, 3, 49, 9);
    assert_eq!(
        Epoch::from_gpst_nanoseconds(now.to_gpst_nanoseconds().unwrap()),
        now,
        "To/from (recip.) GPST nanoseconds failed"
    );
    assert!(
        (now.to_tai_seconds() - SECONDS_GPS_TAI_OFFSET - now.to_gpst_seconds()).abs() < EPSILON
    );
    assert!(
        now.to_gpst_seconds() + SECONDS_GPS_TAI_OFFSET > now.to_utc_seconds(),
        "GPS Time is not ahead of UTC"
    );

    let gps_epoch = Epoch::from_tai_seconds(SECONDS_GPS_TAI_OFFSET);
    assert_eq!(format!("{}", GPST_REF_EPOCH), "1980-01-06T00:00:00 UTC");
    assert_eq!(format!("{:x}", GPST_REF_EPOCH), "1980-01-06T00:00:19 TAI");
    assert_eq!(format!("{:o}", gps_epoch), "0");
    assert_eq!(
        Epoch::from_gpst_days(0.0).to_duration_since_j1900(),
        gps_epoch.duration_since_j1900_tai
    );

    assert_eq!(
        gps_epoch.to_tai_seconds(),
        Epoch::from_gregorian_utc_at_midnight(1980, 1, 6).to_tai_seconds()
    );
    assert!(
        gps_epoch.to_gpst_seconds().abs() < EPSILON,
        "The number of seconds from the GPS epoch was not 0: {}",
        gps_epoch.to_gpst_seconds()
    );
    assert!(
        gps_epoch.to_gpst_days().abs() < EPSILON,
        "The number of days from the GPS epoch was not 0: {}",
        gps_epoch.to_gpst_days()
    );

    let epoch = Epoch::from_gregorian_utc_at_midnight(1972, 1, 1);
    assert!(
        (epoch.to_tai_seconds() - SECONDS_GPS_TAI_OFFSET - epoch.to_gpst_seconds()).abs() < EPSILON
    );
    assert!((epoch.to_tai_days() - DAYS_GPS_TAI_OFFSET - epoch.to_gpst_days()).abs() < 1e-11);

    // 1 Jan 1980 is 5 days before the GPS epoch.
    let epoch = Epoch::from_gregorian_utc_at_midnight(1980, 1, 1);
    assert!((epoch.to_gpst_seconds() + 5.0 * SECONDS_PER_DAY).abs() < EPSILON);
    assert!((epoch.to_gpst_days() + 5.0).abs() < EPSILON);
}

#[test]
fn galileo_time_scale() {
    let now = Epoch::from_gregorian_tai_hms(2019, 8, 24, 3, 49, 9);
    let gst_nanos = now.to_gst_nanoseconds().unwrap();
    assert_eq!(
        Epoch::from_gst_nanoseconds(gst_nanos),
        now,
        "To/from (recip.) GPST nanoseconds failed"
    );
    assert!((now.to_tai_seconds() - SECONDS_GST_TAI_OFFSET - now.to_gst_seconds()).abs() < EPSILON);
    assert!(
        now.to_gst_seconds() + SECONDS_GST_TAI_OFFSET > now.to_utc_seconds(),
        "GST Time is not ahead of UTC"
    );

    let ref_gst = Epoch::from_gst_nanoseconds(0);
    assert_eq!(format!("{}", ref_gst), "1999-08-21T23:59:47 UTC");

    let gst_epoch = Epoch::from_tai_seconds(SECONDS_GST_TAI_OFFSET);
    assert_eq!(gst_epoch, Epoch::from_gst_days(0.0));
    assert_eq!(gst_epoch, Epoch::from_gst_seconds(0.0));
    assert_eq!(gst_epoch, Epoch::from_gst_nanoseconds(0));
    assert_eq!(gst_epoch, GST_REF_EPOCH);
    assert_eq!(format!("{}", GST_REF_EPOCH), "1999-08-21T23:59:47 UTC");
    assert_eq!(format!("{:x}", GST_REF_EPOCH), "1999-08-22T00:00:19 TAI");
    assert_eq!(
        Epoch::from_gst_days(0.0).to_duration_since_j1900(),
        gst_epoch.duration_since_j1900_tai
    );

    assert_eq!(
        gst_epoch.to_tai_seconds(),
        Epoch::from_gregorian_utc_at_midnight(1999, 08, 22).to_tai_seconds() - 13.0
    );
    assert!(
        gst_epoch.to_gst_seconds().abs() < EPSILON,
        "The number of seconds from the GST epoch was not 0: {}",
        gst_epoch.to_gst_seconds()
    );
    assert!(
        gst_epoch.to_gst_days().abs() < EPSILON,
        "The number of days from the GST epoch was not 0: {}",
        gst_epoch.to_gst_days()
    );
}

#[test]
fn beidou_time_scale() {
    let now = Epoch::from_gregorian_tai_hms(2019, 8, 24, 3, 49, 9);
    let nanos = now.to_bdt_nanoseconds().unwrap();
    assert_eq!(
        Epoch::from_bdt_nanoseconds(nanos),
        now,
        "To/from (recip.) BDT nanoseconds failed"
    );
    assert!((now.to_tai_seconds() - SECONDS_BDT_TAI_OFFSET - now.to_bdt_seconds()).abs() < EPSILON);
    assert!(
        now.to_bdt_seconds() + SECONDS_BDT_TAI_OFFSET > now.to_utc_seconds(),
        "BDT Time is not ahead of UTC"
    );

    let bdt_epoch = Epoch::from_tai_seconds(SECONDS_BDT_TAI_OFFSET);
    assert_eq!(bdt_epoch, Epoch::from_bdt_days(0.0));
    assert_eq!(bdt_epoch, Epoch::from_bdt_seconds(0.0));
    assert_eq!(bdt_epoch, Epoch::from_bdt_nanoseconds(0));
    assert_eq!(bdt_epoch, BDT_REF_EPOCH);

    assert_eq!(format!("{bdt_epoch}"), "2006-01-01T00:00:00 UTC");
    assert_eq!(format!("{bdt_epoch:x}"), "2006-01-01T00:00:33 TAI");

    assert_eq!(
        Epoch::from_bdt_days(0.0).to_duration_since_j1900(),
        bdt_epoch.duration_since_j1900_tai
    );

    assert_eq!(
        bdt_epoch.to_tai_seconds(),
        Epoch::from_gregorian_utc_at_midnight(2006, 01, 01).to_tai_seconds()
    );
    assert!(
        bdt_epoch.to_bdt_seconds().abs() < EPSILON,
        "The number of seconds from the BDT epoch was not 0: {}",
        bdt_epoch.to_bdt_seconds()
    );
    assert!(
        bdt_epoch.to_bdt_days().abs() < EPSILON,
        "The number of days from the BDT epoch was not 0: {}",
        bdt_epoch.to_bdt_days()
    );
}

#[test]
fn unix() {
    // Continuous check that the system time as reported by this machine is within millisecond accuracy of what we compute
    #[cfg(feature = "std")]
    {
        use std::time::SystemTime;

        let std_unix_time_s = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let epoch_unix_time_s = Epoch::now().unwrap().to_unix_seconds();

        assert!(
            (std_unix_time_s - epoch_unix_time_s).abs() < 1e-5,
            "hifitime and std differ in UNIX by more than 10 microseconds: hifitime = {}\tstd = {}",
            epoch_unix_time_s,
            std_unix_time_s
        );
    }

    let now = Epoch::from_gregorian_utc_hms(2022, 5, 2, 10, 39, 15);
    assert!((now.to_unix_seconds() - 1651487955.0_f64).abs() < EPSILON);
    assert!((now.to_unix_milliseconds() - 1651487955000.0_f64).abs() < EPSILON);
    assert_eq!(
        Epoch::from_unix_seconds(now.to_unix_seconds()),
        now,
        "To/from UNIX seconds failed"
    );
    assert_eq!(
        Epoch::from_unix_milliseconds(now.to_unix_milliseconds()),
        now,
        "To/from UNIX milliseconds failed"
    );

    let unix_epoch = Epoch::from_gregorian_utc_at_midnight(1970, 1, 1);

    assert_eq!(
        format!("{}", unix_epoch.in_time_scale(TimeScale::UTC)),
        "1970-01-01T00:00:00 UTC"
    );
    assert_eq!(
        format!("{:x}", unix_epoch.in_time_scale(TimeScale::TAI)),
        "1970-01-01T00:00:00 TAI"
    );
    // Print as UNIX seconds
    assert_eq!(format!("{:p}", unix_epoch), "0");

    assert_eq!(
        unix_epoch.to_tai_seconds(),
        Epoch::from_gregorian_utc_at_midnight(1970, 1, 1).to_tai_seconds()
    );
    assert!(
        unix_epoch.to_unix_seconds().abs() < EPSILON,
        "The number of seconds from the UNIX epoch was not 0: {}",
        unix_epoch.to_unix_seconds()
    );
    assert!(
        unix_epoch.to_unix_milliseconds().abs() < EPSILON,
        "The number of milliseconds from the UNIX epoch was not 0: {}",
        unix_epoch.to_unix_seconds()
    );
    assert!(
        unix_epoch.to_unix_days().abs() < EPSILON,
        "The number of days from the UNIX epoch was not 0: {}",
        unix_epoch.to_unix_days()
    );
}

/// This test has a series of verifications between NAIF SPICE and hifitime.
/// All of the test cases were create using spiceypy and cover a range of values from J1900 to J2100.
/// All of the test cases include the UTC conversion, the JDE computation, and the reciprocity within hifitime.
/// To compute the JD date, the following function is used: sp.et2utc(sp.utc2et(date_str), "J", 9)
#[test]
fn naif_spice_et_tdb_verification() {
    // The maximum error due to small perturbations accounted for in ESA algorithm but not SPICE algorithm.
    let max_tdb_et_err = 32 * Unit::Microsecond;
    // Prior to 01 JAN 1972, IERS claims that there is no leap second at all but SPICE claims that there are nine (9) leap seconds
    // between TAI and UTC. Hifitime also claims that there are zero leap seconds (to ensure correct computation of UNIX time at its reference time).
    let spice_utc_tai_ls_err = 9.0;
    // SPICE will only output up to 6 digits for the JDE computation. This is likely due to the precision limitation of the `double`s type.
    // This means that a SPICE JDE is precise to 0.008 seconds, whereas a JDE in Hifitime maintains its nanosecond precision.
    let spice_jde_precision = 1e-7;
    // We allow for 2 microseconds of error in the reciprocity because the SPICE input is in microseconds as well.
    let recip_err_s = 2e-6;

    // The general test function used throughout this verification.
    let spice_verif_func = |epoch: Epoch, et_s: f64, utc_jde_d: f64| {
        // Test reciprocity
        assert!(
            (Epoch::from_et_seconds(et_s).to_et_seconds() - et_s).abs() < recip_err_s,
            "{} failed ET reciprocity test:\nwant: {}\tgot: {}\nerror: {} ns",
            epoch,
            et_s,
            Epoch::from_et_seconds(et_s).to_et_seconds(),
            (et_s - Epoch::from_et_seconds(et_s).to_et_seconds()).abs() * 1e9
        );
        assert!(
            (Epoch::from_tdb_seconds(et_s).to_tdb_seconds() - et_s).abs() < recip_err_s,
            "{} failed TDB reciprocity test:\nwant: {}\tgot: {}\nerror: {} ns",
            epoch,
            et_s,
            Epoch::from_tdb_seconds(et_s).to_tdb_seconds(),
            (et_s - Epoch::from_tdb_seconds(et_s).to_et_seconds()).abs() * 1e9
        );

        // Test ET computation
        let extra_seconds = if epoch.leap_seconds_iers() == 0 {
            spice_utc_tai_ls_err
        } else {
            0.0
        };
        assert!(
            (epoch.to_et_seconds() - et_s + extra_seconds).abs() < EPSILON,
            "{} failed ET test",
            epoch
        );

        // Test TDB computation
        assert!(
            (epoch.to_tdb_duration() - et_s * Unit::Second + extra_seconds * Unit::Second).abs()
                <= max_tdb_et_err,
            "{} failed TDB test",
            epoch
        );

        // TEST JDE computation
        assert!(
            (epoch.to_jde_utc_days() - utc_jde_d).abs() < spice_jde_precision,
            "{} failed JDE UTC days test:\nwant: {}\tgot: {}\nerror = {} days",
            epoch,
            utc_jde_d,
            epoch.to_jde_utc_days(),
            (epoch.to_jde_utc_days() - utc_jde_d).abs()
        );
    };

    // sp.utc2et('1900-01-09 00:17:15.0 UTC')
    spice_verif_func(
        Epoch::from_gregorian_utc(1900, 1, 9, 0, 17, 15, 0),
        -3155024523.8157988,
        2415028.5119792,
    );

    // sp.utc2et('1920-07-23 14:39:29.0 UTC')
    spice_verif_func(
        Epoch::from_gregorian_utc(1920, 7, 23, 14, 39, 29, 0),
        -2506972789.816543,
        2422529.1107523,
    );

    // sp.utc2et('1954-12-24 06:06:31.0 UTC')
    spice_verif_func(
        Epoch::from_gregorian_utc(1954, 12, 24, 6, 6, 31, 0),
        -1420782767.8162904,
        2435100.7545255,
    );

    // Test prior to official leap seconds but with some scaling, valid from 1960 to 1972 according to IAU SOFA.
    spice_verif_func(
        Epoch::from_gregorian_utc(1960, 2, 14, 6, 6, 31, 0),
        -1258523567.8148985,
        2436978.7545255,
    );

    // First test with some leap seconds
    // sp.utc2et('1983 APR 13 12:09:14.274')
    spice_verif_func(
        Epoch::from_gregorian_utc(1983, 4, 13, 12, 9, 14, 274_000_000),
        -527644192.54036534,
        2445438.0064152,
    );

    // Once every 400 years, there is a leap day on the new century! Joyeux anniversaire, Papa!
    // sp.utc2et('2000-02-29 14:57:29.0')
    spice_verif_func(
        Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 0),
        5108313.185383182,
        2451604.1232523,
    );

    // sp.utc2et('2022-11-29 07:58:49.782')
    spice_verif_func(
        Epoch::from_gregorian_utc(2022, 11, 29, 7, 58, 49, 782_000_000),
        722980798.9650334,
        2459912.8325206,
    );

    // sp.utc2et('2044-06-06 12:18:54.0')
    spice_verif_func(
        Epoch::from_gregorian_utc(2044, 6, 6, 12, 18, 54, 0),
        1402100403.1847699,
        2467773.0131250,
    );

    // sp.utc2et('2075-04-30 23:59:54.0')
    spice_verif_func(
        Epoch::from_gregorian_utc(2075, 4, 30, 23, 59, 54, 0),
        2377166463.185493,
        2479058.4999306,
    );
}

#[test]
fn spice_et_tdb() {
    // NOTE: This test has been mostly superseded by the much more thorough `naif_spice_et_tdb_verification`.
    // But it is kept for posteriority.

    // The maximum error due to small perturbations accounted for in ESA algorithm but not SPICE algorithm.
    let max_tdb_et_err = 30 * Unit::Microsecond;
    // The maximum precision that spiceypy/SPICE allow when calling `utc2et`
    let max_prec = 10 * Unit::Nanosecond;
    /*
    >>> sp.str2et("2012-02-07 11:22:33 UTC")
    381885819.18493587
    >>> sp.et2utc(381885819.18493587, 'C', 9)
    '2012 FEB 07 11:22:33.000000000'
    >>> sp.et2utc(381885819.18493587, 'J', 9)
    'JD 2455964.9739931'
    */
    let sp_ex = Epoch::from_gregorian_utc_hms(2012, 2, 7, 11, 22, 33);
    let expected_et_s = 381_885_819.184_935_87;
    // Check reciprocity
    let from_et_s = Epoch::from_tdb_seconds(expected_et_s);
    assert!((from_et_s.to_tdb_seconds() - expected_et_s).abs() < EPSILON);
    // Validate UTC to ET when initialization from UTC
    assert!(dbg!(sp_ex.to_et_seconds() - expected_et_s).abs() < max_prec.to_seconds());
    assert!(dbg!(sp_ex.to_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.to_seconds());
    assert!(dbg!(sp_ex.to_jde_utc_days() - 2455964.9739931).abs() < 1e-7);
    assert!(dbg!(sp_ex.to_tai_seconds() - from_et_s.to_tai_seconds()).abs() < 3e-6);

    // Second example
    let sp_ex = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);
    let expected_et_s = 66_312_064.184_938_76;
    assert!(dbg!(sp_ex.to_et_seconds() - expected_et_s).abs() < max_prec.to_seconds());
    assert!(dbg!(sp_ex.to_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.to_seconds());
    assert!(
        (sp_ex.to_tai_seconds() - Epoch::from_tdb_seconds(expected_et_s).to_tai_seconds()).abs()
            < 1e-5
    );

    // Third example
    let sp_ex = Epoch::from_gregorian_utc_hms(1996, 2, 7, 11, 22, 33);
    let expected_et_s = -123_035_784.815_060_48;
    assert!(dbg!(sp_ex.to_et_seconds() - expected_et_s).abs() < max_prec.to_seconds());
    assert!(dbg!(sp_ex.to_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.to_seconds());
    assert!(
        dbg!(sp_ex.to_tai_seconds() - Epoch::from_tdb_seconds(expected_et_s).to_tai_seconds())
            .abs()
            < 1e-5
    );
    // Fourth example
    /*
    >>> sp.str2et("2015-02-07 11:22:33 UTC")
    476580220.1849411
    >>> sp.et2utc(476580220.1849411, 'C', 9)
    '2015 FEB 07 11:22:33.000000000'
    >>> sp.et2utc(476580220.1849411, 'J', 9)
    'JD 2457060.9739931'
    >>>
    */
    let sp_ex = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
    let expected_et_s = 476580220.1849411;
    assert!(dbg!(sp_ex.to_et_seconds() - expected_et_s).abs() < max_prec.to_seconds());
    assert!(dbg!(sp_ex.to_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.to_seconds());
    assert!((sp_ex.to_jde_utc_days() - 2457060.9739931).abs() < 1e-7);

    // JDE TDB tests
    /* Initial JDE from sp.et2utc:
    >>> sp.str2et("JD 2452312.500372511 TDB")
    66312032.18493909
    */
    // 2002-02-07T00:00:00.4291 TAI
    let sp_ex = Epoch::from_tdb_seconds(66_312_032.184_939_09);
    assert!((2452312.500372511 - sp_ex.to_jde_et_days()).abs() < EPSILON);
    assert!((2452312.500372511 - sp_ex.to_jde_tdb_days()).abs() < EPSILON);
    // Confirm that they are _not_ equal, only that the number of days in f64 is equal
    assert_ne!(sp_ex.to_jde_et_duration(), sp_ex.to_jde_tdb_duration());

    // 2012-02-07T11:22:00.818924427 TAI
    let sp_ex = Epoch::from_tdb_seconds(381_885_753.003_859_5);
    assert!((2455964.9739931 - sp_ex.to_jde_et_days()).abs() < EPSILON);
    assert!((2455964.9739931 - sp_ex.to_jde_tdb_days()).abs() < max_tdb_et_err.to_seconds());
}

#[test]
fn test_from_str() {
    use core::str::FromStr;

    let dt = Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 0);
    assert_eq!(dt, Epoch::from_str("2017-01-14T00:31:55 UTC").unwrap());
    assert_eq!(dt, Epoch::from_str("2017-01-14T00:31:55").unwrap());
    assert_eq!(dt, Epoch::from_str("2017-01-14 00:31:55").unwrap());
    assert!(Epoch::from_str("2017-01-14 00:31:55 TAI").is_ok());
    assert!(Epoch::from_str("2017-01-14 00:31:55 TT").is_ok());
    assert!(Epoch::from_str("2017-01-14 00:31:55 ET").is_ok());
    assert!(Epoch::from_str("2017-01-14 00:31:55 TDB").is_ok());

    let jde = 2_452_312.500_372_511;
    let to_tdb = Epoch::from_str("JD 2452312.500372511 TDB").unwrap();
    let to_et = Epoch::from_str("JD 2452312.500372511 ET").unwrap();
    let to_tai = Epoch::from_str("JD 2452312.500372511 TAI").unwrap();

    // The JDE only has a precision of 1e-9 days, so we can only compare down to that
    const SPICE_EPSILON: f64 = 1e-9;
    assert!((to_tdb.to_jde_tdb_days() - jde).abs() < SPICE_EPSILON);
    assert!((to_et.to_jde_et_days() - jde).abs() < SPICE_EPSILON);
    assert!((to_tai.to_jde_tai_days() - jde).abs() < SPICE_EPSILON);
    assert!(
        (Epoch::from_str("MJD 51544.5 TAI")
            .unwrap()
            .to_mjd_tai_days()
            - 51544.5)
            .abs()
            < EPSILON
    );
    assert!((Epoch::from_str("SEC 0.5 TAI").unwrap().to_tai_seconds() - 0.5).abs() < EPSILON);

    // Must account for the precision error
    assert!(
        (Epoch::from_str("SEC 66312032.18493909 TDB")
            .unwrap()
            .to_tdb_seconds()
            - 66312032.18493909)
            .abs()
            < 1e-4
    );

    // Check reciprocity of string
    let greg = "2020-01-31T00:00:00 UTC";
    assert_eq!(greg, format!("{}", Epoch::from_str(greg).unwrap()));

    let greg = "2020-01-31T00:00:00 TAI";
    assert_eq!(greg, format!("{:x}", Epoch::from_str(greg).unwrap()));

    let greg = "2020-01-31T00:00:00 TDB";
    assert_eq!(greg, format!("{:e}", Epoch::from_str(greg).unwrap()));

    // Newton Raphson of ET leads to an 11 nanosecond error in this case.
    let greg = "2020-01-31T00:00:00 ET";
    assert_eq!(
        "2020-01-31T00:00:00.000000011 ET",
        format!("{:E}", Epoch::from_str(greg).unwrap())
    );

    // Regression test for #90
    assert_eq!(
        Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 811000000),
        Epoch::from_gregorian_str("2017-01-14 00:31:55.811 UTC").unwrap()
    );

    assert_eq!(
        Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 811000000),
        Epoch::from_gregorian(2017, 1, 14, 0, 31, 55, 811000000, TimeScale::UTC),
    );

    // Check that we can correctly parse the result from a hardware clock.
    assert!(Epoch::from_str("2022-12-15 13:32:16.421857-07:00").is_ok());

    assert_eq!(
        Epoch::from_str("blah"),
        Err(Errors::ParseError(ParsingErrors::UnknownFormat))
    );
}

#[test]
fn test_from_str_tdb() {
    use core::str::FromStr;

    let greg = "2020-01-31T00:00:00 TDB";
    assert_eq!(greg, format!("{:e}", Epoch::from_str(greg).unwrap()));
}

#[test]
fn test_rfc3339() {
    use core::str::FromStr;

    assert_eq!(
        Epoch::from_gregorian_utc_hms(1994, 11, 5, 13, 15, 30),
        Epoch::from_str("1994-11-05T08:15:30-05:00").unwrap()
    );
    assert_eq!(
        Epoch::from_gregorian_utc_hms(1994, 11, 5, 13, 15, 30),
        Epoch::from_str("1994-11-05T13:15:30Z").unwrap()
    );
    // Same test with different time systems
    // TAI
    assert_eq!(
        Epoch::from_gregorian_tai_hms(1994, 11, 5, 13, 15, 30),
        Epoch::from_str("1994-11-05T08:15:30-05:00 TAI").unwrap()
    );
    assert_eq!(
        Epoch::from_gregorian_tai_hms(1994, 11, 5, 13, 15, 30),
        Epoch::from_str("1994-11-05T13:15:30Z TAI").unwrap()
    );
    // TDB
    assert_eq!(
        Epoch::from_gregorian_hms(1994, 11, 5, 13, 15, 30, TimeScale::TDB),
        Epoch::from_str("1994-11-05T08:15:30-05:00 TDB").unwrap()
    );
    assert_eq!(
        Epoch::from_gregorian_hms(1994, 11, 5, 13, 15, 30, TimeScale::TDB),
        Epoch::from_str("1994-11-05T13:15:30Z TDB").unwrap()
    );
}

#[test]
fn test_format() {
    use core::str::FromStr;
    use hifitime::Epoch;

    let epoch = Epoch::from_gregorian_utc_hms(2022, 9, 6, 23, 24, 29);

    // Check the ET computation once more
    assert!((epoch.to_et_seconds() - 715778738.1825389).abs() < EPSILON);

    // This was initialized as UTC, so the debug print is UTC.
    assert_eq!(format!("{epoch:?}"), "2022-09-06T23:24:29 UTC");
    assert_eq!(format!("{epoch}"), "2022-09-06T23:24:29 UTC");
    assert_eq!(format!("{epoch:x}"), "2022-09-06T23:25:06 TAI");
    assert_eq!(format!("{epoch:X}"), "2022-09-06T23:25:38.184000000 TT");
    assert_eq!(format!("{epoch:E}"), "2022-09-06T23:25:38.182538909 ET");
    assert_eq!(format!("{epoch:e}"), "2022-09-06T23:25:38.182541259 TDB");
    assert_eq!(format!("{epoch:p}"), "1662506669"); // UNIX seconds
    assert_eq!(format!("{epoch:o}"), "1346541887000000000"); // GPS nanoseconds

    // Ensure that the appropriate time system is used in the debug print.
    for ts_u8 in 0..=7 {
        let ts: TimeScale = ts_u8.into();

        let recent = Epoch::from_gregorian(2020, 9, 6, 23, 24, 29, 2, ts);
        let post_ref = Epoch::from_gregorian_hms(1900, 1, 1, 0, 0, 1, ts);
        let pre_ref = Epoch::from_gregorian_hms(1899, 12, 31, 23, 59, 59, ts);
        let way_old = Epoch::from_gregorian(1820, 9, 6, 23, 24, 29, 2, ts);

        // TDB building may have a 2 nanosecond error is seems
        assert!(
            ((post_ref - pre_ref) - 2 * Unit::Second).abs() < 2 * Unit::Nanosecond,
            "delta time should be 2 s in {ts:?} but is {}",
            post_ref - pre_ref
        );

        for (i, epoch) in [recent, post_ref, pre_ref, way_old].iter().enumerate() {
            // Check that the formatting is correct (and duration for the easy ones)
            if ts == TimeScale::TAI {
                match i {
                    0 => assert_eq!(format!("{epoch:x}"), "2020-09-06T23:24:29.000000002 TAI"),
                    1 => {
                        assert_eq!(epoch.duration_since_j1900_tai, 1 * Unit::Second);
                        assert_eq!(format!("{epoch:x}"), "1900-01-01T00:00:01 TAI")
                    }
                    2 => {
                        assert_eq!(epoch.duration_since_j1900_tai, -1 * Unit::Second);
                        assert_eq!(format!("{epoch:x}"), "1899-12-31T23:59:59 TAI")
                    }
                    3 => assert_eq!(format!("{epoch:x}"), "1820-09-06T23:24:29.000000002 TAI"),
                    _ => {}
                }
            }

            assert_eq!(
                format!("{epoch:?}"),
                match ts {
                    TimeScale::TAI => format!("{epoch:x}"),
                    TimeScale::ET => format!("{epoch:E}"),
                    TimeScale::TDB => format!("{epoch:e}"),
                    TimeScale::TT => format!("{epoch:X}"),
                    TimeScale::UTC => format!("{epoch}"),
                    TimeScale::GPST => format!("{epoch:x}").replace("TAI", "GPST"),
                    TimeScale::GST => format!("{epoch:x}").replace("TAI", "GST"),
                    TimeScale::BDT => format!("{epoch:x}").replace("TAI", "BDT"),
                }
            );

            // Check that we can correctly parse the date we print.
            match Epoch::from_str(&format!("{epoch:?}")) {
                Ok(rebuilt) => {
                    if ts == TimeScale::ET {
                        // ET has a Newton Raphston iteration for rebuilding, so we allow for a small time error.
                        assert!(
                            (rebuilt - *epoch) < 30.0 * Unit::Microsecond,
                            "#{i} error = {}\ngot = {}\nwant: {}",
                            rebuilt - *epoch,
                            rebuilt.duration_since_j1900_tai,
                            epoch.duration_since_j1900_tai
                        )
                    } else {
                        assert_eq!(
                            &rebuilt,
                            epoch,
                            "#{i} error = {}\ngot = {}\nwant: {}",
                            rebuilt - *epoch,
                            rebuilt.duration_since_j1900_tai,
                            epoch.duration_since_j1900_tai
                        )
                    }
                }
                Err(e) => {
                    panic!(
                        "#{i} {e:?} with {epoch:?} (duration since j1900 = {})",
                        epoch.duration_since_j1900_tai
                    )
                }
            };
        }
    }

    // Check the leap day formatting to/from works correctly.
    let epoch = Epoch::from_gregorian_utc_hms(2020, 1, 2, 23, 24, 29);
    assert_eq!(format!("{epoch:?}"), "2020-01-02T23:24:29 UTC");

    // Try with epochs near 1900, reference of TAI
    let epoch_post = Epoch::from_gregorian_tai_hms(1900, 1, 1, 0, 0, 1);
    let epoch_pre = Epoch::from_gregorian_tai_hms(1899, 12, 31, 23, 59, 59);

    assert_eq!(epoch_post.duration_since_j1900_tai.decompose().0, 0);
    assert_eq!(epoch_pre.duration_since_j1900_tai.decompose().0, -1);
}

#[test]
fn ops() {
    // Test adding a second
    let sp_ex: Epoch = Epoch::from_gregorian_utc_hms(2012, 2, 7, 11, 22, 33) + Unit::Second * 1.0;
    let expected_et_s = 381_885_819.184_935_87;
    assert!(dbg!(sp_ex.to_tdb_seconds() - expected_et_s - 1.0).abs() < 2.6e-6);
    let sp_ex: Epoch = sp_ex - Unit::Second * 1.0;
    assert!((sp_ex.to_tdb_seconds() - expected_et_s).abs() < 2.6e-6);
}

#[test]
fn test_range() {
    let start = Epoch::from_gregorian_utc_hms(2012, 2, 7, 11, 22, 33);
    let middle = Epoch::from_gregorian_utc_hms(2012, 2, 30, 0, 11, 22);
    let end = Epoch::from_gregorian_utc_hms(2012, 3, 7, 11, 22, 33);
    let rng = start..end;
    assert_eq!(rng, core::ops::Range { start, end });
    assert!(rng.contains(&middle));
}

#[test]
fn regression_test_gh_85() {
    let earlier_epoch =
        Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 100, TimeScale::TAI).unwrap();
    let later_epoch =
        Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 200, TimeScale::TAI).unwrap();

    assert!(
        later_epoch > earlier_epoch,
        "later_epoch should be 100ns after earlier_epoch"
    );
}

#[test]
fn test_leap_seconds_iers() {
    // Just before the very first leap second.
    let epoch_from_utc_greg = Epoch::from_gregorian_tai_hms(1971, 12, 31, 23, 59, 59);
    // Just after it.
    let epoch_from_utc_greg1 = Epoch::from_gregorian_tai_hms(1972, 1, 1, 0, 0, 0);
    assert_eq!(epoch_from_utc_greg1.day_of_year(), 0.0);
    assert_eq!(epoch_from_utc_greg.leap_seconds_iers(), 0);
    // The first leap second is special; it adds 10 seconds.
    assert_eq!(epoch_from_utc_greg1.leap_seconds_iers(), 10);

    // Just before the second leap second.
    let epoch_from_utc_greg = Epoch::from_gregorian_tai_hms(1972, 6, 30, 23, 59, 59);
    assert_eq!(
        epoch_from_utc_greg.duration_in_year(),
        (31 + 29 + 31 + 30 + 31 + 30) * Unit::Day - Unit::Second
    );
    // Just after it.
    let epoch_from_utc_greg1 = Epoch::from_gregorian_tai_hms(1972, 7, 1, 0, 0, 0);
    assert_eq!(epoch_from_utc_greg.leap_seconds_iers(), 10);
    assert_eq!(epoch_from_utc_greg1.leap_seconds_iers(), 11);
}

#[cfg(feature = "std")]
#[test]
fn test_utc_str() {
    let dt_str = "2017-01-14T00:31:55 UTC";
    let dt = Epoch::from_gregorian_str(dt_str).unwrap();
    let (centuries, nanos) = dt.to_tai_duration().to_parts();
    assert_eq!(centuries, 1);
    assert_eq!(nanos, 537582752000000000);
}

#[test]
fn test_floor_ceil_round() {
    // NOTE: This test suite is more limited than the Duration equivalent because Epoch uses Durations for these operations.
    use hifitime::TimeUnits;

    let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    assert_eq!(
        e.ceil(1.hours()),
        Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    );
    assert_eq!(
        e.floor(1.hours()),
        Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 0, 0)
    );
    assert_eq!(
        e.round(1.hours()),
        Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    );
}

#[test]
fn test_ord() {
    let epoch1 = Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 100, TimeScale::TAI).unwrap();
    let epoch2 = Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 200, TimeScale::TAI).unwrap();

    assert_eq!(epoch1.max(epoch2), epoch2);
    assert_eq!(epoch2.min(epoch1), epoch1);
    assert_eq!(epoch1.cmp(&epoch1), core::cmp::Ordering::Equal);
}

#[test]
fn regression_test_gh_145() {
    // Ceil and floor in the TAI time system
    let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
    assert_eq!(
        e.floor(3.minutes()),
        Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 42, 0)
    );

    assert_eq!(
        e.ceil(3.minutes()),
        Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 45, 0)
    );

    assert_eq!(
        e.round(3.minutes()),
        Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 45, 0)
    );

    assert_eq!(
        e.round(3.minutes()),
        Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 45, 0)
    );

    // Same in UTC
    let e = Epoch::from_gregorian_utc(2022, 10, 3, 17, 44, 29, 898032665);
    assert_eq!(
        e.floor(3.minutes()),
        Epoch::from_gregorian_utc_hms(2022, 10, 3, 17, 42, 0)
    );

    assert_eq!(
        e.ceil(3.minutes()),
        Epoch::from_gregorian_utc_hms(2022, 10, 3, 17, 45, 0)
    );

    assert_eq!(
        e.round(3.minutes()),
        Epoch::from_gregorian_utc_hms(2022, 10, 3, 17, 45, 0)
    );

    // Same in TT
    let e = Epoch::from_gregorian(2022, 10, 3, 17, 44, 29, 898032665, TimeScale::TT);
    assert_eq!(
        e.floor(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 42, 0, TimeScale::TT)
    );

    assert_eq!(
        e.ceil(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 45, 0, TimeScale::TT)
    );

    assert_eq!(
        e.round(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 45, 0, TimeScale::TT)
    );

    // Same in TDB
    let e = Epoch::from_gregorian(2022, 10, 3, 17, 44, 29, 898032665, TimeScale::TDB);
    assert_eq!(
        e.floor(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 42, 0, TimeScale::TDB)
    );

    assert_eq!(
        e.ceil(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 45, 0, TimeScale::TDB)
    );

    assert_eq!(
        e.round(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 45, 0, TimeScale::TDB)
    );

    // Same in ET
    let e = Epoch::from_gregorian(2022, 10, 3, 17, 44, 29, 898032665, TimeScale::ET);
    assert_eq!(
        e.floor(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 42, 0, TimeScale::ET)
    );

    assert_eq!(
        e.ceil(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 45, 0, TimeScale::ET)
    );

    assert_eq!(
        e.round(3.minutes()),
        Epoch::from_gregorian_hms(2022, 10, 3, 17, 45, 0, TimeScale::ET)
    );
}

/// Tests that for a number of epochs covering different leap seconds, creating an Epoch with a given time scale will allow us to retrieve in that same time scale with the same value.
#[test]
fn test_timescale_recip() {
    // The general test function used throughout this verification.
    let recip_func = |utc_epoch: Epoch| {
        assert_eq!(utc_epoch, utc_epoch.set(utc_epoch.to_duration()));
        // Test that we can convert this epoch into another time scale and re-initialize it correctly from that value.
        for ts in &[
            // TimeScale::TAI,
            TimeScale::ET,
            TimeScale::TDB,
            TimeScale::TT,
            TimeScale::UTC,
        ] {
            let converted = utc_epoch.to_duration_in_time_scale(*ts);
            let from_dur = Epoch::from_duration(converted, *ts);
            if *ts == TimeScale::ET {
                // There is limitation in the ET scale due to the Newton Raphson iteration.
                // So let's check for a near equality
                // TODO: Make this more strict
                assert!(
                    (utc_epoch - from_dur).abs() < 150 * Unit::Nanosecond,
                    "ET recip error = {} for {}",
                    utc_epoch - from_dur,
                    utc_epoch
                );
            } else {
                assert_eq!(utc_epoch, from_dur);
            }

            // RFC3339 test
            #[cfg(feature = "std")]
            {
                use core::str::FromStr;
                let to_rfc = utc_epoch.to_rfc3339();
                let from_rfc = Epoch::from_str(&to_rfc).unwrap();

                println!(
                    "{} for {utc_epoch}\tto = {to_rfc}\tfrom={from_rfc}",
                    from_rfc - utc_epoch
                );

                assert_eq!(from_rfc, utc_epoch);
            }
        }
    };

    recip_func(Epoch::from_gregorian_utc(1900, 1, 9, 0, 17, 15, 0));

    recip_func(Epoch::from_gregorian_utc(1920, 7, 23, 14, 39, 29, 0));

    recip_func(Epoch::from_gregorian_utc(1954, 12, 24, 6, 6, 31, 0));

    // Test prior to official leap seconds but with some scaling, valid from 1960 to 1972 according to IAU SOFA.
    recip_func(Epoch::from_gregorian_utc(1960, 2, 14, 6, 6, 31, 0));

    // First test with some leap seconds
    recip_func(Epoch::from_gregorian_utc(
        1983,
        4,
        13,
        12,
        9,
        14,
        274_000_000,
    ));

    // Once every 400 years, there is a leap day on the new century! Joyeux anniversaire, Papa!
    recip_func(Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 0));

    recip_func(Epoch::from_gregorian_utc(
        2022,
        11,
        29,
        7,
        58,
        49,
        782_000_000,
    ));

    recip_func(Epoch::from_gregorian_utc(2044, 6, 6, 12, 18, 54, 0));

    recip_func(Epoch::from_gregorian_utc(2075, 4, 30, 23, 59, 54, 0));
}

/// Tests that the time scales are included when performing operations on Epochs.
#[test]
fn test_add_durations_over_leap_seconds() {
    // Noon UTC after the first leap second is in fact ten seconds _after_ noon TAI.
    // Hence, there are as many TAI seconds since Epoch between UTC Noon and TAI Noon + 10s.
    let pre_ls_utc = Epoch::from_gregorian_utc_at_noon(1971, 12, 31);
    let pre_ls_tai = pre_ls_utc.in_time_scale(TimeScale::TAI);

    // Before the first leap second, there is no time difference between both epochs (because only IERS announced leap seconds are accounted for by default).
    assert_eq!(pre_ls_utc - pre_ls_tai, Duration::ZERO);
    // When add 24 hours to either of the them, the UTC initialized epoch will increase the duration by 36 hours in UTC, which will cause a leap second jump.
    // Therefore the difference between both epochs then becomes 10 seconds.
    assert_eq!(
        (pre_ls_utc + 1 * Unit::Day) - (pre_ls_tai + 1 * Unit::Day),
        10 * Unit::Second
    );
    // Of course this works the same way the other way around
    let post_ls_utc = pre_ls_utc + Unit::Day;
    let post_ls_tai = pre_ls_tai + Unit::Day;
    assert_eq!(
        (post_ls_utc - Unit::Day) - (post_ls_tai - Unit::Day),
        Duration::ZERO
    );
}

#[test]
fn test_add_f64_seconds() {
    let e = Epoch::from_gregorian_tai(2044, 6, 6, 12, 18, 54, 0);
    assert_eq!(e + 159 * Unit::Second, e + 159.0);
}

#[test]
#[should_panic]
fn from_infinite_tai_seconds() {
    let _ = Epoch::from_tai_seconds(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_tai_days() {
    let _ = Epoch::from_tai_days(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_mjd_tai_days() {
    let _ = Epoch::from_mjd_tai(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_mjd_utc_days() {
    let _ = Epoch::from_mjd_utc(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_jde_tai_days() {
    let _ = Epoch::from_jde_tai(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_jde_et_days() {
    let _ = Epoch::from_jde_et(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_jde_tdb_days() {
    let _ = Epoch::from_jde_tdb(f64::NAN);
}

#[test]
#[should_panic]
fn from_infinite_tdb_seconds() {
    let _ = Epoch::from_tdb_seconds(f64::NAN);
}

#[test]
fn test_minmax() {
    use hifitime::Epoch;

    let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
    let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);

    assert_eq!(e0, e1.min(e0));
    assert_eq!(e0, e0.min(e1));

    assert_eq!(e1, e1.max(e0));
    assert_eq!(e1, e0.max(e1));
}

#[test]
fn test_weekday() {
    // Ensure that even when we switch the time scale of the underlying Epoch, we're still correctly computing the weekday.
    let permutate_time_scale = |e: Epoch, expect: Weekday| {
        for new_time_scale in [
            TimeScale::BDT,
            TimeScale::ET,
            TimeScale::GPST,
            TimeScale::GST,
            TimeScale::TAI,
            TimeScale::TDB,
            TimeScale::TT,
            TimeScale::UTC,
        ] {
            let e_ts = e.in_time_scale(new_time_scale);
            assert_eq!(e_ts.weekday(), expect, "error with {new_time_scale}");
        }
    };
    // J1900 was a monday
    let j1900 = J1900_REF_EPOCH;
    assert_eq!(j1900.weekday(), Weekday::Monday);
    permutate_time_scale(j1900, Weekday::Monday);
    // 1 nanosec into TAI: still a monday
    let j1900_1ns = Epoch::from_gregorian_tai(1900, 01, 01, 0, 0, 0, 1);
    assert_eq!(j1900_1ns.weekday(), Weekday::Monday);
    permutate_time_scale(j1900_1ns, Weekday::Monday);
    // some portion of that day: still a mon day
    let j1900_10h_123_ns = Epoch::from_gregorian_tai(1900, 01, 01, 10, 00, 00, 123);
    assert_eq!(j1900_10h_123_ns.weekday(), Weekday::Monday);
    permutate_time_scale(j1900_10h_123_ns, Weekday::Monday);
    // Day +1: tuesday
    let j1901 = j1900 + Duration::from_days(1.0);
    assert_eq!(j1901.weekday(), Weekday::Tuesday);
    permutate_time_scale(j1901, Weekday::Tuesday);
    // 1 ns into tuesday, still a tuesday
    let j1901 = j1901 + Duration::from_nanoseconds(1.0);
    assert_eq!(j1901.weekday(), Weekday::Tuesday);
    permutate_time_scale(j1901, Weekday::Tuesday);
    // 6 days into TAI was a sunday
    let e = j1900 + Duration::from_days(6.0);
    assert_eq!(e.weekday(), Weekday::Sunday);
    permutate_time_scale(e, Weekday::Sunday);
    // 6 days + some tiny offset, still a sunday
    let e = e + Duration::from_nanoseconds(10000.0);
    assert_eq!(e.weekday(), Weekday::Sunday);
    permutate_time_scale(e, Weekday::Sunday);
    // 7 days into TAI: back to a monday
    let e = j1900 + Duration::from_days(7.0);
    assert_eq!(e.weekday(), Weekday::Monday);
    permutate_time_scale(e, Weekday::Monday);
    // 2022/12/01 was a thursday
    let epoch = Epoch::from_gregorian_utc_at_midnight(2022, 12, 01);
    assert_eq!(epoch.weekday_utc(), Weekday::Thursday);
    permutate_time_scale(epoch, Weekday::Thursday);
    // 2022/11/28 was a monday
    let epoch = Epoch::from_gregorian_utc_at_midnight(2022, 11, 28);
    assert_eq!(epoch.weekday_utc(), Weekday::Monday);
    permutate_time_scale(epoch, Weekday::Monday);
    // 1988/01/02 was a Saturday
    let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    assert_eq!(epoch.weekday_utc(), Weekday::Saturday);
    permutate_time_scale(epoch, Weekday::Saturday);

    let epoch_tai = BDT_REF_EPOCH;
    assert_eq!(epoch_tai.weekday(), Weekday::Sunday);

    let epoch_tai = GST_REF_EPOCH;
    assert_eq!(epoch_tai.weekday(), Weekday::Sunday);
}

#[test]
fn test_get_time() {
    let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    assert_eq!(epoch.hours(), 10);
    assert_eq!(epoch.minutes(), 11);
    assert_eq!(epoch.seconds(), 12);
    assert_eq!(epoch.milliseconds(), 0);
    assert_eq!(epoch.microseconds(), 0);
    assert_eq!(epoch.nanoseconds(), 13);

    let epoch_midnight = epoch.with_hms(0, 0, 0);
    assert_eq!(
        epoch_midnight,
        Epoch::from_gregorian_utc_at_midnight(2022, 12, 01) + 13 * Unit::Nanosecond
    );

    let epoch_midnight = epoch.with_hms_strict(0, 0, 0);
    assert_eq!(
        epoch_midnight,
        Epoch::from_gregorian_utc_at_midnight(2022, 12, 01)
    );

    let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    let other = other_utc.in_time_scale(TimeScale::TDB);

    assert_eq!(
        epoch.with_hms_from(other),
        Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 13)
    );

    assert_eq!(
        epoch.with_hms_strict_from(other),
        Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 0)
    );

    assert_eq!(
        epoch.with_time_from(other),
        Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 23)
    );
}

#[test]
fn test_start_of_week() {
    // 2022/12/01 + some offset, was a thursday
    let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    assert_eq!(epoch.weekday_utc(), Weekday::Thursday);
    // 2022/11/27 was the related sunday / start of week
    assert_eq!(
        epoch.previous_weekday_at_midnight(Weekday::Sunday),
        Epoch::from_gregorian_utc_at_midnight(2022, 11, 27)
    );
    assert_eq!(
        epoch
            .previous_weekday_at_midnight(Weekday::Sunday)
            .weekday_utc(),
        Weekday::Sunday
    );

    let epoch = Epoch::from_gregorian_utc(2022, 09, 15, 01, 01, 01, 01);
    assert_eq!(epoch.weekday_utc(), Weekday::Thursday);
    assert_eq!(
        epoch.previous_weekday_at_midnight(Weekday::Sunday),
        Epoch::from_gregorian_utc_at_midnight(2022, 09, 11)
    );
    assert_eq!(
        epoch
            .previous_weekday_at_midnight(Weekday::Sunday)
            .weekday_utc(),
        Weekday::Sunday
    );
}

#[test]
fn test_time_of_week() {
    // TAI
    // 0W + 10s + 10ns into TAI
    let epoch = Epoch::from_time_of_week(0, 10 * 1_000_000_000 + 10, TimeScale::TAI);
    assert_eq!(epoch.to_gregorian_utc(), (1900, 01, 01, 00, 00, 10, 10));
    assert_eq!(epoch.to_time_of_week(), (0, 10 * 1_000_000_000 + 10));

    // TAI<=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // 1W + 10s + 10ns into TAI
    let epoch = Epoch::from_time_of_week(1, 10 * 1_000_000_000 + 10, TimeScale::TAI);
    assert_eq!(epoch.to_gregorian_utc(), (1900, 01, 08, 00, 00, 10, 10));

    // GPST
    // https://www.labsat.co.uk/index.php/en/gps-time-calculator
    // 01/12/2022 00:00:00 <=> (2238, 345_618_000_000_000)
    //      2238 weeks since 1980 + 345_600_000_000_000 ns since previous Sunday
    //                            +      18_000_000_000 ns for elapsed leap seconds
    let epoch = Epoch::from_time_of_week(2238, 345_618_000_000_000, TimeScale::GPST);
    assert_eq!(epoch.to_gregorian_utc(), (2022, 12, 01, 00, 00, 00, 00));
    assert_eq!(epoch.to_time_of_week(), (2238, 345_618_000_000_000));

    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (utc_wk, utc_tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(utc_wk, utc_tow, TimeScale::UTC),
        epoch_utc
    );

    // 06/01/1980 01:00:00 = 1H into GPST <=> (0, 3_618_000_000_000)
    let epoch = Epoch::from_time_of_week(0, 3_618_000_000_000, TimeScale::GPST);
    assert_eq!(epoch.to_gregorian_utc(), (1980, 01, 06, 01, 00, 0 + 18, 00));
    assert_eq!(epoch.to_time_of_week(), (0, 3_618_000_000_000));

    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (utc_wk, utc_tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(utc_wk, utc_tow, TimeScale::UTC),
        epoch_utc
    );

    // 01/01/1981 01:00:00 = 51W + 1 hour into GPS epoch <=> 51, 349_218_000_000_000
    let epoch = Epoch::from_time_of_week(51, 349_218_000_000_000, TimeScale::GPST);
    assert_eq!(epoch.to_gregorian_utc(), (1981, 01, 01, 01, 00, 18, 00));
    assert_eq!(epoch.to_time_of_week(), (51, 349_218_000_000_000));

    // 06/25/1980 13:07:19 = 24W + 13:07:19 into GPS epoch <=> 24, 306_457_000_000_000
    let epoch = Epoch::from_time_of_week(24, 306_457_000_000_000, TimeScale::GPST);
    assert_eq!(
        epoch.to_gregorian_utc(),
        (1980, 06, 25, 13, 07, 18 + 19, 00)
    );
    assert_eq!(epoch.to_time_of_week(), (24, 306_457_000_000_000));

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // add 1 nanos
    let epoch = Epoch::from_time_of_week(2238, 345_618_000_000_001, TimeScale::GPST);
    assert_eq!(epoch.to_gregorian_utc(), (2022, 12, 01, 00, 00, 00, 01));

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // add 1/2 day
    let epoch = Epoch::from_time_of_week(2238, 475_218_000_000_000, TimeScale::GPST);
    assert_eq!(epoch.to_gregorian_utc(), (2022, 12, 02, 12, 00, 00, 00));

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // add 1/2 day + 3 hours + 27 min + 19s +10ns
    let epoch = Epoch::from_time_of_week(2238, 487_657_000_000_010, TimeScale::GPST);
    assert_eq!(epoch.to_gregorian_utc(), (2022, 12, 02, 15, 27, 19, 10));

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // 1H into Galileo timescale
    let epoch = Epoch::from_time_of_week(0, 3_600_000_000_000, TimeScale::GST);
    let expected_tai = TimeScale::GST.ref_epoch() + Duration::from_hours(1.0);
    assert_eq!(epoch.to_gregorian_utc(), expected_tai.to_gregorian_utc());
    assert_eq!(epoch.to_time_of_week(), (0, 3_600_000_000_000));

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // 1W + 128H into Galileo timescale
    let epoch = Epoch::from_time_of_week(1, 128 * 3600 * 1_000_000_000, TimeScale::GST);
    let expected_tai =
        TimeScale::GST.ref_epoch() + Duration::from_days(7.0) + Duration::from_hours(128.0);
    assert_eq!(epoch.to_gregorian_utc(), expected_tai.to_gregorian_utc());
    assert_eq!(epoch.to_time_of_week(), (1, 128 * 3600 * 1_000_000_000));

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // 13.5H into BeiDou timescale
    let epoch = Epoch::from_time_of_week(
        0,
        13 * 3600 * 1_000_000_000 + 1800 * 1_000_000_000,
        TimeScale::BDT,
    );
    let expected_tai = TimeScale::BDT.ref_epoch() + Duration::from_hours(13.5);
    assert_eq!(epoch.to_gregorian_utc(), expected_tai.to_gregorian_utc());
    assert_eq!(
        epoch.to_time_of_week(),
        (0, 13 * 3600 * 1_000_000_000 + 1800 * 1_000_000_000)
    );

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );

    // 10W + 36.25 H into BeiDou Timescale
    let epoch = Epoch::from_time_of_week(
        10,
        36 * 3600 * 1_000_000_000 + 900 * 1_000_000_000,
        TimeScale::BDT,
    );
    let expected_tai =
        TimeScale::BDT.ref_epoch() + Duration::from_days(70.0) + Duration::from_hours(36.25);
    assert_eq!(epoch.to_gregorian_utc(), expected_tai.to_gregorian_utc());
    assert_eq!(
        epoch.to_time_of_week(),
        (10, 36 * 3600 * 1_000_000_000 + 900 * 1_000_000_000)
    );

    // <=>UTC
    let epoch_utc = epoch.in_time_scale(TimeScale::UTC);
    let (week, tow) = epoch_utc.to_time_of_week();
    assert_eq!(
        Epoch::from_time_of_week(week, tow, TimeScale::UTC),
        epoch_utc
    );
}

/// Tests that for a number of epochs covering different leap seconds, creating an Epoch with a given time scale will allow us to retrieve in that same time scale with the same value.
#[test]
fn test_day_of_year() {
    // The general test function used throughout this verification.
    let recip_func = |utc_epoch: Epoch| {
        // Test that we can convert this epoch into another time scale and re-initialize it correctly from that value.
        for ts in &[
            TimeScale::UTC,
            TimeScale::TAI,
            TimeScale::TT,
            TimeScale::ET,
            TimeScale::TDB,
        ] {
            let epoch = utc_epoch.in_time_scale(*ts);
            let (year, days) = epoch.year_days_of_year();
            let rebuilt = Epoch::from_day_of_year(year, days, *ts);
            if *ts == TimeScale::ET || *ts == TimeScale::TDB {
                // There is limitation in the ET scale due to the Newton Raphson iteration.
                // So let's check for a near equality
                assert!(
                    (epoch - rebuilt).abs() < 750 * Unit::Nanosecond,
                    "{} recip error = {} for {}",
                    ts,
                    epoch - rebuilt,
                    epoch
                );
            } else {
                assert!(
                    (epoch - rebuilt).abs() < 50 * Unit::Nanosecond,
                    "{} recip error = {} for {}",
                    ts,
                    epoch - rebuilt,
                    epoch
                );
            }
        }
    };

    recip_func(Epoch::from_gregorian_utc(1900, 1, 9, 0, 17, 15, 0));

    recip_func(Epoch::from_gregorian_utc(1920, 7, 23, 14, 39, 29, 0));

    recip_func(Epoch::from_gregorian_utc(1954, 12, 24, 6, 6, 31, 0));

    // Test prior to official leap seconds but with some scaling, valid from 1960 to 1972 according to IAU SOFA.
    recip_func(Epoch::from_gregorian_utc(1960, 2, 14, 6, 6, 31, 0));

    // First test with some leap seconds
    recip_func(Epoch::from_gregorian_utc(
        1983,
        4,
        13,
        12,
        9,
        14,
        274_000_000,
    ));

    // Once every 400 years, there is a leap day on the new century! Joyeux anniversaire, Papa!
    recip_func(Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 0));

    recip_func(Epoch::from_gregorian_utc(
        2022,
        11,
        29,
        7,
        58,
        49,
        782_000_000,
    ));

    recip_func(Epoch::from_gregorian_utc(2044, 6, 6, 12, 18, 54, 0));

    recip_func(Epoch::from_gregorian_utc(2075, 4, 30, 23, 59, 54, 0));
}

/// Tests that for a number of epochs covering different leap seconds, creating an Epoch with a given time scale will allow us to retrieve in that same time scale with the same value.
#[test]
fn test_epoch_formatter() {
    use core::str::FromStr;
    use hifitime::efmt::consts::*;

    let bday = Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 37);

    let fmt_iso_ord = Formatter::new(bday, ISO8601_ORDINAL);
    assert_eq!(format!("{fmt_iso_ord}"), "2000-059");

    let fmt_iso_ord = Formatter::new(bday, Format::from_str("%j").unwrap());
    assert_eq!(format!("{fmt_iso_ord}"), "059");

    let fmt_iso = Formatter::new(bday, ISO8601);
    assert_eq!(format!("{fmt_iso}"), format!("{bday}"));

    let fmt = Formatter::new(bday, ISO8601_DATE);
    assert_eq!(format!("{fmt}"), format!("2000-02-29"));

    let fmt = Formatter::new(bday, RFC2822);
    assert_eq!(format!("{fmt}"), format!("Tue, 29 Feb 2000 14:57:29"));

    let fmt = Formatter::new(bday, RFC2822_LONG);
    assert_eq!(
        format!("{fmt}"),
        format!("Tuesday, 29 February 2000 14:57:29")
    );

    // Decimal week day starts counting at zero ... it's dumb.
    let fmt = Formatter::new(bday, Format::from_str("%w").unwrap());
    assert_eq!(format!("{fmt}"), format!("2"));

    let init_str = "1994-11-05T08:15:30-05:00";
    let e = Epoch::from_str(init_str).unwrap();

    let fmt = Format::from_str("%Y-%m-%dT%H:%M:%S.%f%z").unwrap();
    assert_eq!(fmt, RFC3339);

    let fmtd = Formatter::with_timezone(e, Duration::from_str("-05:00").unwrap(), RFC3339_FLEX);

    assert_eq!(init_str, format!("{fmtd}"));

    assert_eq!(
        format!("{:?}", Format::from_str("%A, ").unwrap()),
        "EpochFormat:`Weekday, `"
    );
    assert_eq!(
        format!("{:?}", Format::from_str("%A,?").unwrap()),
        "EpochFormat:`Weekday,?`"
    );

    // Test an invalid token
    assert_eq!(
        Format::from_str("%p"),
        Err(hifitime::ParsingErrors::UnknownFormattingToken('p'))
    );
}

#[cfg(feature = "std")]
#[test]
fn test_leap_seconds_file() {
    use hifitime::leap_seconds::{LatestLeapSeconds, LeapSecondsFile};

    let provider = LeapSecondsFile::from_path("data/leap-seconds.list").unwrap();

    let default = LatestLeapSeconds::default();

    // Check that we read the data correctly knowing that the IERS data only contains the announced leap seconds.
    let mut pos = 0;
    for expected in default {
        if expected.announced_by_iers {
            assert_eq!(expected, provider[pos]);
            pos += 1;
        }
    }
}

#[test]
fn regression_test_gh_204() {
    use core::str::FromStr;
    use hifitime::Epoch;

    let e1700 = Epoch::from_str("1700-01-01T00:00:00 TAI").unwrap();
    assert_eq!(format!("{e1700:x}"), "1700-01-01T00:00:00 TAI");

    let e1700 = Epoch::from_str("1700-04-17T02:10:09 TAI").unwrap();
    assert_eq!(format!("{e1700:x}"), "1700-04-17T02:10:09 TAI");

    let e1799 = Epoch::from_str("1799-01-01T00:00:01 TAI").unwrap();
    assert_eq!(format!("{e1799:x}"), "1799-01-01T00:00:01 TAI");

    let e1899 = Epoch::from_str("1899-01-01T00:00:00 TAI").unwrap();
    assert_eq!(format!("{e1899:x}"), "1899-01-01T00:00:00 TAI");

    let e1900_m1 = Epoch::from_str("1899-12-31T23:59:59 TAI").unwrap();
    assert_eq!(format!("{e1900_m1:x}"), "1899-12-31T23:59:59 TAI");
}

#[test]
fn regression_test_gh_261() {
    let mut test: Epoch;
    println!("Dates and JDE for year 1607");
    for i in 25..=31 {
        test = Epoch::from_gregorian_utc(1606, 12, i, 0, 0, 0, 0);
        println!("{} - {}", test, test.to_jde_utc_days());
        assert_eq!(test.year_days_of_year().0, 1606);
    }
    for i in 25..=31 {
        test = Epoch::from_gregorian_utc(1607, 12, i, 0, 0, 0, 0);
        println!("{} - {}", test, test.to_jde_utc_days());
    }
    println!("Dates and JDE for year 1608");
    for i in 1..=5 {
        test = Epoch::from_gregorian_utc(1608, 1, i, 0, 0, 0, 0);
        println!("{} - {}", test, test.to_jde_utc_days());
    }
    println!("Dates and JDE for year 2023");
    for i in 1..=5 {
        test = Epoch::from_gregorian_utc(2023, 1, i, 0, 0, 0, 0);
        println!("{} - {}", test, test.to_jde_utc_days());
    }
}
