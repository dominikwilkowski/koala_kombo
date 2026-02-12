mod koala_kombo;
mod plugin;

pub use plugin::GamePlugin;

pub fn run_game() {
	GamePlugin::run_game();
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
	run_game();
}
