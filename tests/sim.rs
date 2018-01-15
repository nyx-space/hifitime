extern crate hifitime;

#[cfg(feature = "simulation")]
#[test]
fn clock_noise() {
    use hifitime::sim::ClockNoise;
    use std::time::Duration;

    // The IRIS clock is 1 part per billion per second
    let nasa_iris = ClockNoise::with_ppm_over_1sec(1e-3);
    let ddoor = Duration::new(8 * 60, 0);
    let noisy = nasa_iris.noise_up(ddoor);
    if noisy > ddoor {
        assert_eq!(
            (noisy - ddoor).as_secs(),
            0,
            "Expected a zero deviation for IRIS"
        );
    } else {
        assert_eq!(
            (ddoor - noisy).as_secs(),
            0,
            "Expected a zero deviation for IRIS"
        );
    }
}
