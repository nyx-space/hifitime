extern crate hifitime;

#[test]
fn add_duration() {
    use hifitime::instant::{Era, Instant, Duration};
    // Add in the Present era.
    let tick = Instant::new(159, 10, Era::Present) + Duration::new(5, 2);
    assert_eq!(tick.secs(), 164);
    assert_eq!(tick.nanos(), 12);
    assert_eq!(tick.era(), Era::Present);

    // Add in the Past era.
    let tick = Instant::new(159, 10, Era::Past) + Duration::new(5, 2);
    assert_eq!(tick.secs(), 154);
    assert_eq!(tick.nanos(), 8);
    assert_eq!(tick.era(), Era::Past);

    // Add from the Past to overflow into the Present
    let tick = Instant::new(159, 0, Era::Past) + Duration::new(160, 0);
    assert_eq!(tick.secs(), 1);
    assert_eq!(tick.nanos(), 0);
    assert_eq!(tick.era(), Era::Present);
}
