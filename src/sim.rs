extern crate rand;
extern crate rand_distr;

use self::rand::thread_rng;
use self::rand_distr::{Distribution, Normal};

/// ClockNoise adds true clock drift to a given Duration measurement. For example, if a vehicle is
/// measuring the time of flight of a signal with high precision oscillator, the engineering
/// specifications will include the oscillator stability. This specification bounds the preciseness
/// of time span calculations. On very short time spans, i.e. less than a few minutes, clock drift
/// is usually negligible. However, in several high fidelity systems the clock drift may lead to
/// a significant error (e.g. several kilometers in two-way radar ranging). This module allows high
/// fidelity simulation systems to test the resilience of algorithms with oscillator stability.
/// The constructors here are specified in parts per million: for a parts per billion specification
/// simply  multiply the value by `1e-3`.
/// *NOTE:* Clock stability is not linear. If a clock is rated at stable within 15 ppm per
/// fifteen minute interval this *does not* correspond to 1 ppm per minute.
///
/// # Example
/// ```
/// use hifitime::sim::ClockNoise;
///
/// // The IRIS clock is 1 part per billion over one second
/// let nasa_iris = ClockNoise::with_ppm_over_1sec(1e-3);
/// let ddoor = 8.0 * 60.0;
/// let noisy = nasa_iris.noise_up(ddoor);
/// assert!(
///     (noisy - ddoor).abs() < 1e-3,
///     "Expected a zero deviation for IRIS"
/// );
///
/// ```
pub struct ClockNoise {
    dist: Normal<f64>, // Stores the initialized Normal distribution generator
    span: f64,         // Stores the time span of the drift in seconds
}

impl ClockNoise {
    fn with_ppm_over(ppm: f64, span: f64) -> ClockNoise {
        ClockNoise {
            dist: Normal::new(span, ppm * 1e-6).unwrap(),
            span: span,
        }
    }
    /// Creates a new ClockNoise generator from the stability characteristics in parts per million
    /// over **one** second.
    pub fn with_ppm_over_1sec(ppm: f64) -> ClockNoise {
        ClockNoise::with_ppm_over(ppm, 1.0)
    }
    /// Creates a new ClockNoise generator from the stability characteristics in parts per million
    /// over **one minute** (i.e. 60 seconds).
    pub fn with_ppm_over_1min(ppm: f64) -> ClockNoise {
        ClockNoise::with_ppm_over(ppm, 60.0)
    }
    /// Creates a new ClockNoise generator from the stability characteristics in parts per million
    /// over **fifteen minutes** (i.e. 900 seconds).
    pub fn with_ppm_over_15min(ppm: f64) -> ClockNoise {
        ClockNoise::with_ppm_over(ppm, 900.0)
    }
    /// Returns a noisy Duration of the provided noiseless `Duration`
    pub fn noise_up(&self, noiseless: f64) -> f64 {
        let mut nl_secs = noiseless;
        let mut drift: f64 = 0.0;
        while nl_secs > 0.0 {
            drift += self.dist.sample(&mut thread_rng());
            nl_secs -= self.span;
        }
        drift
    }
}

#[test]
fn clock_noise() {
    let clock_1ppm_1s = ClockNoise::with_ppm_over_1sec(1.0);
    let clock_1ppm_1m = ClockNoise::with_ppm_over_1min(1.0);
    let clock_1ppm_15m = ClockNoise::with_ppm_over_15min(1.0);

    let mut err_1s = 0;
    let mut err_1m = 0;
    let mut err_15m = 0;

    // These all use normal distribution, so after 100 draws, there should be at most one large
    // deviation greater than the expected time span.

    for _ in 0..100 {
        if (clock_1ppm_1s.noise_up(1.0) - 1.0).abs() > 1.0 {
            err_1s += 1;
        }
        if (clock_1ppm_1m.noise_up(60.0) - 60.0).abs() > 60.0 {
            err_1m += 1;
        }
        if (clock_1ppm_15m.noise_up(900.0) - 900.0).abs() > 900.0 {
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
