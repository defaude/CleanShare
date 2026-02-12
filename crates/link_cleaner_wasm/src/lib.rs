use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn clean_text(input: &str) -> String {
    link_cleaner_core::clean_text(input)
}
