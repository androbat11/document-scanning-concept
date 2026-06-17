use scanner_types::{RawFrame, ScanResult, ScannerError};

/// Post-processes the rectified image into a clean document scan.
///
/// Applies adaptive thresholding (binarization), sharpening, and
/// encodes the result as PNG bytes ready for display or storage.
pub fn digitalize(frame: RawFrame) -> Result<ScanResult, ScannerError> {
    todo!()
}
