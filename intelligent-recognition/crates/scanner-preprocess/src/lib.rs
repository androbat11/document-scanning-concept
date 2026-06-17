use scanner_types::{ModelInput, RawFrame, ScannerError};

/// Converts an already-resized RGBA frame into a model-ready tensor.
///
/// React is responsible for resizing the camera frame to the model input
/// dimensions (e.g. 640x640) before calling into WASM. This function
/// handles the remaining conversion steps:
///   1. RGBA -> RGB  (drop the alpha channel)
///   2. Normalize    (u8 [0, 255] -> f32 [0.0, 1.0])
///   3. HWC -> CHW   (rearrange axes for kornia-yolo tensor input)
pub fn preprocess(frame: RawFrame) -> Result<ModelInput, ScannerError> {
    todo!()
}
