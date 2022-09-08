#[cfg(feature = "std")]
extern crate core;
extern crate hifitime;

use hifitime::{
    is_gregorian_valid, Duration, Epoch, TimeSystem, Unit, DAYS_GPS_TAI_OFFSET, J1900_OFFSET,
    J2000_OFFSET, MJD_OFFSET, SECONDS_GPS_TAI_OFFSET, SECONDS_PER_DAY,
};

#[cfg(feature = "std")]
use core::f64::EPSILON;
#[cfg(not(feature = "std"))]
use std::f64::EPSILON;

#[test]
fn test_const_ops() {
    // Tests that multiplying a constant with a unit returns the correct number in that same unit
    let mjd_offset = MJD_OFFSET * Unit::Day;
    assert!((mjd_offset.in_unit(Unit::Day) - MJD_OFFSET).abs() < f64::EPSILON);
    let j2000_offset = J2000_OFFSET * Unit::Day;
    assert!((j2000_offset.in_unit(Unit::Day) - J2000_OFFSET).abs() < f64::EPSILON);
}

#[allow(clippy::float_equality_without_abs)]
#[test]
fn utc_epochs() {
    assert!(Epoch::from_mjd_tai(J1900_OFFSET).as_tai_seconds() < EPSILON);
    assert!((Epoch::from_mjd_tai(J1900_OFFSET).as_mjd_tai_days() - J1900_OFFSET).abs() < EPSILON);

    // Tests are chronological dates.
    // All of the following examples are cross validated against NASA HEASARC,
    // refered to as "X-Val" for "cross validation."

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
    assert!(epoch_from_tai_secs.as_tai_seconds() - 2_272_060_800.0 < EPSILON);
    let epoch_from_tai_greg = Epoch::from_tai_seconds(2_272_060_800.0);
    assert_eq!(epoch_from_tai_greg, epoch_from_tai_secs, "Incorrect epoch");

    // Check that second leap second happens
    let epoch_from_utc_greg = Epoch::from_gregorian_utc_hms(1972, 6, 30, 23, 59, 59);
    let epoch_from_utc_greg1 = Epoch::from_gregorian_utc_hms(1972, 7, 1, 0, 0, 0);
    assert!(
        (epoch_from_utc_greg1.as_tai_seconds() - epoch_from_utc_greg.as_tai_seconds() - 2.0).abs()
            < EPSILON
    );

    // Just prior to the 2017 leap second, there should be an offset of 36 seconds between UTC and TAI
    let this_epoch = Epoch::from_tai_seconds(3_692_217_599.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 23);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    assert!(this_epoch.as_tai_seconds() - epoch_utc.as_utc_seconds() - 36.0 < EPSILON);

    // Just after to the 2017 leap second, there should be an offset of 37 seconds between UTC and TAI
    let this_epoch = Epoch::from_tai_seconds(3_692_217_600.0);
    let epoch_utc = Epoch::from_gregorian_utc_hms(2016, 12, 31, 23, 59, 24);
    assert_eq!(epoch_utc, this_epoch, "Incorrect epoch");
    assert!(this_epoch.as_tai_seconds() - epoch_utc.as_utc_seconds() - 37.0 < EPSILON);

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

    let this_epoch = Epoch::from_gregorian_tai_at_midnight(2020, 1, 1);
    assert!((this_epoch.as_jde_tai_days() - 2_458_849.5).abs() < EPSILON)
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
    assert!(
        flp_from_secs_tai.as_tai_seconds() > flp_from_secs_tai.as_utc_seconds(),
        "TAI is not ahead of UTC (via function call)"
    );
    assert!(
        (flp_from_secs_tai.as_tai_seconds() - flp_from_secs_tai.as_utc_seconds() - 10.0) < EPSILON,
        "TAI is not ahead of UTC"
    );

    // Check that all of the TAI/UTC time differences are of 37.0 as of today.
    let epoch_utc = Epoch::from_gregorian_utc_hms(2019, 8, 1, 20, 10, 23);
    let epoch_tai = Epoch::from_gregorian_tai_hms(2019, 8, 1, 20, 10, 23);
    assert!(epoch_tai < epoch_utc, "TAI is not ahead of UTC");
    let delta: Duration = epoch_utc - epoch_tai - Unit::Second * 37.0;
    assert!(delta < Unit::Nanosecond, "TAI is not ahead of UTC");
    assert!(
        (epoch_utc.as_tai_seconds() - epoch_tai.as_tai_seconds() - 37.0).abs() < EPSILON,
        "TAI is not ahead of UTC"
    );
    assert!(
        (epoch_utc.as_utc_seconds() - epoch_tai.as_utc_seconds() - 37.0).abs() < EPSILON,
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
}

#[test]
fn julian_epoch() {
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let nist_j1900 = Epoch::from_tai_days(0.0);
    assert!((nist_j1900.as_mjd_tai_days() - 15_020.0).abs() < EPSILON);
    assert!((nist_j1900.as_jde_tai_days() - 2_415_020.5).abs() < EPSILON);
    let mjd = Epoch::from_gregorian_utc_at_midnight(1900, 1, 1);
    assert!((mjd.as_mjd_tai_days() - 15_020.0).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+12%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j1900 = Epoch::from_tai_days(0.5);
    assert!((j1900.as_mjd_tai_days() - 15_020.5).abs() < EPSILON);
    assert!((j1900.as_jde_tai_days() - 2_415_021.0).abs() < EPSILON);
    let mjd = Epoch::from_gregorian_utc_at_noon(1900, 1, 1);
    assert!((mjd.as_mjd_tai_days() - 15_020.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-08+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let mjd = Epoch::from_gregorian_utc_at_midnight(1900, 1, 8);
    assert!((mjd.as_mjd_tai_days() - 15_027.0).abs() < EPSILON);
    assert!((mjd.as_jde_tai_days() - 2_415_027.5).abs() < EPSILON);
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1980-01-06+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let gps_std_epoch = Epoch::from_gregorian_tai_at_midnight(1980, 1, 6);
    assert!((gps_std_epoch.as_mjd_tai_days() - 44_244.0).abs() < EPSILON);
    assert!((gps_std_epoch.as_jde_tai_days() - 2_444_244.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2000-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j2000 = Epoch::from_gregorian_tai_at_midnight(2000, 1, 1);
    assert!((j2000.as_mjd_tai_days() - 51_544.0).abs() < EPSILON);
    assert!((j2000.as_jde_tai_days() - 2_451_544.5).abs() < EPSILON);

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
    assert!((j2000.as_mjd_utc_days() - 51_544.0).abs() < EPSILON);
    assert!((j2000.as_jde_utc_days() - 2_451_544.5).abs() < EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2002-02-07+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let jd020207 = Epoch::from_gregorian_tai_at_midnight(2002, 2, 7);
    assert!((jd020207.as_mjd_tai_days() - 52_312.0).abs() < EPSILON);
    assert!((jd020207.as_jde_tai_days() - 2_452_312.5).abs() < EPSILON);

    // Test leap seconds and Julian at the same time
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // NOTE: Precision of HEASARC is less than hifitime, hence the last four digit difference
    // HEASARC reports 57203.99998843 but hifitime computes 57203.99998842592 (three additional)
    // significant digits.
    assert!(
        (Epoch::from_gregorian_tai_hms(2015, 6, 30, 23, 59, 59).as_mjd_tai_days()
            - 57_203.999_988_425_92)
            .abs()
            < EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (Epoch::from_gregorian_tai_hms(2015, 6, 30, 23, 59, 60).as_mjd_tai_days()
            - 57_203.999_988_425_92)
            .abs()
            < EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (Epoch::from_gregorian_tai_at_midnight(2015, 7, 1).as_mjd_tai_days() - 57_204.0).abs()
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
    let now = Epoch::from_gregorian_tai_hms(2019, 8, 24, 3, 49, 9);
    assert!(
        now.as_tai_seconds() > now.as_utc_seconds(),
        "TAI is not ahead of UTC"
    );
    assert!((now.as_tai_seconds() - now.as_utc_seconds() - 37.0).abs() < EPSILON);
    assert!(
        now.as_tai_seconds() > now.as_gpst_seconds(),
        "TAI is not ahead of GPS Time"
    );
    assert_eq!(
        Epoch::from_gpst_nanoseconds(now.as_gpst_nanoseconds().unwrap()),
        now,
        "To/from GPST nanoseconds failed"
    );
    assert!(
        (now.as_tai_seconds() - SECONDS_GPS_TAI_OFFSET - now.as_gpst_seconds()).abs() < EPSILON
    );
    assert!(
        now.as_gpst_seconds() + SECONDS_GPS_TAI_OFFSET > now.as_utc_seconds(),
        "GPS Time is not ahead of UTC"
    );

    let gps_epoch = Epoch::from_tai_seconds(SECONDS_GPS_TAI_OFFSET);
    #[cfg(feature = "std")]
    {
        assert_eq!(
            gps_epoch.as_gregorian_str(TimeSystem::UTC),
            "1980-01-06T00:00:00 UTC"
        );
        assert_eq!(
            gps_epoch.as_gregorian_str(TimeSystem::TAI),
            "1980-01-06T00:00:19 TAI"
        );
        assert_eq!(format!("{:o}", gps_epoch), "0");
    }
    assert_eq!(
        gps_epoch.as_tai_seconds(),
        Epoch::from_gregorian_utc_at_midnight(1980, 1, 6).as_tai_seconds()
    );
    assert!(
        gps_epoch.as_gpst_seconds().abs() < EPSILON,
        "The number of seconds from the GPS epoch was not 0: {}",
        gps_epoch.as_gpst_seconds()
    );
    assert!(
        gps_epoch.as_gpst_days().abs() < EPSILON,
        "The number of days from the GPS epoch was not 0: {}",
        gps_epoch.as_gpst_days()
    );

    let epoch = Epoch::from_gregorian_utc_at_midnight(1972, 1, 1);
    assert!(
        (epoch.as_tai_seconds() - SECONDS_GPS_TAI_OFFSET - epoch.as_gpst_seconds()).abs() < EPSILON
    );
    assert!((epoch.as_tai_days() - DAYS_GPS_TAI_OFFSET - epoch.as_gpst_days()).abs() < 1e-11);

    // 1 Jan 1980 is 5 days before the GPS epoch.
    let epoch = Epoch::from_gregorian_utc_at_midnight(1980, 1, 1);
    assert!((epoch.as_gpst_seconds() + 5.0 * SECONDS_PER_DAY).abs() < EPSILON);
    assert!((epoch.as_gpst_days() + 5.0).abs() < EPSILON);
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

        let epoch_unix_time_s = Epoch::now().unwrap().as_unix_seconds();

        assert!(
            (std_unix_time_s - epoch_unix_time_s).abs() < 1e-5,
            "hifitime and std differ in UNIX by more than 10 microseconds: hifitime = {}\tstd = {}",
            epoch_unix_time_s,
            std_unix_time_s
        );
    }

    let now = Epoch::from_gregorian_utc_hms(2022, 5, 2, 10, 39, 15);
    assert!((now.as_unix_seconds() - 1651487955.0_f64).abs() < EPSILON);
    assert!((now.as_unix_milliseconds() - 1651487955000.0_f64).abs() < EPSILON);
    assert_eq!(
        Epoch::from_unix_seconds(now.as_unix_seconds()),
        now,
        "To/from UNIX seconds failed"
    );
    assert_eq!(
        Epoch::from_unix_milliseconds(now.as_unix_milliseconds()),
        now,
        "To/from UNIX milliseconds failed"
    );

    let unix_epoch = Epoch::from_gregorian_utc_at_midnight(1970, 1, 1);
    #[cfg(feature = "std")]
    {
        assert_eq!(
            unix_epoch.as_gregorian_str(TimeSystem::UTC),
            "1970-01-01T00:00:00 UTC"
        );
        assert_eq!(
            unix_epoch.as_gregorian_str(TimeSystem::TAI),
            "1970-01-01T00:00:00 TAI"
        );
        // Print as UNIX seconds
        assert_eq!(format!("{:p}", unix_epoch), "0");
    }
    assert_eq!(
        unix_epoch.as_tai_seconds(),
        Epoch::from_gregorian_utc_at_midnight(1970, 1, 1).as_tai_seconds()
    );
    assert!(
        unix_epoch.as_unix_seconds().abs() < EPSILON,
        "The number of seconds from the UNIX epoch was not 0: {}",
        unix_epoch.as_unix_seconds()
    );
    assert!(
        unix_epoch.as_unix_milliseconds().abs() < EPSILON,
        "The number of milliseconds from the UNIX epoch was not 0: {}",
        unix_epoch.as_unix_seconds()
    );
    assert!(
        unix_epoch.as_unix_days().abs() < EPSILON,
        "The number of days from the UNIX epoch was not 0: {}",
        unix_epoch.as_unix_days()
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
            (Epoch::from_et_seconds(et_s).as_et_seconds() - et_s).abs() < recip_err_s,
            "{} failed ET reciprocity test:\nwant: {}\tgot: {}\nerror: {} ns",
            epoch,
            et_s,
            Epoch::from_et_seconds(et_s).as_et_seconds(),
            (et_s - Epoch::from_et_seconds(et_s).as_et_seconds()).abs() * 1e9
        );
        assert!(
            (Epoch::from_tdb_seconds(et_s).as_tdb_seconds() - et_s).abs() < recip_err_s,
            "{} failed TDB reciprocity test:\nwant: {}\tgot: {}\nerror: {} ns",
            epoch,
            et_s,
            Epoch::from_tdb_seconds(et_s).as_tdb_seconds(),
            (et_s - Epoch::from_tdb_seconds(et_s).as_et_seconds()).abs() * 1e9
        );

        // Test ET computation
        let extra_seconds = if epoch.leap_seconds_iers() == 0 {
            spice_utc_tai_ls_err
        } else {
            0.0
        };
        assert!(
            (epoch.as_et_seconds() - et_s + extra_seconds).abs() < EPSILON,
            "{} failed ET test",
            epoch
        );

        // Test TDB computation
        assert!(
            (epoch.as_tdb_duration() - et_s * Unit::Second + extra_seconds * Unit::Second).abs()
                <= max_tdb_et_err,
            "{} failed TDB test",
            epoch
        );

        // TEST JDE computation
        assert!(
            (epoch.as_jde_utc_days() - utc_jde_d).abs() < spice_jde_precision,
            "{} failed JDE UTC days test:\nwant: {}\tgot: {}\nerror = {} days",
            epoch,
            utc_jde_d,
            epoch.as_jde_utc_days(),
            (epoch.as_jde_utc_days() - utc_jde_d).abs()
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
    assert!((from_et_s.as_tdb_seconds() - expected_et_s).abs() < EPSILON);
    // Validate UTC to ET when initialization from UTC
    assert!(dbg!(sp_ex.as_et_seconds() - expected_et_s).abs() < max_prec.in_seconds());
    assert!(dbg!(sp_ex.as_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.in_seconds());
    assert!(dbg!(sp_ex.as_jde_utc_days() - 2455964.9739931).abs() < 1e-7);
    assert!(dbg!(sp_ex.as_tai_seconds() - from_et_s.as_tai_seconds()).abs() < 3e-6);

    // Second example
    let sp_ex = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);
    let expected_et_s = 66_312_064.184_938_76;
    assert!(dbg!(sp_ex.as_et_seconds() - expected_et_s).abs() < max_prec.in_seconds());
    assert!(dbg!(sp_ex.as_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.in_seconds());
    assert!(
        (sp_ex.as_tai_seconds() - Epoch::from_tdb_seconds(expected_et_s).as_tai_seconds()).abs()
            < 1e-5
    );

    // Third example
    let sp_ex = Epoch::from_gregorian_utc_hms(1996, 2, 7, 11, 22, 33);
    let expected_et_s = -123_035_784.815_060_48;
    assert!(dbg!(sp_ex.as_et_seconds() - expected_et_s).abs() < max_prec.in_seconds());
    assert!(dbg!(sp_ex.as_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.in_seconds());
    assert!(
        dbg!(sp_ex.as_tai_seconds() - Epoch::from_tdb_seconds(expected_et_s).as_tai_seconds())
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
    assert!(dbg!(sp_ex.as_et_seconds() - expected_et_s).abs() < max_prec.in_seconds());
    assert!(dbg!(sp_ex.as_tdb_seconds() - expected_et_s).abs() < max_tdb_et_err.in_seconds());
    assert!((sp_ex.as_jde_utc_days() - 2457060.9739931).abs() < 1e-7);

    // JDE TDB tests
    /* Initial JDE from sp.et2utc:
    >>> sp.str2et("JD 2452312.500372511 TDB")
    66312032.18493909
    */
    // 2002-02-07T00:00:00.4291 TAI
    let sp_ex = Epoch::from_tdb_seconds(66_312_032.184_939_09);
    assert!((2452312.500372511 - sp_ex.as_jde_et_days()).abs() < EPSILON);
    assert!((2452312.500372511 - sp_ex.as_jde_tdb_days()).abs() < EPSILON);
    // Confirm that they are _not_ equal, only that the number of days in f64 is equal
    assert_ne!(sp_ex.as_jde_et_duration(), sp_ex.as_jde_tdb_duration());

    // 2012-02-07T11:22:00.818924427 TAI
    let sp_ex = Epoch::from_tdb_seconds(381_885_753.003_859_5);
    assert!((2455964.9739931 - sp_ex.as_jde_et_days()).abs() < EPSILON);
    assert!((2455964.9739931 - sp_ex.as_jde_tdb_days()).abs() < max_tdb_et_err.in_seconds());
}

#[cfg(feature = "std")]
#[test]
fn test_from_str() {
    use std::str::FromStr;

    let dt = Epoch::from_gregorian_utc(2017, 1, 14, 0, 31, 55, 0);
    assert_eq!(dt, Epoch::from_str("2017-01-14T00:31:55 UTC").unwrap());
    assert_eq!(dt, Epoch::from_str("2017-01-14T00:31:55").unwrap());
    assert_eq!(dt, Epoch::from_str("2017-01-14 00:31:55").unwrap());
    assert!(Epoch::from_str("2017-01-14 00:31:55 TAI").is_ok());
    assert!(Epoch::from_str("2017-01-14 00:31:55 TT").is_ok());
    assert!(Epoch::from_str("2017-01-14 00:31:55 ET").is_ok());
    assert!(Epoch::from_str("2017-01-14 00:31:55 TDB").is_ok());

    let jde = 2_452_312.500_372_511;
    let as_tdb = Epoch::from_str("JD 2452312.500372511 TDB").unwrap();
    let as_et = Epoch::from_str("JD 2452312.500372511 ET").unwrap();
    let as_tai = Epoch::from_str("JD 2452312.500372511 TAI").unwrap();

    // The JDE only has a precision of 1e-9 days, so we can only compare down to that
    const SPICE_EPSILON: f64 = 1e-9;
    assert!((as_tdb.as_jde_tdb_days() - jde).abs() < SPICE_EPSILON);
    assert!((as_et.as_jde_et_days() - jde).abs() < SPICE_EPSILON);
    assert!((as_tai.as_jde_tai_days() - jde).abs() < SPICE_EPSILON);
    assert!(
        (Epoch::from_str("MJD 51544.5 TAI")
            .unwrap()
            .as_mjd_tai_days()
            - 51544.5)
            .abs()
            < EPSILON
    );
    assert!((Epoch::from_str("SEC 0.5 TAI").unwrap().as_tai_seconds() - 0.5).abs() < EPSILON);

    // Must account for the precision error
    assert!(
        (Epoch::from_str("SEC 66312032.18493909 TDB")
            .unwrap()
            .as_tdb_seconds()
            - 66312032.18493909)
            .abs()
            < 1e-4
    );

    // Check reciprocity of string
    let greg = "2020-01-31T00:00:00 UTC";
    assert_eq!(
        greg,
        Epoch::from_str(greg)
            .unwrap()
            .as_gregorian_str(TimeSystem::UTC)
    );
    let greg = "2020-01-31T00:00:00 TAI";
    assert_eq!(
        greg,
        Epoch::from_str(greg)
            .unwrap()
            .as_gregorian_str(TimeSystem::TAI)
    );
    let greg = "2020-01-31T00:00:00 TDB";
    assert_eq!(
        greg,
        Epoch::from_str(greg)
            .unwrap()
            .as_gregorian_str(TimeSystem::TDB)
    );
    let greg = "2020-01-31T00:00:00 ET";
    assert_eq!(
        greg,
        Epoch::from_str(greg)
            .unwrap()
            .as_gregorian_str(TimeSystem::ET)
    );
}

#[cfg(feature = "std")]
#[test]
fn test_from_str_tdb() {
    use std::str::FromStr;

    let greg = "2020-01-31T00:00:00 TDB";
    assert_eq!(
        greg,
        Epoch::from_str(greg)
            .unwrap()
            .as_gregorian_str(TimeSystem::TDB)
    );
}

#[test]
fn test_format() {
    use hifitime::Epoch;

    let epoch = Epoch::from_gregorian_utc_hms(2022, 9, 6, 23, 24, 29);

    // Check the ET computation once more
    assert!((epoch.as_et_seconds() - 715778738.1825389).abs() < EPSILON);

    assert_eq!(format!("{epoch}"), "2022-09-06T23:24:29 UTC");
    assert_eq!(format!("{epoch:x}"), "2022-09-06T23:25:06 TAI");
    assert_eq!(format!("{epoch:X}"), "2022-09-06T23:25:38.184000015 TT");
    assert_eq!(format!("{epoch:E}"), "2022-09-06T23:25:38.182538986 ET");
    assert_eq!(format!("{epoch:e}"), "2022-09-06T23:25:38.182541370 TDB");
    assert_eq!(format!("{epoch:p}"), "1662506669"); // UNIX seconds
    assert_eq!(format!("{epoch:o}"), "1346541887000000000"); // GPS nanoseconds
}

#[test]
fn ops() {
    // Test adding a second
    let sp_ex: Epoch = Epoch::from_gregorian_utc_hms(2012, 2, 7, 11, 22, 33) + Unit::Second * 1.0;
    let expected_et_s = 381_885_819.184_935_87;
    assert!(dbg!(sp_ex.as_tdb_seconds() - expected_et_s - 1.0).abs() < 2.6e-6);
    let sp_ex: Epoch = sp_ex - Unit::Second * 1.0;
    assert!((sp_ex.as_tdb_seconds() - expected_et_s).abs() < 2.6e-6);
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
        Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 100, TimeSystem::TAI).unwrap();
    let later_epoch =
        Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 200, TimeSystem::TAI).unwrap();

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
    assert_eq!(epoch_from_utc_greg.leap_seconds_iers(), 0);
    // The first leap second is special; it adds 10 seconds.
    assert_eq!(epoch_from_utc_greg1.leap_seconds_iers(), 10);

    // Just before the second leap second.
    let epoch_from_utc_greg = Epoch::from_gregorian_tai_hms(1972, 6, 30, 23, 59, 59);
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
    let (centuries, nanos) = dt.as_tai_duration().to_parts();
    assert_eq!(centuries, 1);
    assert_eq!(nanos, 537582752000000000);
}

#[cfg(feature = "std")]
#[test]
fn test_now() {
    // Simply ensure that this call does not panic
    let now = Epoch::now().unwrap();
    println!("{now}");
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
    let epoch1 = Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 100, TimeSystem::TAI).unwrap();
    let epoch2 = Epoch::maybe_from_gregorian(2020, 1, 8, 16, 1, 17, 200, TimeSystem::TAI).unwrap();

    assert_eq!(epoch1.max(epoch2), epoch2);
    assert_eq!(epoch2.min(epoch1), epoch1);
    assert_eq!(epoch1.cmp(&epoch1), core::cmp::Ordering::Equal);
}
