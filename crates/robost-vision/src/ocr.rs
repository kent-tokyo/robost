//! OCR text extraction — requires the `ocr` feature and a Tesseract installation.
//!
//! # System requirements
//!
//! Install Tesseract and the desired language packs:
//!
//! ```sh
//! # macOS
//! brew install tesseract tesseract-lang
//!
//! # Ubuntu / Debian
//! sudo apt install tesseract-ocr tesseract-ocr-jpn tesseract-ocr-eng
//!
//! # Windows (installer from GitHub releases)
//! # https://github.com/UB-Mannheim/tesseract/wiki
//! ```
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "ocr")]
//! # {
//! use robost_vision::ocr::OcrEngine;
//! use image::open;
//!
//! let image = open("dialog.png").unwrap().into_rgba8();
//!
//! // Japanese OCR
//! let eng = OcrEngine::japanese();
//! let text = eng.extract_text(&image).unwrap();
//! println!("{text}");
//!
//! // Mixed Japanese + English
//! let eng = OcrEngine::new("jpn+eng");
//! # }
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
/// Wrap [`leptess::LepTess`] to extract text from [`RgbaImage`] regions.
/// Supports any language for which Tesseract data files are installed.
///
/// # CJK support
///
/// Use `"jpn"` for Japanese, `"chi_sim"` for Simplified Chinese,
/// `"chi_tra"` for Traditional Chinese, `"kor"` for Korean.
/// Combine with `+` for mixed scripts: `"jpn+eng"`.
pub struct OcrEngine {
    lang: String,
}

impl OcrEngine {
    /// Create an engine with the given Tesseract language string.
    ///
    /// Language codes follow Tesseract conventions: `"eng"`, `"jpn"`, `"jpn+eng"`, etc.
    pub fn new(lang: impl Into<String>) -> Self {
        Self { lang: lang.into() }
    }

    /// Japanese OCR (`jpn`).
    pub fn japanese() -> Self {
        Self::new("jpn")
    }

    /// Japanese + Latin mixed OCR (`jpn+eng`). Useful for UIs mixing romaji and kanji.
    pub fn japanese_with_latin() -> Self {
        Self::new("jpn+eng")
    }

    /// English OCR (`eng`).
    pub fn english() -> Self {
        Self::new("eng")
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
    /// `region` coordinates are relative to the image top-left.
    pub fn extract_text_in_region(&self, image: &RgbaImage, region: Rect) -> Result<String> {
        let x0 = region.x.max(0) as u32;
        let y0 = region.y.max(0) as u32;
        let x1 = (region.x + region.width as i32).min(image.width() as i32) as u32;
        let y1 = (region.y + region.height as i32).min(image.height() as i32) as u32;
        let w = x1.saturating_sub(x0);
        let h = y1.saturating_sub(y0);
        let roi = image::imageops::crop_imm(image, x0, y0, w, h).to_image();
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
