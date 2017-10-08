extern crate hifitime;

#[test]
fn julian() {
    use hifitime::instant;
    use hifitime::traits::TimeSystem;
    use hifitime::julian::ModifiedJulian;

    // Add in the Present era.
    let tick = instant::Instant::new(159, 10, instant::Era::Present);
    let mjd = ModifiedJulian::from_instant(tick);
    println!("{:?}", mjd);
    println!("{:?}", mjd.as_instant());
}
