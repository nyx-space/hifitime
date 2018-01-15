extern crate hifitime;

#[cfg(feature = "simulation")]
#[test]
fn clock_noise() {
    use hifitime::sim::ClockNoise;
    use std::time::Duration;

    let clock_1ppm_1s = ClockNoise::with_ppm_over_1sec(1.0);
    let clock_1ppm_1m = ClockNoise::with_ppm_over_1min(1.0);
    let clock_1ppm_15m = ClockNoise::with_ppm_over_15min(1.0);
    let truth_1s = Duration::new(1, 0);
    let truth_1m = Duration::new(60, 0);
    let truth_15m = Duration::new(900, 0);

    let mut err_1s = 0;
    let mut err_1m = 0;
    let mut err_15m = 0;

    // These all use normal distribution, so after 100 draws, there should be at most one large
    // deviation greater than the expected time span.

    for _ in 0..100 {
        if clock_1ppm_1s.noise_up(truth_1s).as_secs() > 1 {
            err_1s += 1;
        }
        if clock_1ppm_1m.noise_up(truth_1m).as_secs() > 60 {
            err_1m += 1;
        }
        if clock_1ppm_15m.noise_up(truth_15m).as_secs() > 900 {
            err_15m += 1;
        }
    }
    assert!(
        err_1s <= 1,
        "Clock drift greater than span {:} times over 100 draws (1s)",
        err_1s
    );
    assert!(
        err_1m <= 1,
        "Clock drift greater than span {:} times over 100 draws (1m)",
        err_1m
    );
    assert!(
        err_15m <= 1,
        "Clock drift greater than span {:} times over 100 draws (15m)",
        err_15m
    );
}
