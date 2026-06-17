use scanner_types::{DetectionResult, RawFrame, ScannerError};

/// Runs YOLO object detection on the preprocessed frame.
///
/// Accepts the raw RGBA frame (already resized by React to model input
/// dimensions), runs it through scanner-preprocess internally, then
/// passes the result to kornia-yolo for inference.
///
/// Returns None if no document is found in the frame.
pub fn detect(frame: RawFrame) -> Result<Option<DetectionResult>, ScannerError> {
    todo!()
}
