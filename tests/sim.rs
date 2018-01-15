//#[feature(simulation)]
extern crate hifitime;

#[test]
fn clock_noise() {
    use hifitime::sim::ClockNoise;
    use std::time::Duration;

    // The IRIS clock is 1 part per billion per second
    let nasa_iris = ClockNoise::with_ppm_over_1sec(1e-3);
    let ddoor = Duration::new(8 * 60, 0);
    assert_eq!(
        (nasa_iris.noise_up(ddoor) - ddoor).as_secs(),
        0,
        "Expected a zero deviation for IRIS"
    );
}
