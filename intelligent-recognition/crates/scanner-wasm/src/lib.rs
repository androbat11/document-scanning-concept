use wasm_bindgen::prelude::*;

/// Called every animation frame during live preview.
///
/// Accepts the 640x640 RGBA frame from the React detection canvas.
/// Returns a JS object with corner points, or null if no document is found.
#[wasm_bindgen]
pub fn detect_document(rgba: &[u8], width: u32, height: u32) -> JsValue {
    todo!()
}

/// Called once when the user captures the document.
///
/// Accepts the full-resolution RGBA frame from the React live canvas.
/// Returns PNG bytes of the digitalized document.
#[wasm_bindgen]
pub fn scan_document(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    todo!()
}
