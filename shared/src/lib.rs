pub use crux_core::{bridge::Bridge, Core, Request};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::wasm_bindgen;

pub mod app;
pub mod matrix;

pub use app::*;

uniffi::include_scaffolding!("shared");

static CORE: Lazy<Bridge<Effect, App>> = Lazy::new(|| Bridge::new(Core::new()));

#[wasm_bindgen]
pub fn process_event(data: &[u8]) -> Vec<u8> {
	CORE.process_event(data)
}

#[wasm_bindgen]
pub fn handle_response(id: u32, data: &[u8]) -> Vec<u8> {
	CORE.handle_response(id, data)
}

#[wasm_bindgen]
pub fn view() -> Vec<u8> {
	CORE.view()
}
