//! OCR text extraction — requires the `ocr` feature and a Tesseract installation.
//!
//! # System requirements
//!
//! ```sh
//! # macOS
//! brew install tesseract tesseract-lang
//!
//! # Ubuntu / Debian
//! sudo apt install tesseract-ocr tesseract-ocr-jpn tesseract-ocr-eng
//!
//! # Windows: https://github.com/UB-Mannheim/tesseract/wiki
//! ```

use crate::types::Rect;
use image::RgbaImage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("tesseract init failed: {0}")]
    Init(String),
    #[error("image encoding failed: {0}")]
    ImageEncode(String),
    #[error("OCR recognition failed: {0}")]
    Recognition(String),
}

pub type Result<T> = std::result::Result<T, OcrError>;

/// Tesseract-based OCR engine.
///
/// Wraps [`leptess::LepTess`] to extract text from [`RgbaImage`] regions.
/// Language codes follow Tesseract conventions: `"eng"`, `"jpn"`, `"jpn+eng"`, etc.
pub struct OcrEngine {
    lang: String,
}

impl OcrEngine {
    /// Create an engine with the given Tesseract language string.
    pub fn new(lang: impl Into<String>) -> Self {
        Self { lang: lang.into() }
    }

    pub fn english() -> Self {
        Self::new("eng")
    }

    pub fn japanese() -> Self {
        Self::new("jpn")
    }

    /// Japanese + Latin mixed (`jpn+eng`). Useful for UIs mixing romaji and kanji.
    pub fn japanese_with_latin() -> Self {
        Self::new("jpn+eng")
    }

    /// Extract all text from `image`.
    pub fn extract_text(&self, image: &RgbaImage) -> Result<String> {
        let mut tess =
            leptess::LepTess::new(None, &self.lang).map_err(|e| OcrError::Init(e.to_string()))?;

        let png = encode_png(image)?;
        tess.set_image_from_mem(&png)
            .map_err(|e| OcrError::Recognition(e.to_string()))?;

        tess.get_utf8_text()
            .map_err(|e| OcrError::Recognition(e.to_string()))
    }

    /// Extract text from the given rectangular region of `image`.
    ///
    /// Coordinates are relative to the image top-left (not screen-global).
    pub fn extract_text_in_region(&self, image: &RgbaImage, region: Rect) -> Result<String> {
        let x0 = region.x.max(0) as u32;
        let y0 = region.y.max(0) as u32;
        let x1 = (region.x + region.width as i32).min(image.width() as i32) as u32;
        let y1 = (region.y + region.height as i32).min(image.height() as i32) as u32;
        let roi = image::imageops::crop_imm(image, x0, y0, x1 - x0, y1 - y0).to_image();
        self.extract_text(&roi)
    }
}

fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgba8(image.clone())
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| OcrError::ImageEncode(e.to_string()))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn ocr_init_fails_gracefully_without_tesseract() {
        // When Tesseract is not installed, extract_text should return OcrError::Init,
        // not panic or hang.
        let img = RgbaImage::from_pixel(100, 30, Rgba([255, 255, 255, 255]));
        let eng = OcrEngine::english();
        match eng.extract_text(&img) {
            Ok(_) => {
                // Tesseract is installed — that's fine too
            }
            Err(OcrError::Init(_)) => {
                // Expected when Tesseract is not installed
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    #[ignore = "requires Tesseract + language packs to be installed"]
    fn ocr_extracts_english_text() {
        // To run: cargo test -p rpa-vision --features ocr -- --ignored
        //
        // Place a PNG with visible English text at the path below, or
        // adjust to point at a real fixture image.
        let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/eng_sample.png");
        let img = image::open(img_path)
            .expect("fixture not found")
            .into_rgba8();
        let eng = OcrEngine::english();
        let text = eng.extract_text(&img).expect("OCR failed");
        assert!(!text.trim().is_empty(), "expected non-empty OCR output");
    }

    #[test]
    #[ignore = "requires Tesseract + jpn language pack to be installed"]
    fn ocr_extracts_japanese_text() {
        let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/jpn_sample.png");
        let img = image::open(img_path)
            .expect("fixture not found")
            .into_rgba8();
        let eng = OcrEngine::japanese();
        let text = eng.extract_text(&img).expect("OCR failed");
        assert!(!text.trim().is_empty(), "expected non-empty OCR output");
    }
}
