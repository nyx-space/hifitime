extern crate hifitime;

#[test]
fn add_duration() {
    use hifitime::instant;
    use std::time::Duration;
    // Add in the Present era.
    let tick = instant::Instant::new(159, 10, instant::Era::Present) + Duration::new(5, 2);
    assert_eq!(tick.secs(), 164);
    assert_eq!(tick.nanos(), 12);
    match tick.era() {
        instant::Era::Present => assert_eq!(true, true),
        instant::Era::Past => assert_eq!(true, false),
    }

    // Add in the Past era.
    let tick = instant::Instant::new(159, 10, instant::Era::Past) + Duration::new(5, 2);
    assert_eq!(tick.secs(), 154);
    assert_eq!(tick.nanos(), 8);
    match tick.era() {
        instant::Era::Present => assert_eq!(true, false),
        instant::Era::Past => assert_eq!(true, true),
    }

    // Add from the Past to overflow into the Present
    let tick = instant::Instant::new(159, 0, instant::Era::Past) + Duration::new(160, 0);
    assert_eq!(tick.secs(), 1);
    assert_eq!(tick.nanos(), 0);
    match tick.era() {
        instant::Era::Present => assert_eq!(true, true),
        instant::Era::Past => assert_eq!(true, false),
    }
}
