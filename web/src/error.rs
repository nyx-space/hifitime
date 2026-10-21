/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use core::fmt::Display;

use wasm_bindgen::JsError;

/// Converts any hifitime error into a JavaScript `Error` that holds the
/// hifitime message.
pub(crate) fn js_err<E: Display>(err: E) -> JsError {
    JsError::new(&err.to_string())
}
