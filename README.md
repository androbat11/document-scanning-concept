# Document Scanning Concept

A document scanner built in Rust, compiled to WebAssembly, and consumed by a React frontend. The computer vision and image processing pipeline runs entirely in Rust — the browser calls into it via WASM. React handles only the camera, UI, and rendering.

---

## Concept

The goal is to let a user point their camera at a physical document and receive a clean, digitalized image of it — flat, corrected, and readable — without any server round-trip. Everything runs locally in the browser.

The process has five distinct stages:

1. **Preprocess** — Prepare the camera frame for the detection model. React is responsible for resizing the frame to the model's expected input dimensions using the canvas API. Rust receives the already-resized frame and handles the remaining preparation: stripping the alpha channel (RGBA → RGB), normalizing pixel values to `[0.0, 1.0]`, and rearranging the axis layout to the channel-first format (CHW) that YOLO expects.
2. **Detect** — Run a YOLO model to determine whether a document is present in the frame and where it is (bounding box + confidence score). This drives the live camera preview: the React UI can draw a box around the detected document in real time.
3. **Scan** — Given the detected region, find the precise document boundary. Uses edge detection and corner extraction to identify the four corners of the document as a quadrilateral.
4. **Rectify** — Apply a perspective transformation (homography) to the quadrilateral, producing a flat, front-facing image as if the camera were directly above the document.
5. **Digitalize** — Post-process the rectified image into a clean document scan: adaptive thresholding, sharpening, and encoding to PNG.

---

## Architecture

The implementation is structured as a **Cargo workspace** — a single repository containing multiple Rust library crates, each responsible for exactly one stage of the pipeline.

### Why a multi-crate workspace?

- Each stage has a clear input/output contract and can be developed and tested independently.
- The WASM binding layer (`scanner-wasm`) stays thin — it only translates between JavaScript types and Rust types. No business logic lives there.
- The `scanner-pipeline` crate orchestrates all stages and can be tested natively with `cargo test`, without a browser.
- If the detection model changes (e.g. swap YOLO for another model), only `scanner-detect` is affected.

### Workspace Layout

```
document-scanning-concept/
├── Cargo.toml                  ← workspace root (lists all member crates)
│
├── scanner-pipeline/           ← orchestrates all stages (library)
│   ├── Cargo.toml
│   └── src/lib.rs
│
└── crates/
    ├── scanner-types/          ← shared types, traits, and error definitions
    ├── scanner-preprocess/     ← RGBA→RGB, normalize, CHW layout (React handles resize)
    ├── scanner-detect/         ← YOLO object detection (kornia-yolo)
    ├── scanner-scan/           ← edge detection + quad corner extraction
    ├── scanner-rectify/        ← homography + perspective warp
    ├── scanner-digitalize/     ← binarize, sharpen, encode to PNG
    └── scanner-wasm/           ← wasm-bindgen bindings (consumed by React)
```

Every crate is a **library crate** (`src/lib.rs`). There are no binaries. This is required for WASM compilation — only library crates can be compiled to `wasm32-unknown-unknown`.

---

## Crate Responsibilities

### `scanner-types`
Foundational types shared across all crates. Has no computer vision dependencies.

- `RawFrame` — raw RGBA pixel buffer from the camera
- `GrayImage` — single-channel image after preprocessing
- `Point2D` — a 2D coordinate
- `Quad` — four `Point2D` corners representing the document boundary
- `BoundingBox` — axis-aligned rectangle from YOLO detection
- `DetectionResult` — bounding box + confidence score
- `ScanResult` — the final output (PNG bytes + metadata)
- `ScannerError` — unified error enum for all stages

### `scanner-preprocess`
Prepares the camera frame for the YOLO model. This crate exists as a dedicated stage for separation of concerns — the conversion from a browser pixel buffer to a model-ready tensor is its sole responsibility.

React handles the resize (see React Integration). By the time the frame arrives here it is already at the correct spatial dimensions (e.g. 640×640). This crate then does:

- **RGBA → RGB** — drops the alpha channel that the browser always includes in `ImageData` but YOLO does not expect
- **Normalize** — converts integer pixel values `[0, 255]` to `[0.0, 1.0]` floats
- **HWC → CHW** — rearranges axis order from height-width-channels (browser layout) to channels-height-width (the tensor layout kornia-yolo expects)

Input: `RawFrame` (RGBA bytes, already at model input size)
Output: `ModelInput` (normalized float tensor in CHW format)

- Dependencies: `kornia-image`, `kornia-imgproc`, `scanner-types`

### `scanner-detect`
Runs object detection to locate the document in the frame.

- Loads and runs a YOLO model via `kornia-yolo`
- Returns `Option<DetectionResult>` — `None` if no document is found
- This is the only crate that depends on `kornia-yolo`
- Dependencies: `kornia-yolo`, `scanner-preprocess`, `scanner-types`

### `scanner-scan`
Extracts the precise document boundary from the detected region.

- Applies edge detection (Canny or Sobel) within the bounding box
- Finds contours and selects the largest quadrilateral
- Returns `Option<Quad>` — the four document corners
- Dependencies: `kornia-imgproc`, `kornia-feature`, `scanner-types`

### `scanner-rectify`
Flattens the document using a perspective transformation.

- Computes a homography matrix from the `Quad` corners
- Warps the original frame to produce a flat, front-facing image
- Dependencies: `kornia-geometry`, `scanner-types`

### `scanner-digitalize`
Produces the final clean document image.

- Applies adaptive thresholding (binarization)
- Sharpens edges
- Encodes output as PNG bytes
- Dependencies: `kornia-imgproc`, `image`, `scanner-types`

### `scanner-pipeline`
High-level orchestrator. Wires all stages into a complete pipeline.

Exposes two functions:
- `detect_only(frame) -> Option<DetectionResult>` — fast path for live preview
- `full_scan(frame) -> Result<ScanResult>` — full pipeline on capture

This crate is the primary target for integration tests. It does not depend on `scanner-wasm` and can run natively with `cargo test`.

- Dependencies: all stage crates

### `scanner-wasm`
The only crate that depends on `wasm-bindgen`. Its job is to translate between JavaScript types and Rust types, then delegate to `scanner-pipeline`.

Exports to JavaScript:
- `detect_document(rgba_ptr, width, height) -> JsValue` — returns corner points or `null` for live preview
- `scan_document(rgba_ptr, width, height) -> Uint8Array` — returns PNG bytes on capture

No business logic lives here.

- Dependencies: `scanner-pipeline`, `wasm-bindgen`, `js-sys`, `web-sys`

---

## Dependency Graph

```
scanner-wasm
  └── scanner-pipeline
        ├── scanner-digitalize  →  kornia-imgproc
        ├── scanner-rectify     →  kornia-geometry
        ├── scanner-scan        →  kornia-imgproc, kornia-feature
        ├── scanner-detect      →  kornia-yolo          ← isolated here only
        └── scanner-preprocess  →  kornia-image, kornia-imgproc
              (all above)       →  scanner-types        ← no kornia dependency
```

Dependency direction is strictly top-down. No cycles.

---

## Technology Stack

| Concern | Library |
|---|---|
| Computer vision primitives | `kornia-rs` (`kornia-image`, `kornia-imgproc`, `kornia-feature`) |
| Geometric transforms | `kornia-geometry` |
| Object detection | `kornia-yolo` |
| WASM bindings | `wasm-bindgen` |
| JS interop | `js-sys`, `web-sys` |
| Image encoding | `image` (PNG/JPEG output) |
| Frontend | React (camera, UI, viewer) |

---

## React Integration

React owns the camera, the canvas, and the document viewer. It calls into the WASM module for all computer vision work. No image processing logic lives in React — it is responsible only for frame extraction, resize, and rendering results.

### Camera Setup

```
navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' } })
  → <video> element (rear camera on mobile)
  → requestAnimationFrame loop draws each frame onto <canvas>
```

`facingMode: 'environment'` selects the rear camera on mobile, which is what document scanning requires.

### Two Canvases

React maintains two canvases simultaneously:

| Canvas | Resolution | Purpose |
|---|---|---|
| `liveCanvas` | Full camera resolution (e.g. 1920×1080) | Source of truth — used for the final scan |
| `detectionCanvas` | 640×640 (or model input size) | Resized copy passed to `detect_document()` each frame |

The resize from `liveCanvas` to `detectionCanvas` is done with a single `drawImage()` call. The browser handles this with hardware acceleration — it is faster and cheaper than resizing inside WASM.

### Live Detection Loop

On every animation frame:

```
1. Draw camera frame to liveCanvas (full resolution)
2. drawImage(liveCanvas, detectionCanvas, 0, 0, 640, 640)   ← hardware-accelerated resize
3. detectionCanvas.getImageData() → RGBA bytes
4. detect_document(rgba, 640, 640) → BoundingBox | null      ← WASM call
5. If BoundingBox returned: draw quad overlay on liveCanvas
6. requestAnimationFrame (repeat)
```

The overlay gives the user live feedback that the document has been detected and is in frame.

### On Capture

When the user taps the capture button (or auto-capture triggers on stable detection):

```
1. liveCanvas.getImageData() → RGBA bytes (full resolution)
2. scan_document(rgba, width, height) → Uint8Array (PNG bytes)   ← WASM call
3. URL.createObjectURL(new Blob([pngBytes])) → display in viewer
```

The full-resolution frame is passed directly to WASM — no resize. This ensures maximum output quality.

### WASM Module Lifecycle

The WASM module is loaded once at app startup using dynamic `import()`:

```js
import init, { detect_document, scan_document } from './scanner_wasm.js'
await init()  // loads and compiles the .wasm binary
```

After `init()` resolves, `detect_document` and `scan_document` are available as regular JS functions for the lifetime of the session.

---

## WASM Build Notes

- All crates except `scanner-wasm` are pure Rust and can be tested with `cargo test` on any platform.
- To build for WASM: `wasm-pack build crates/scanner-wasm --target web`
- YOLO model weights must either be bundled via `include_bytes!` at compile time or passed in as a byte slice from the React side at runtime.
- `kornia-yolo` may use Rayon for parallelism. Rayon is incompatible with standard WASM. If needed, disable default features and use single-threaded inference for the WASM target.
