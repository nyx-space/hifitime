use crate::{Duration, Errors};

#[cfg(target_arch = "wasm32")]
pub(crate) fn duration_since_unix_epoch() -> Result<Duration, Errors> {
    {
        use crate::Unit;
        use wasm_bindgen_rs::prelude::*;
        js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance"))
            .map_err(|_| Errors::SystemTimeError)
            .map(|performance| {
                performance.unchecked_into::<web_sys::Performance>().now() * Unit::Second
            })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn duration_since_unix_epoch() -> Result<Duration, Errors> {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|_| Errors::SystemTimeError)
        .and_then(|d| d.try_into().map_err(|_| Errors::SystemTimeError))
}
