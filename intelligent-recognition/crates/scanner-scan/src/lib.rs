use scanner_types::{BoundingBox, Quad, RawFrame, ScannerError};

/// Extracts the precise document boundary within the detected region.
///
/// Applies edge detection and contour analysis to the bounding box area
/// of the full-resolution frame to find the four corners of the document.
///
/// Returns None if a clean quadrilateral cannot be identified.
pub fn scan(frame: &RawFrame, region: BoundingBox) -> Result<Option<Quad>, ScannerError> {
    todo!()
}
