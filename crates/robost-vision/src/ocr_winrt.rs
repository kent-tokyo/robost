//! WinRT OCR (Windows.Media.Ocr) — requires the `windows-ocr` feature.
//!
//! Uses the Windows 10/11 built-in OCR engine. No Tesseract installation needed.
//!
//! # Language packs
//!
//! The recognizer language must be installed in Windows.
//! Open Settings → Time & Language → Language & Region → Add a language.
//! Then install the "Optical character recognition" optional feature for that language.
//!
//! Common BCP-47 tags accepted by Tesseract-style codes:
//! `"jpn"` → Japanese, `"eng"` → English, `"chi_sim"` → Simplified Chinese.

use crate::types::Rect;
use image::RgbaImage;
use thiserror::Error;
use windows::{
    core::HSTRING,
    Globalization::Language,
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine as WinOcrEngine,
    Storage::Streams::DataWriter,
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
};

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("WinRT OCR init failed: {0}")]
    Init(String),
    #[error("image encoding failed: {0}")]
    ImageEncode(String),
    #[error("OCR recognition failed: {0}")]
    Recognition(String),
    #[error(
        "OCR language pack not installed for '{0}'. \
         Install via Settings \u{2192} Time & Language \u{2192} Language & Region \u{2192} \
         Add a language, then install the \"Optical character recognition\" optional feature."
    )]
    LanguageNotInstalled(String),
}

pub type Result<T> = std::result::Result<T, OcrError>;

impl From<windows::core::Error> for OcrError {
    fn from(e: windows::core::Error) -> Self {
        OcrError::Recognition(e.to_string())
    }
}

/// WinRT-based OCR engine using the Windows 10/11 built-in recognizer.
///
/// Language codes follow Tesseract conventions (`"jpn"`, `"eng"`, `"chi_sim"`, etc.)
/// and are automatically mapped to BCP-47 tags required by WinRT.
pub struct OcrEngine {
    lang: String,
}

impl OcrEngine {
    /// Create an engine for the given Tesseract-style language code.
    pub fn new(lang: impl Into<String>) -> Self {
        Self { lang: lang.into() }
    }

    /// Japanese OCR (`jpn`).
    pub fn japanese() -> Self {
        Self::new("jpn")
    }

    /// Japanese + Latin mixed OCR (`jpn+eng`). Uses the Japanese recognizer.
    pub fn japanese_with_latin() -> Self {
        Self::new("jpn+eng")
    }

    /// English OCR (`eng`).
    pub fn english() -> Self {
        Self::new("eng")
    }

    /// Extract all text from `image`.
    pub fn extract_text(&self, image: &RgbaImage) -> Result<String> {
        let engine = create_engine(&self.lang)?;
        let bitmap = image_to_software_bitmap(image)?;
        let result = engine.RecognizeAsync(&bitmap)?.get()?;

        let mut text = String::new();
        let lines = result.Lines()?;
        for li in 0..lines.Size()? {
            let line = lines.GetAt(li)?;
            text.push_str(&line.Text()?.to_string());
            text.push('\n');
        }
        Ok(text)
    }

    /// Extract text from a rectangular region of `image`.
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

    /// Find the bounding box of the first region containing `text`.
    ///
    /// Tries the original image first, then an inverted copy (for light-on-dark UIs).
    pub fn find_text_bounds(&self, image: &RgbaImage, text: &str) -> Result<Option<Rect>> {
        if let Some(r) = self.winrt_search(image, text)? {
            return Ok(Some(r));
        }
        let inverted = invert_image(image);
        self.winrt_search(&inverted, text)
    }

    fn winrt_search(&self, image: &RgbaImage, text: &str) -> Result<Option<Rect>> {
        let engine = create_engine(&self.lang)?;
        let bitmap = image_to_software_bitmap(image)?;
        let result = engine.RecognizeAsync(&bitmap)?.get()?;
        let needle = text.to_lowercase();

        let lines = result.Lines()?;
        for li in 0..lines.Size()? {
            let line = lines.GetAt(li)?;
            let words = line.Words()?;
            let n = words.Size()?;

            // Pass 1: word-level match (single token).
            for i in 0..n {
                let word = words.GetAt(i)?;
                let word_text = word.Text()?.to_string().to_lowercase();
                if word_text.contains(&needle) {
                    let r = word.BoundingRect()?;
                    return Ok(Some(Rect {
                        x: r.X as i32,
                        y: r.Y as i32,
                        width: r.Width as u32,
                        height: r.Height as u32,
                    }));
                }
            }

            // Pass 2: line-level match (joins words; handles CJK where spaces are absent).
            let line_text = line.Text()?.to_string();
            let joined: String = line_text.chars().filter(|c| !c.is_whitespace()).collect();
            if joined.to_lowercase().contains(&needle) {
                // Return the union bounding rect of all words in this line.
                let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
                let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
                for i in 0..n {
                    let r = words.GetAt(i)?.BoundingRect()?;
                    min_x = min_x.min(r.X);
                    min_y = min_y.min(r.Y);
                    max_x = max_x.max(r.X + r.Width);
                    max_y = max_y.max(r.Y + r.Height);
                }
                if max_x > min_x {
                    return Ok(Some(Rect {
                        x: min_x as i32,
                        y: min_y as i32,
                        width: (max_x - min_x) as u32,
                        height: (max_y - min_y) as u32,
                    }));
                }
            }
        }

        Ok(None)
    }
}

/// Map Tesseract language code to a BCP-47 tag accepted by WinRT OCR.
/// Only the primary language (before `+`) is used.
fn tesseract_to_bcp47(lang: &str) -> &str {
    let primary = lang.split('+').next().unwrap_or(lang);
    match primary {
        "jpn" => "ja",
        "eng" => "en-US",
        "chi_sim" => "zh-Hans-CN",
        "chi_tra" => "zh-Hant-TW",
        "kor" => "ko",
        "fra" => "fr-FR",
        "deu" => "de-DE",
        "spa" => "es-ES",
        "por" => "pt-PT",
        "ita" => "it-IT",
        "rus" => "ru-RU",
        "ara" => "ar",
        other => other, // pass through (may already be BCP-47)
    }
}

fn create_engine(lang: &str) -> Result<WinOcrEngine> {
    // Initialize COM MTA for this thread.
    // Ignoring the HRESULT is intentional: S_OK, S_FALSE, and RPC_E_CHANGED_MODE
    // all mean "COM is usable on this thread."
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    let bcp47 = tesseract_to_bcp47(lang);
    let language = Language::CreateLanguage(&HSTRING::from(bcp47))
        .map_err(|e| OcrError::Init(e.to_string()))?;

    // Returns an error (HRESULT) when the language pack is not installed.
    WinOcrEngine::TryCreateFromLanguage(&language)
        .map_err(|_| OcrError::LanguageNotInstalled(bcp47.to_owned()))
}

fn image_to_software_bitmap(image: &RgbaImage) -> Result<SoftwareBitmap> {
    let width = image.width() as i32;
    let height = image.height() as i32;

    // RgbaImage is RGBA; Windows BGRA8 expects BGRA byte order.
    let mut bgra = Vec::with_capacity((width * height * 4) as usize);
    for px in image.pixels() {
        bgra.push(px[2]); // B
        bgra.push(px[1]); // G
        bgra.push(px[0]); // R
        bgra.push(px[3]); // A
    }

    let writer = DataWriter::new().map_err(|e| OcrError::ImageEncode(e.to_string()))?;
    writer
        .WriteBytes(&bgra)
        .map_err(|e| OcrError::ImageEncode(e.to_string()))?;
    let buffer = writer
        .DetachBuffer()
        .map_err(|e| OcrError::ImageEncode(e.to_string()))?;

    SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, width, height)
        .map_err(|e| OcrError::ImageEncode(e.to_string()))
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
