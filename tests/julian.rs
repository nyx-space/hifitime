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
    use hifitime::julian::{ModifiedJulian, J2000_OFFSET, SECONDS_PER_DAY, DAYS_PER_YEAR};
    // use hifitime::utc::Utc;
    // use hifitime::traits::TimeZone;

    // J2000 is defined at noon
    let j2000 = instant::Instant::new(
        (DAYS_PER_YEAR * SECONDS_PER_DAY * 100.0) as u64,
        0,
        instant::Era::Present,
    );
    let mjd = ModifiedJulian::from_instant(j2000);
    assert_eq!(mjd.days, J2000_OFFSET);
    assert_eq!(mjd.julian_days(), 2_451_545.0);

    // TODO: Add epoch tests from Vallado, p. 183, after UTC is implemented
    // NOTE: The J1900.0 offset in Vallado is different from the one given by NIST.
    // NOTE: From Nerem's slides:
    // 1980 Jan 6.0 2444244.5 GPS Standard Epoch
    // 2000 Jan 1.5 2451545.0 J2000 Epoch
    // The Julian Day Number for 7 February 2002 is 2452313
}
