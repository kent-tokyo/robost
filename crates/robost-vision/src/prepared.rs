use crate::types::MaskRegion;
use image::{GrayImage, RgbaImage};

/// A template image with its grayscale conversion pre-computed.
///
/// Use this when the same template is searched many times (e.g. inside a
/// [`wait_for_match`] polling loop) to avoid repeating the RGB→gray conversion.
///
/// # Example
///
/// ```no_run
/// use robost_vision::{TemplateMatcher, ScreenPoint, PreparedTemplate};
/// use image::open;
///
/// let rgba = open("button.png").unwrap().into_rgba8();
/// let tmpl = PreparedTemplate::new(rgba);
///
/// let matcher = TemplateMatcher::default();
/// // haystack changes every loop iteration; template does not.
/// # let haystack = image::RgbaImage::new(1, 1);
/// let result = matcher.find_prepared(&haystack, &tmpl, ScreenPoint { x: 0, y: 0 });
/// ```
///
/// [`wait_for_match`]: crate::template_match::TemplateMatcher::wait_for_match
#[derive(Clone)]
pub struct PreparedTemplate {
    /// Original RGBA image (used for multi-scale resizing).
    pub rgba: RgbaImage,
    /// Pre-computed grayscale version (possibly after mask fill).
    pub(crate) gray: GrayImage,
    pub width: u32,
    pub height: u32,
}

impl PreparedTemplate {
    /// Prepare a template without any masks.
    pub fn new(rgba: RgbaImage) -> Self {
        let width = rgba.width();
        let height = rgba.height();
        let gray = to_gray(&rgba);
        Self {
            rgba,
            gray,
            width,
            height,
        }
    }

    /// Load a template from a file path, equivalent to `load_rgba` + `new`.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> image::ImageResult<Self> {
        crate::load_rgba(path).map(Self::new)
    }

    /// Prepare a template with mask regions filled before grayscale conversion.
    ///
    /// Masked pixels become neutral mid-gray, making the NCC invariant to those areas.
    pub fn with_masks(rgba: RgbaImage, masks: &[MaskRegion]) -> Self {
        let width = rgba.width();
        let height = rgba.height();
        let masked = fill_masks(&rgba, masks);
        let gray = to_gray(&masked);
        Self {
            rgba: masked,
            gray,
            width,
            height,
        }
    }
}

fn fill_masks(img: &RgbaImage, masks: &[MaskRegion]) -> RgbaImage {
    if masks.is_empty() {
        return img.clone();
    }
    let mut out = img.clone();
    let neutral = image::Rgba([128u8, 128, 128, 255]);
    for mask in masks {
        let r = &mask.rect;
        let x0 = r.x.max(0) as u32;
        let y0 = r.y.max(0) as u32;
        let x1 = (r.x + r.width as i32).min(img.width() as i32) as u32;
        let y1 = (r.y + r.height as i32).min(img.height() as i32) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                out.put_pixel(x, y, neutral);
            }
        }
    }
    out
}

fn to_gray(img: &RgbaImage) -> GrayImage {
    image::DynamicImage::ImageRgba8(img.clone()).into_luma8()
}
