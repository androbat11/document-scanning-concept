use scanner_types::{Quad, RawFrame, ScannerError};

/// Applies a perspective transformation to produce a flat, front-facing image.
///
/// Computes the homography matrix from the four Quad corners and warps
/// the full-resolution frame so the document appears as if photographed
/// directly from above.
pub fn rectify(frame: &RawFrame, quad: Quad) -> Result<RawFrame, ScannerError> {
    todo!()
}
