/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

//! WebAssembly and TypeScript bindings for hifitime.
//!
//! Each exported class is a thin wrapper around the hifitime type of the same
//! name. The wrapper adds no time logic: every method calls hifitime.

// JavaScript names such as `add`, `next`, `toString`, and `clone` are the
// whole point of this crate, so the lints that ask for the Rust traits instead
// do not apply here.
#![allow(
    clippy::should_implement_trait,
    clippy::inherent_to_string,
    clippy::wrong_self_convention,
    clippy::too_many_arguments,
    clippy::new_without_default
)]

use wasm_bindgen::prelude::*;

mod constants;
mod duration;
mod enums;
mod epoch;
mod error;
mod polynomial;
mod timeseries;

pub use constants::{constants, Constants};
pub use duration::{Duration, DurationComponents, DurationParts};
pub use enums::{
    is_gnss_time_scale, month_from_number, month_name, month_number, parse_time_scale,
    parse_weekday, time_scale_name, unit_in_seconds, weekday_name, MonthName, TimeScale, Unit,
    Weekday,
};
pub use epoch::{Epoch, GregorianDate, TimeOfWeek};
pub use polynomial::Polynomial;
pub use timeseries::TimeSeries;

/// Runs once, when the module finishes its initialization.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Returns the version of hifitime that this module wraps.
#[wasm_bindgen]
pub fn version() -> String {
    env!("HIFITIME_VERSION").to_string()
}
