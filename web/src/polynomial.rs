/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use hifitime as ht;
use wasm_bindgen::prelude::*;

use crate::duration::Duration;

/// An interpolation polynomial, used for the maintenance of a time scale and for
/// a precise conversion between two time scales.
///
/// A GNSS constellation publishes the three terms as `(a0, a1, a2)`: an offset
/// in seconds, a drift in seconds per second, and a drift change in seconds per
/// second squared.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Polynomial(pub(crate) ht::Polynomial);

#[wasm_bindgen]
impl Polynomial {
    /// Builds a polynomial from the three terms, in seconds, seconds per second,
    /// and seconds per second squared.
    #[wasm_bindgen(constructor)]
    pub fn new(constant_s: f64, rate_s_s: f64, accel_s_s2: f64) -> Polynomial {
        Self(ht::Polynomial::from((constant_s, rate_s_s, accel_s_s2)))
    }

    /// Builds a polynomial that holds a constant offset only.
    #[wasm_bindgen(js_name = fromConstantOffset)]
    pub fn from_constant_offset(constant: &Duration) -> Polynomial {
        Self(ht::Polynomial::from_constant_offset(constant.0))
    }

    /// Builds a polynomial that holds a constant offset in nanoseconds.
    #[wasm_bindgen(js_name = fromConstantOffsetNanoseconds)]
    pub fn from_constant_offset_nanoseconds(nanoseconds: f64) -> Polynomial {
        Self(ht::Polynomial::from_constant_offset_nanoseconds(
            nanoseconds,
        ))
    }

    /// Builds a polynomial that holds an offset and a rate.
    #[wasm_bindgen(js_name = fromOffsetAndRate)]
    pub fn from_offset_and_rate(constant: &Duration, rate: &Duration) -> Polynomial {
        Self(ht::Polynomial::from_offset_and_rate(constant.0, rate.0))
    }

    /// Builds a polynomial from an offset in nanoseconds and a drift in
    /// nanoseconds per second.
    #[wasm_bindgen(js_name = fromOffsetRateNanoseconds)]
    pub fn from_offset_rate_nanoseconds(
        offset_nanoseconds: f64,
        drift_nanoseconds_per_second: f64,
    ) -> Polynomial {
        Self(ht::Polynomial::from_offset_rate_nanoseconds(
            offset_nanoseconds,
            drift_nanoseconds_per_second,
        ))
    }

    /// The constant offset term.
    #[wasm_bindgen(getter)]
    pub fn constant(&self) -> Duration {
        self.0.constant.into()
    }

    /// The rate term, in seconds per second.
    #[wasm_bindgen(getter)]
    pub fn rate(&self) -> Duration {
        self.0.rate.into()
    }

    /// The acceleration term, in seconds per second squared.
    #[wasm_bindgen(getter)]
    pub fn accel(&self) -> Duration {
        self.0.accel.into()
    }

    /// Returns the correction for the given interpolation interval.
    #[wasm_bindgen(js_name = correctionDuration)]
    pub fn correction_duration(&self, time_interval: &Duration) -> Duration {
        self.0.correction_duration(time_interval.0).into()
    }

    /// Returns a copy of this polynomial.
    #[wasm_bindgen(js_name = clone)]
    pub fn duplicate(&self) -> Polynomial {
        *self
    }

    /// Returns a readable form of this polynomial.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
