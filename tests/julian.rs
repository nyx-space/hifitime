extern crate hifitime;

#[test]
fn reciprocity() {
    use hifitime::instant;
    use hifitime::traits::TimeSystem;
    use hifitime::julian::ModifiedJulian;

    // Check reciprocity in the Present
    let tick = instant::Instant::new(159, 10, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(tick);
    assert_eq!(mjd.as_instant(), tick);

    // Check reciprocity in the Past
    let tick = instant::Instant::new(159, 10, instant::Era::Past);
    let mjd = ModifiedJulian::from_instant(tick);
    assert_eq!(mjd.as_instant(), tick);
}

#[test]
fn epochs() {
    use hifitime::instant;
    use hifitime::traits::TimeSystem;
    use hifitime::julian::{ModifiedJulian, SECONDS_PER_DAY};
    use hifitime::utc::Utc;
    use hifitime::traits::TimeZone;

    // Tests are chronological dates.
    // All of the following examples are cross validated against NASA HEASARC,
    // refered to as "X-Val" for "cross validation."

    // X-Val: https://goo.gl/6EW7J3
    let nist_j1900 = instant::Instant::new(0, 0, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(nist_j1900);
    assert_eq!(mjd.days, 15_020.0);
    assert_eq!(mjd.julian_days(), 2_415_020.5);
    assert_eq!(
        ModifiedJulian::from_instant(
            Utc::new(1900, 01, 01, 0, 0, 0, 0)
                .expect("01 January 1900 invalid?!")
                .as_instant(),
        ).days,
        15_020.0
    );

    // X-Val: https://goo.gl/DXRUfh
    let j1900 = instant::Instant::new((SECONDS_PER_DAY * 0.5) as u64, 0, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(j1900);
    assert_eq!(mjd.days, 15_020.5);
    assert_eq!(mjd.julian_days(), 2_415_021.0);
    assert_eq!(
        ModifiedJulian::from_instant(
            Utc::new(1900, 01, 01, 12, 0, 0, 0)
                .expect("01 January 1900 invalid?!")
                .as_instant(),
        ).days,
        15_020.5
    );

    // X-Val: https://goo.gl/HC1C6W
    let mjd = ModifiedJulian::from_instant(
        Utc::new(1900, 01, 08, 00, 0, 0, 0)
            .expect("08 January 1900 invalid?!")
            .as_instant(),
    );
    assert_eq!(mjd.days, 15_027.0);
    assert_eq!(mjd.julian_days(), 2_415_027.5);

    // X-Val: https://goo.gl/drKoeV
    let gps_std_epoch = ModifiedJulian::from_instant(
        Utc::new(1980, 01, 06, 0, 0, 0, 0)
            .expect("06 January 1980 invalid?!")
            .as_instant(),
    );
    assert_eq!(gps_std_epoch.days, 44244.0);
    assert_eq!(gps_std_epoch.julian_days(), 2_444_244.5);

    // X-Val: https://goo.gl/tvqY23
    let j2000 = Utc::new(2000, 01, 01, 0, 0, 0, 0)
        .expect("01 January 2000 invalid?!")
        .as_instant();
    let mjd = ModifiedJulian::from_instant(j2000);
    assert_eq!(mjd.days, 51_544.0);
    assert_eq!(mjd.julian_days(), 2_451_544.5);

    // X-Val: https://goo.gl/Bu4YKh
    let jd020207 = ModifiedJulian::from_instant(
        Utc::new(2002, 02, 07, 0, 0, 0, 0)
            .expect("7 February 2002 invalid?!")
            .as_instant(),
    );
    assert_eq!(jd020207.days, 52312.0);
    assert_eq!(jd020207.julian_days(), 2_452_312.5);
}
