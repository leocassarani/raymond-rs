extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub struct Scene {
    pixels: Vec<u8>,
}

#[wasm_bindgen]
impl Scene {
    pub fn new() -> Self {
        let capacity = 500 * 500 * 4;
        let mut pixels = Vec::with_capacity(capacity);
        for i in 0..capacity {
            if i % 4 == 0 {
                pixels.push(255);
                pixels.push(0);
                pixels.push(0);
                pixels.push(255);
            }
        }
        Self { pixels }
    }

    pub fn render(&self) -> *const u8 {
        self.pixels.as_ptr()
    }
}
