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

    /// Find the bounding box of the first word in `image` whose text contains `text`.
    ///
    /// Tries the original image first, then an inverted copy (for light-on-dark UIs).
    /// Returns `None` when the text is not found in either pass.
    /// Coordinates are relative to the image top-left.
    pub fn find_text_bounds(&self, image: &RgbaImage, text: &str) -> Result<Option<Rect>> {
        // Pass 1: original image (works for dark text on light background)
        if let Some(r) = self.hocr_search(image, text)? {
            return Ok(Some(r));
        }
        // Pass 2: inverted image (works for light text on dark background)
        let inverted = invert_image(image);
        self.hocr_search(&inverted, text)
    }

    fn hocr_search(&self, image: &RgbaImage, text: &str) -> Result<Option<Rect>> {
        let mut tess =
            leptess::LepTess::new(None, &self.lang).map_err(|e| OcrError::Init(e.to_string()))?;
        let png = encode_png(image)?;
        tess.set_image_from_mem(&png)
            .map_err(|e| OcrError::Recognition(e.to_string()))?;
        let hocr = tess
            .get_hocr_text(0)
            .map_err(|e| OcrError::Recognition(e.to_string()))?;
        Ok(find_word_in_hocr(&hocr, text))
    }
}

/// Parse Tesseract HOCR output and return the bounding box of the first region
/// whose text contains `target` (case-insensitive).
///
/// Strategy:
///   1. Word level (`ocrx_word`) — exact single-token match.
///   2. Line level (`ocr_line`) — concatenate all words in the line; handles
///      Japanese text that Tesseract splits across multiple word spans.
fn find_word_in_hocr(hocr: &str, target: &str) -> Option<Rect> {
    let needle = target.to_lowercase();

    // Pass 1: word-level
    if let Some(r) = hocr_search_by_class(hocr, "ocrx_word", &needle, false) {
        return Some(r);
    }

    // Pass 2: line-level (joins all word texts in the line)
    hocr_search_by_class(hocr, "ocr_line", &needle, true)
}

/// Parse `s` as a coordinate integer, ignoring trailing non-digit characters
/// (e.g. "60;" → 60, "-10" → -10).
fn parse_coord(s: &str) -> Option<i32> {
    // Keep leading '-' and ascii digits only.
    let end = s
        .char_indices()
        .skip_while(|(i, c)| *i == 0 && *c == '-')
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s[..end].parse().ok()
}

/// Search `hocr` for spans of `class_name` whose visible text contains `needle`.
/// When `join_children` is true, the text of all nested word spans is concatenated
/// (spaces removed) before matching — needed for line-level search.
fn hocr_search_by_class(
    hocr: &str,
    class_name: &str,
    needle: &str,
    join_children: bool,
) -> Option<Rect> {
    let mut search = hocr;
    loop {
        let class_pos = search.find(class_name)?;
        search = &search[class_pos + class_name.len()..];

        // Find the closing '>' of this opening tag.
        let tag_end = search.find('>')?;
        let tag = &search[..tag_end];

        // Parse bbox from title='... bbox X0 Y0 X1 Y1; ...'
        let (x0, y0, x1, y1) = if let Some(b) = tag.find("bbox ") {
            let mut parts = tag[b + 5..].split_whitespace();
            match (
                parts.next().and_then(parse_coord),
                parts.next().and_then(parse_coord),
                parts.next().and_then(parse_coord),
                parts.next().and_then(parse_coord),
            ) {
                (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
                _ => continue,
            }
        } else {
            continue;
        };

        // Collect the visible text after the opening tag.
        let content = &search[tag_end + 1..];
        let text = if join_children {
            // Concatenate all word-span texts (strip HTML tags, remove spaces).
            strip_html_text(content)
        } else {
            // Single word span: text is directly between '>' and '<'.
            content
                .find('<')
                .map(|end| content[..end].trim().to_owned())
                .unwrap_or_default()
        };

        if text.to_lowercase().contains(needle) {
            return Some(Rect {
                x: x0,
                y: y0,
                width: (x1 - x0).max(0) as u32,
                height: (y1 - y0).max(0) as u32,
            });
        }
    }
}

/// Strip all HTML tags from `s` and return the concatenated text content
/// (without inter-word spaces, suitable for space-free CJK matching).
fn strip_html_text(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(lt) = rest.find('<') {
        // Text before the next tag.
        let text = rest[..lt].trim();
        if !text.is_empty() {
            out.push_str(text);
        }
        // Skip past the closing '>'.
        if let Some(gt) = rest[lt..].find('>') {
            rest = &rest[lt + gt + 1..];
        } else {
            break;
        }
    }
    // Remaining text after the last tag.
    let tail = rest.trim();
    if !tail.is_empty() {
        out.push_str(tail);
    }
    out
}

fn invert_image(image: &RgbaImage) -> RgbaImage {
    let mut out = image.clone();
    for px in out.pixels_mut() {
        px[0] = 255 - px[0];
        px[1] = 255 - px[1];
        px[2] = 255 - px[2];
        // preserve alpha
    }
    out
}

fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgba8(image.clone())
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| OcrError::ImageEncode(e.to_string()))?;
    Ok(buf)
}
