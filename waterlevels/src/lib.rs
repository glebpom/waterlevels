use core::cell::RefCell;
use std::collections::HashMap;

use lazy_static::lazy_static;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use core::cell::Cell;
use std::rc::Rc;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Model {
    inner: snapview_test_lib::Model,
}

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    pub fn new(values: Vec<f64>, max_time: f64) -> Self {
        Self {
            inner: snapview_test_lib::Model::new(
                &values,
                max_time,
            ).unwrap(),
        }
    }

    pub fn calculate(&self, time: f64) -> Vec<f64> {
        self.inner.calculate_levels(time).unwrap()
    }
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

    Ok(())
}
