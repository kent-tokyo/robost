//! Multi-scale NCC template matching for desktop automation.
//!
//! Finds a template image inside a screenshot using Normalized Cross-Correlation (NCC).
//! Tolerates RDP compression artifacts, DPI scaling, and dynamic UI elements via mask regions.
//!
//! # Quick start
//!
//! ```no_run
//! use robost_vision::{TemplateMatcher, ScreenPoint, load_rgba};
//!
//! let haystack = load_rgba("screenshot.png").unwrap();
//! let template  = load_rgba("button.png").unwrap();
//!
//! let matcher = TemplateMatcher::default(); // threshold 0.87, multi-scale on
//! let result  = matcher.find(&haystack, &template, ScreenPoint { x: 0, y: 0 }).unwrap();
//! println!("center={:?}  score={:.3}", result.center, result.score);
//! ```
//!
//! # Features
//!
//! | Feature | Enables | Extra system requirement |
//! |---------|---------|--------------------------|
//! | `template-match` *(default)* | NCC matching | — |
//! | `async` | `find_async`, `wait_for_match`, `wait_for_diff` | tokio runtime |
//! | `ocr` | [`ocr::OcrEngine`] | Tesseract + language data |
//! | `ml` | ML detection | ONNX Runtime |

pub mod diff;
pub mod prepared;
pub mod template_match;
pub mod types;

pub use diff::{diff, diff_in_region, DiffResult};
pub use prepared::PreparedTemplate;
pub use template_match::{MatchError, MatchResult, Result, TemplateMatcher};
pub use types::{MaskRegion, Rect, ScreenPoint, WindowPoint};

#[cfg(feature = "async")]
pub mod async_util;

#[cfg(feature = "async")]
pub use async_util::wait_for_diff;

// WinRT OCR takes priority on Windows when the `windows-ocr` feature is enabled.
#[cfg(all(feature = "windows-ocr", target_os = "windows"))]
#[path = "ocr_winrt.rs"]
pub mod ocr;

// Tesseract OCR for all other cases.
#[cfg(all(
    feature = "ocr",
    not(all(feature = "windows-ocr", target_os = "windows"))
))]
pub mod ocr;

#[cfg(any(feature = "ml", feature = "ml-vision"))]
pub mod ml;

/// Load an image file as RGBA8.
///
/// Thin convenience wrapper around [`image::open`] + `into_rgba8()`.
///
/// ```no_run
/// let img = robost_vision::load_rgba("screenshot.png").unwrap();
/// ```
pub fn load_rgba(path: impl AsRef<std::path::Path>) -> image::ImageResult<image::RgbaImage> {
    image::open(path).map(|d| d.into_rgba8())
}

/// Save an RGBA8 image to a file.
///
/// The output format is inferred from the file extension (`.png`, `.jpg`, etc.).
///
/// ```no_run
/// # let img = image::RgbaImage::new(1, 1);
/// robost_vision::save_rgba(&img, "debug_screenshot.png").unwrap();
/// ```
pub fn save_rgba(
    img: &image::RgbaImage,
    path: impl AsRef<std::path::Path>,
) -> image::ImageResult<()> {
    img.save(path)
}
