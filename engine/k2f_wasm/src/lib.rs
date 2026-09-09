#[cfg(feature = "compile")]
mod compile;
#[cfg(feature = "sdk")]
mod edit;
#[cfg(feature = "sdk")]
mod sdk;
#[cfg(feature = "viewer")]
mod viewer;

#[cfg(feature = "viewer")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
