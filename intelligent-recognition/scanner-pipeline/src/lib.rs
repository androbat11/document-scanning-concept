use scanner_types::{DetectionResult, RawFrame, ScanResult, ScannerError};

/// Fast path — called every frame during live preview.
/// Returns the detected document bounding box, or None if no document is found.
pub fn detect_only(frame: RawFrame) -> Option<DetectionResult> {
    todo!()
}

/// Full pipeline — called once on capture.
/// Runs all stages: preprocess → detect → scan → rectify → digitalize.
pub fn full_scan(frame: RawFrame) -> Result<ScanResult, ScannerError> {
    todo!()
}
