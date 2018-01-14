extern crate hifitime;

#[test]
fn reciprocity() {
    use hifitime::instant;
    use hifitime::TimeSystem;
    use hifitime::julian::ModifiedJulian;

    // Check reciprocity in the Present
    let tick = instant::Instant::new(159, 10, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(tick);
    assert_eq!(mjd.into_instant(), tick);

    // Check reciprocity in the Past
    let tick = instant::Instant::new(159, 10, instant::Era::Past);
    let mjd = ModifiedJulian::from_instant(tick);
    assert_eq!(mjd.into_instant(), tick);
}

#[test]
fn epochs() {
    use std;
    use hifitime::instant;
    use hifitime::TimeSystem;
    use hifitime::julian::{ModifiedJulian, SECONDS_PER_DAY};
    use hifitime::datetime::Datetime;

    // Tests are chronological dates.
    // All of the following examples are cross validated against NASA HEASARC,
    // refered to as "X-Val" for "cross validation."

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let nist_j1900 = instant::Instant::new(0, 0, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(nist_j1900);
    assert!((mjd.days - 15_020.0).abs() < std::f64::EPSILON);
    assert!((mjd.julian_days() - 2_415_020.5).abs() < std::f64::EPSILON);
    assert!(
        (ModifiedJulian::from_instant(
            Datetime::new(1900, 1, 1, 0, 0, 0, 0)
                .expect("01 January 1900 invalid?!")
                .into_instant(),
        ).days - 15_020.0)
            .abs() < std::f64::EPSILON
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-01+12%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j1900 = instant::Instant::new((SECONDS_PER_DAY * 0.5) as u64, 0, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(j1900);
    assert!((mjd.days - 15_020.5).abs() < std::f64::EPSILON);
    assert!((mjd.julian_days() - 2_415_021.0).abs() < std::f64::EPSILON);
    assert!(
        (ModifiedJulian::from_instant(
            Datetime::new(1900, 1, 1, 12, 0, 0, 0)
                .expect("01 January 1900 invalid?!")
                .into_instant(),
        ).days - 15_020.5)
            .abs() < std::f64::EPSILON
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1900-01-08+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let mjd = ModifiedJulian::from_instant(
        Datetime::new(1900, 1, 8, 00, 0, 0, 0)
            .expect("08 January 1900 invalid?!")
            .into_instant(),
    );
    assert!((mjd.days - 15_027.0).abs() < std::f64::EPSILON);
    assert!((mjd.julian_days() - 2_415_027.5).abs() < std::f64::EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=1980-01-06+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let gps_std_epoch = ModifiedJulian::from_instant(
        Datetime::new(1980, 1, 6, 0, 0, 0, 0)
            .expect("06 January 1980 invalid?!")
            .into_instant(),
    );
    assert!((gps_std_epoch.days - 44244.0).abs() < std::f64::EPSILON);
    assert!((gps_std_epoch.julian_days() - 2_444_244.5).abs() < std::f64::EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2000-01-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let j2000 = Datetime::new(2000, 1, 1, 0, 0, 0, 0)
        .expect("01 January 2000 invalid?!")
        .into_instant();
    let mjd = ModifiedJulian::from_instant(j2000);
    assert!((mjd.days - 51_544.0).abs() < std::f64::EPSILON);
    assert!((mjd.julian_days() - 2_451_544.5).abs() < std::f64::EPSILON);

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2002-02-07+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    let jd020207 = ModifiedJulian::from_instant(
        Datetime::new(2002, 2, 7, 0, 0, 0, 0)
            .expect("7 February 2002 invalid?!")
            .into_instant(),
    );
    assert!((jd020207.days - 52_312.0).abs() < std::f64::EPSILON);
    assert!((jd020207.julian_days() - 2_452_312.5).abs() < std::f64::EPSILON);

    // Test leap seconds and Julian at the same time
    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    // NOTE: Precision of HEASARC is less than hifitime, hence the last four digit difference
    // HEASARC reports 57203.99998843 but hifitime computes 57203.99998842592 (three additional)
    // significant digits.
    assert!(
        (ModifiedJulian::from_instant(
            Datetime::new(2015, 6, 30, 23, 59, 59, 0)
                .expect("July leap second failed")
                .into_instant(),
        ).days - 57_203.99998842592)
            .abs() < std::f64::EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (ModifiedJulian::from_instant(
            Datetime::new(2015, 6, 30, 23, 59, 60, 0)
                .expect("July leap second failed")
                .into_instant(),
        ).days - 57_203.99998842592)
            .abs() < std::f64::EPSILON,
        "Incorrect July 2015 leap second MJD computed"
    );

    // X-Val: https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes
    assert!(
        (ModifiedJulian::from_instant(
            Datetime::new(2015, 7, 1, 0, 0, 0, 0)
                .expect("Post July leap second failed")
                .into_instant(),
        ).days - 57_204.0)
            .abs() < std::f64::EPSILON,
        "Incorrect Post July 2015 leap second MJD computed"
    );
}
