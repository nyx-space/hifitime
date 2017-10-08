extern crate hifitime;


#[test]
fn it_works2() {
    use hifitime::instant;
    use std::time::Duration;
    let example = instant::Instant::new(159, 0, instant::Era::Present);
    let delta = Duration::new(5, 0);
    println!("{:?}", example + delta);
}
