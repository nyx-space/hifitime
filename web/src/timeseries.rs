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
use crate::epoch::Epoch;

/// A sequence of epochs with an even step.
///
/// The class is a cursor: `next()` moves it forward and returns `undefined` at
/// the end. Use `toArray()` to read every epoch without moving the cursor. The
/// exported class is not a JavaScript iterable, because wasm-bindgen cannot
/// export `Symbol.iterator`.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct TimeSeries(ht::TimeSeries);

#[wasm_bindgen]
impl TimeSeries {
    /// Builds a series that includes `start` and excludes `end`.
    pub fn exclusive(start: &Epoch, end: &Epoch, step: &Duration) -> TimeSeries {
        Self(ht::TimeSeries::exclusive(start.0, end.0, step.0))
    }

    /// Builds a series that includes both `start` and `end`.
    pub fn inclusive(start: &Epoch, end: &Epoch, step: &Duration) -> TimeSeries {
        Self(ht::TimeSeries::inclusive(start.0, end.0, step.0))
    }

    /// The first epoch of the series. The cursor does not move.
    #[wasm_bindgen(getter, js_name = firstEpoch)]
    pub fn first_epoch(&self) -> Epoch {
        self.0.first_epoch().into()
    }

    /// The last epoch of the series. The cursor does not move.
    #[wasm_bindgen(getter, js_name = lastEpoch)]
    pub fn last_epoch(&self) -> Epoch {
        self.0.last_epoch().into()
    }

    /// Returns the next epoch of the series, or `undefined` at the end.
    pub fn next(&mut self) -> Option<Epoch> {
        self.0.next().map(Epoch::from)
    }

    /// Returns the number of epochs that `next()` still returns.
    ///
    /// It is a method, not a property, because it walks a copy of the series. The
    /// `len` of the Rust iterator is not used here: it reports one epoch too few
    /// for an inclusive series, and it ignores the position of the cursor.
    pub fn count(&self) -> u32 {
        self.0.clone().count() as u32
    }

    /// Returns every remaining epoch as an array. The cursor does not move.
    ///
    /// Each element is an object that owns memory in the WebAssembly heap. Read
    /// the note about memory in the README before you build a large array.
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<Epoch> {
        self.0.clone().map(Epoch::from).collect()
    }

    /// Returns every remaining epoch, formatted with the time scale of each
    /// epoch. The cursor does not move.
    ///
    /// Prefer this method over `toArray()` when the strings are the result you
    /// need, because it does not allocate one object per epoch.
    #[wasm_bindgen(js_name = toStringArray)]
    pub fn to_string_array(&self) -> Vec<String> {
        self.0.clone().map(|epoch| epoch.to_string()).collect()
    }

    /// Returns a copy of this series, with its own cursor.
    #[wasm_bindgen(js_name = clone)]
    pub fn duplicate(&self) -> TimeSeries {
        self.clone()
    }

    /// Returns a readable form of this series.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
