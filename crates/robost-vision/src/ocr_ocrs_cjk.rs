//! ocrs-cjk OCR backend — Pure Rust, offline, CJK-enhanced.
//!
//! Requires the `ocrs-cjk-ocr` feature and model files:
//! - `detection.rten` (or `.onnx`) — text detection model
//! - `recognition.rten` (or `.onnx`) — text recognition model (PP-OCRv5 CJK)

use std::path::Path;
use image::RgbaImage;
use crate::types::Rect;

#[derive(Debug, thiserror::Error)]
pub enum OcrsCjkError {
    #[error("ocrs-cjk init failed: {0}")]
    Init(String),
    #[error("OCR recognition failed: {0}")]
    Recognition(String),
}

/// ocrs-cjk OCR engine wrapper. Load once; can be wrapped in `Arc` for shared use.
pub struct OcrsCjkEngine {
    inner: ocrs_cjk::OcrEngine,
}

// SAFETY: OcrEngine uses rten models which are Send + rayon-parallel.
// Verified by ocrs-cjk's own rayon usage.
unsafe impl Send for OcrsCjkEngine {}
unsafe impl Sync for OcrsCjkEngine {}

impl OcrsCjkEngine {
    /// Create engine from model file paths (`.rten` or `.onnx`).
    pub fn new(detection_model: &Path, recognition_model: &Path) -> Result<Self, OcrsCjkError> {
        let det = rten::Model::load_file(detection_model)
            .map_err(|e| OcrsCjkError::Init(e.to_string()))?;
        let rec = rten::Model::load_file(recognition_model)
            .map_err(|e| OcrsCjkError::Init(e.to_string()))?;
        let inner = ocrs_cjk::OcrEngine::new(ocrs_cjk::OcrEngineParams {
            detection_model: Some(det),
            recognition_model: Some(rec),
            alphabet_chars: Some(ocrs_cjk::cjk_alphabet_chars()),
            ..Default::default()
        })
        .map_err(|e| OcrsCjkError::Init(e.to_string()))?;
        Ok(Self { inner })
    }

    fn run_ocr(&self, image: &RgbaImage) -> Result<Vec<Option<ocrs_cjk::TextLine>>, OcrsCjkError> {
        // ocrs-cjk expects RGB8; robost uses RGBA
        let rgb = image::DynamicImage::ImageRgba8(image.clone()).to_rgb8();
        let src = ocrs_cjk::ImageSource::from_bytes(rgb.as_raw(), rgb.dimensions())
            .map_err(|e| OcrsCjkError::Recognition(e.to_string()))?;
        let input = self
            .inner
            .prepare_input(src)
            .map_err(|e| OcrsCjkError::Recognition(e.to_string()))?;
        let word_rects = self
            .inner
            .detect_words(&input)
            .map_err(|e| OcrsCjkError::Recognition(e.to_string()))?;
        let line_rects = self.inner.find_text_lines(&input, &word_rects);
        self.inner
            .recognize_text(&input, &line_rects)
            .map_err(|e| OcrsCjkError::Recognition(e.to_string()))
    }

    /// Extract all text from the image, lines joined by `\n`.
    pub fn extract_text(&self, image: &RgbaImage) -> Result<String, OcrsCjkError> {
        let lines = self.run_ocr(image)?;
        Ok(lines
            .iter()
            .flatten()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n"))
    }

    /// Extract text from a sub-region of the image.
    pub fn extract_text_in_region(
        &self,
        image: &RgbaImage,
        region: Rect,
    ) -> Result<String, OcrsCjkError> {
        let x0 = region.x.max(0) as u32;
        let y0 = region.y.max(0) as u32;
        let x1 = (region.x + region.width as i32).min(image.width() as i32).max(0) as u32;
        let y1 = (region.y + region.height as i32).min(image.height() as i32).max(0) as u32;
        if x1 <= x0 || y1 <= y0 {
            return Ok(String::new());
        }
        let cropped =
            image::imageops::crop_imm(image, x0, y0, x1 - x0, y1 - y0).to_image();
        self.extract_text(&cropped)
    }

    /// Return the bounding rectangle of the first occurrence of `needle` in the image.
    pub fn find_text_bounds(
        &self,
        image: &RgbaImage,
        needle: &str,
    ) -> Result<Option<Rect>, OcrsCjkError> {
        use ocrs_cjk::TextItem as _;
        let lines = self.run_ocr(image)?;
        for line in lines.iter().flatten() {
            let line_text = line.to_string();
            let Some(start) = line_text.find(needle) else {
                continue;
            };
            let end = start + needle.len();
            let chars = line.chars();
            let mut byte_pos = 0usize;
            let mut left = i32::MAX;
            let mut top = i32::MAX;
            let mut right = i32::MIN;
            let mut bottom = i32::MIN;
            let mut found_any = false;
            for tc in chars {
                let char_end = byte_pos + tc.char.len_utf8();
                if byte_pos >= start && char_end <= end {
                    left = left.min(tc.rect.left());
                    top = top.min(tc.rect.top());
                    right = right.max(tc.rect.right());
                    bottom = bottom.max(tc.rect.bottom());
                    found_any = true;
                }
                byte_pos = char_end;
            }
            if found_any {
                return Ok(Some(Rect {
                    x: left,
                    y: top,
                    width: (right - left).max(0) as u32,
                    height: (bottom - top).max(0) as u32,
                }));
            }
        }
        Ok(None)
    }
}
