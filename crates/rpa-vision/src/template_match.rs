use image::{GrayImage, RgbaImage};
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use crate::{MaskRegion, Rect, ScreenPoint};
use thiserror::Error;
use tracing::instrument;

/// Minimum NCC score to accept a match. Chosen to tolerate RDP compression artifacts.
const DEFAULT_THRESHOLD: f32 = 0.87;

/// DPI scale factors tried for multi-scale matching (template scaled to these factors).
const SCALE_FACTORS: &[f32] = &[1.0, 0.8, 1.25];

#[derive(Debug, Error)]
pub enum MatchError {
    #[error("template larger than haystack")]
    TemplateTooLarge,
    #[error("no match above threshold {threshold:.2} (best score: {score:.4})")]
    BelowThreshold { score: f32, threshold: f32 },
}

pub type Result<T> = std::result::Result<T, MatchError>;

#[derive(Clone, Debug)]
pub struct MatchResult {
    /// Top-left of the matched region in screen-global coordinates.
    pub location: ScreenPoint,
    /// Centre of the matched region.
    pub center: ScreenPoint,
    /// NCC score in [0, 1].
    pub score: f32,
}

#[derive(Clone)]
pub struct TemplateMatcher {
    threshold: f32,
    /// Whether to try multiple scales when the first scale fails.
    multi_scale: bool,
}

impl Default for TemplateMatcher {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            multi_scale: true,
        }
    }
}

impl TemplateMatcher {
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            threshold,
            multi_scale: true,
        }
    }

    pub fn with_multi_scale(mut self, enabled: bool) -> Self {
        self.multi_scale = enabled;
        self
    }

    /// Find `template` inside `haystack` using normalised cross-correlation.
    /// `haystack_origin` is the screen-global top-left of the haystack image,
    /// so all returned coordinates are screen-global.
    /// `masks` are template-local rects to ignore during matching.
    #[instrument(name = "template_match", skip(self, haystack, template))]
    pub fn find(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
    ) -> Result<MatchResult> {
        self.find_with_masks(haystack, template, haystack_origin, &[])
    }

    /// Like [`find`] but with mask regions applied to the template before matching.
    #[instrument(name = "template_match_masked", skip(self, haystack, template, masks))]
    pub fn find_with_masks(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        masks: &[MaskRegion],
    ) -> Result<MatchResult> {
        let masked = apply_masks(template, masks);

        // Try scale 1.0 first; on failure try other scales if multi_scale is on.
        let scales: &[f32] = if self.multi_scale {
            SCALE_FACTORS
        } else {
            &[1.0]
        };

        let mut best_err = MatchError::BelowThreshold {
            score: 0.0,
            threshold: self.threshold,
        };

        for &scale in scales {
            let scaled = if (scale - 1.0).abs() < f32::EPSILON {
                masked.clone()
            } else {
                let nw = ((masked.width() as f32) * scale) as u32;
                let nh = ((masked.height() as f32) * scale) as u32;
                image::imageops::resize(&masked, nw, nh, image::imageops::FilterType::Lanczos3)
            };

            match self.match_one(haystack, &scaled, haystack_origin, scale) {
                Ok(m) => return Ok(m),
                Err(e) => best_err = e,
            }
        }

        Err(best_err)
    }

    fn match_one(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        scale: f32,
    ) -> Result<MatchResult> {
        if template.width() > haystack.width() || template.height() > haystack.height() {
            return Err(MatchError::TemplateTooLarge);
        }

        let hay_gray = to_gray(haystack);
        let tmpl_gray = to_gray(template);

        let scores = match_template(
            &hay_gray,
            &tmpl_gray,
            MatchTemplateMethod::CrossCorrelationNormalized,
        );
        let extremes = find_extremes(&scores);
        let score = extremes.max_value;
        let (mx, my) = (extremes.max_value_location.0, extremes.max_value_location.1);

        tracing::debug!(
            score,
            mx,
            my,
            scale,
            threshold = self.threshold,
            "template match result"
        );

        if score < self.threshold {
            return Err(MatchError::BelowThreshold {
                score,
                threshold: self.threshold,
            });
        }

        // Un-scale to get original template dimensions for centre calculation.
        let tw = (template.width() as f32 / scale) as i32;
        let th = (template.height() as f32 / scale) as i32;

        Ok(MatchResult {
            location: ScreenPoint {
                x: haystack_origin.x + mx as i32,
                y: haystack_origin.y + my as i32,
            },
            center: ScreenPoint {
                x: haystack_origin.x + mx as i32 + tw / 2,
                y: haystack_origin.y + my as i32 + th / 2,
            },
            score,
        })
    }
}

/// Fill mask rects in a copy of the template with a neutral mid-gray.
/// NCC is invariant to constant regions, so this effectively excludes dynamic areas.
fn apply_masks(template: &RgbaImage, masks: &[MaskRegion]) -> RgbaImage {
    if masks.is_empty() {
        return template.clone();
    }
    let mut out = template.clone();
    let neutral = image::Rgba([128u8, 128, 128, 255]);
    for mask in masks {
        fill_rect(&mut out, &mask.rect, neutral);
    }
    out
}

fn fill_rect(img: &mut RgbaImage, rect: &Rect, color: image::Rgba<u8>) {
    let x0 = rect.x.max(0) as u32;
    let y0 = rect.y.max(0) as u32;
    let x1 = (rect.x + rect.width as i32).min(img.width() as i32) as u32;
    let y1 = (rect.y + rect.height as i32).min(img.height() as i32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            img.put_pixel(x, y, color);
        }
    }
}

fn to_gray(img: &RgbaImage) -> GrayImage {
    image::DynamicImage::ImageRgba8(img.clone()).into_luma8()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn solid(w: u32, h: u32, color: Rgba<u8>) -> RgbaImage {
        RgbaImage::from_pixel(w, h, color)
    }

    #[test]
    fn template_too_large() {
        let hay = solid(10, 10, Rgba([255, 0, 0, 255]));
        let tmpl = solid(20, 20, Rgba([255, 0, 0, 255]));
        let m = TemplateMatcher::default();
        assert!(matches!(
            m.find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }),
            Err(MatchError::TemplateTooLarge)
        ));
    }

    #[test]
    fn mask_fill_stays_in_bounds() {
        let mut img = solid(50, 50, Rgba([200, 100, 50, 255]));
        // rect that goes outside image bounds — should clip gracefully
        fill_rect(
            &mut img,
            &Rect {
                x: 40,
                y: 40,
                width: 30,
                height: 30,
            },
            Rgba([0, 0, 0, 255]),
        );
        // Pixel at (49, 49) should be filled; (50, 50) is out of bounds and doesn't panic
        assert_eq!(*img.get_pixel(49, 49), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn apply_masks_neutral() {
        let tmpl = solid(20, 20, Rgba([255, 0, 0, 255]));
        let mask = MaskRegion {
            rect: Rect {
                x: 5,
                y: 5,
                width: 10,
                height: 10,
            },
            label: None,
        };
        let out = apply_masks(&tmpl, &[mask]);
        assert_eq!(*out.get_pixel(9, 9), Rgba([128, 128, 128, 255]));
        assert_eq!(*out.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    // LCG-based deterministic noise image. CrossCorrelationNormalized is scale-
    // invariant, so uniform images all give NCC = 1.0 regardless of color.
    // Non-uniform (noise) images give meaningful NCC scores.
    fn noise(w: u32, h: u32, seed: u64) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        let mut s = seed;
        for y in 0..h {
            for x in 0..w {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                let v = (s >> 56) as u8;
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        img
    }

    fn checker(w: u32, h: u32) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let v = if (x + y) % 2 == 0 { 200u8 } else { 0u8 };
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        img
    }

    /// Crop a 20×20 patch from a noise haystack and verify it's found at the
    /// correct location. Fully hermetic — no display, no filesystem.
    #[test]
    fn find_patch_at_known_position() {
        let hay = noise(200, 200, 42);
        // Template = exact crop from haystack at (70, 90).
        let tmpl: RgbaImage = image::imageops::crop_imm(&hay, 70, 90, 20, 20).to_image();

        let m = TemplateMatcher::default().with_multi_scale(false);
        let result = m
            .find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 })
            .expect("crop of haystack must be found");

        assert!(result.score > 0.99, "score={}", result.score);
        assert_eq!(result.location.x, 70);
        assert_eq!(result.location.y, 90);
        assert_eq!(result.center.x, 70 + 10); // top-left + half template width
        assert_eq!(result.center.y, 90 + 10);
    }

    /// Two independent noise images should not correlate above the 0.87 threshold.
    #[test]
    fn find_returns_below_threshold_when_absent() {
        let hay = noise(100, 100, 1111);
        let tmpl = noise(20, 20, 9999); // different seed → different pattern
        let m = TemplateMatcher::default().with_multi_scale(false);
        assert!(
            matches!(
                m.find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }),
                Err(MatchError::BelowThreshold { .. })
            ),
            "independent noise patterns should not match"
        );
    }

    /// Masking the entire template neutralises it to mid-gray (128).
    /// A gray-128 template vs a 0/200 checkerboard gives NCC ≈ 0.707, below the
    /// 0.87 threshold — so matching should fail even though the unmasked template
    /// matches perfectly.
    #[test]
    fn mask_prevents_match() {
        let hay = checker(80, 80);
        let tmpl = checker(20, 20);
        let m = TemplateMatcher::default().with_multi_scale(false);
        let origin = ScreenPoint { x: 0, y: 0 };

        // Without mask the checkerboard template is found (NCC = 1.0).
        assert!(
            m.find(&hay, &tmpl, origin).is_ok(),
            "unmasked checkerboard must match"
        );

        // Masking everything → template becomes all-gray (128) → NCC ≈ 0.707.
        let mask = MaskRegion {
            rect: Rect { x: 0, y: 0, width: 20, height: 20 },
            label: None,
        };
        assert!(
            matches!(
                m.find_with_masks(&hay, &tmpl, origin, &[mask]),
                Err(MatchError::BelowThreshold { .. })
            ),
            "fully masked template should not match (NCC ≈ 0.707 < 0.87)"
        );
    }
}
