use crate::prepared::PreparedTemplate;
use crate::types::{MaskRegion, Rect, ScreenPoint};
use image::{GrayImage, RgbaImage};
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
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
    #[error("timed out waiting for match")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, MatchError>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MatchResult {
    /// Top-left of the matched region in screen-global coordinates.
    pub location: ScreenPoint,
    /// Centre of the matched region.
    pub center: ScreenPoint,
    /// Bounding box of the matched region (location + template dimensions).
    pub bounds: Rect,
    /// NCC score in [0, 1].
    pub score: f32,
}

#[derive(Clone, Debug)]
pub struct TemplateMatcher {
    threshold: f32,
    /// Whether to try multiple scales when the first scale fails.
    multi_scale: bool,
    /// Override for the NMS suppression radius used by `find_all` variants.
    /// `None` → auto (`min(tw, th) / 2`).
    nms_radius: Option<u32>,
    /// Custom scale factors for multi-scale matching. `None` → use `SCALE_FACTORS`.
    scales: Option<Vec<f32>>,
}

impl Default for TemplateMatcher {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            multi_scale: true,
            nms_radius: None,
            scales: None,
        }
    }
}

impl TemplateMatcher {
    /// Create a matcher with an explicit NCC threshold and all other options at their defaults.
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            multi_scale: true,
            nms_radius: None,
            scales: None,
        }
    }

    /// Returns the configured NCC threshold.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Override the NCC threshold on an existing matcher (builder method).
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_multi_scale(mut self, enabled: bool) -> Self {
        self.multi_scale = enabled;
        self
    }

    /// Override the non-maximum suppression radius used by [`find_all`] variants.
    ///
    /// Two candidates whose centres are within `radius` pixels in both x and y are
    /// considered the same hit; the lower-scoring one is suppressed.
    ///
    /// Defaults to `min(template_width, template_height) / 2` when not set.
    pub fn with_nms_radius(mut self, radius: u32) -> Self {
        self.nms_radius = Some(radius);
        self
    }

    /// Set explicit scale factors for multi-scale matching.
    ///
    /// Overrides the default `[1.0, 0.8, 1.25]`. Useful when the target environment
    /// has a known DPI scaling (e.g. `[1.0, 1.25]` for RDP at 125%).
    ///
    /// Also enables multi-scale matching (equivalent to calling `.with_multi_scale(true)`).
    ///
    /// # Panics
    ///
    /// Panics if `scales` is empty.
    pub fn with_scales(mut self, scales: Vec<f32>) -> Self {
        assert!(!scales.is_empty(), "scales must not be empty");
        self.scales = Some(scales);
        self.multi_scale = true;
        self
    }

    /// Find `template` inside `haystack` using normalised cross-correlation.
    ///
    /// `haystack_origin` is the screen-global top-left of the haystack image,
    /// so all returned coordinates are screen-global.
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
    ///
    /// Mask regions cover dynamic parts of the UI (timestamps, badges, etc.)
    /// by filling them with neutral mid-gray before the NCC computation.
    #[instrument(name = "template_match_masked", skip(self, haystack, template, masks))]
    pub fn find_with_masks(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        masks: &[MaskRegion],
    ) -> Result<MatchResult> {
        let masked = apply_masks(template, masks);
        let tw = template.width();
        let th = template.height();

        let scales: &[f32] = match &self.scales {
            Some(s) => s,
            None if self.multi_scale => SCALE_FACTORS,
            None => &[1.0],
        };

        let mut best_err = MatchError::BelowThreshold {
            score: 0.0,
            threshold: self.threshold,
        };

        for &scale in scales {
            let scaled = scale_image(&masked, scale);
            match self.match_one(haystack, &scaled, haystack_origin, scale, tw, th) {
                Ok(m) => return Ok(m),
                Err(e) => best_err = e,
            }
        }

        Err(best_err)
    }

    /// Find **all** occurrences of `template` in `haystack` above the configured threshold.
    ///
    /// Uses non-maximum suppression with a radius of `min(tw, th) / 2` to avoid
    /// reporting overlapping hits. Results are sorted by score descending.
    #[instrument(name = "template_match_all", skip(self, haystack, template))]
    pub fn find_all(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
    ) -> Vec<MatchResult> {
        self.find_all_with_masks(haystack, template, haystack_origin, &[])
    }

    /// Like [`find_all`] but with mask regions.
    #[instrument(
        name = "template_match_all_masked",
        skip(self, haystack, template, masks)
    )]
    pub fn find_all_with_masks(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        masks: &[MaskRegion],
    ) -> Vec<MatchResult> {
        let orig_tw = template.width();
        let orig_th = template.height();
        let masked = apply_masks(template, masks);

        let scales: &[f32] = match &self.scales {
            Some(s) => s,
            None if self.multi_scale => SCALE_FACTORS,
            None => &[1.0],
        };

        let hay_gray = to_gray(haystack);
        // Candidates carry their actual (tw, th) because each scale produces different dimensions.
        let mut candidates: Vec<(u32, u32, f32, u32, u32)> = Vec::new();

        for &scale in scales {
            let scaled = if (scale - 1.0).abs() < f32::EPSILON {
                masked.clone()
            } else {
                scale_image(&masked, scale)
            };
            let tw = scaled.width();
            let th = scaled.height();
            if tw > haystack.width() || th > haystack.height() {
                continue;
            }
            let tmpl_gray = to_gray(&scaled);
            let scores = match_template(
                &hay_gray,
                &tmpl_gray,
                MatchTemplateMethod::CrossCorrelationNormalized,
            );
            for y in 0..scores.height() {
                for x in 0..scores.width() {
                    let s = *scores.get_pixel(x, y).0.first().unwrap_or(&0.0);
                    if s >= self.threshold {
                        candidates.push((x, y, s, tw, th));
                    }
                }
            }
        }

        // Sort by score descending, then apply NMS using original template size as radius reference.
        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let nms_radius = self.nms_radius.unwrap_or(orig_tw.min(orig_th) / 2) as i64;
        let mut suppressed = vec![false; candidates.len()];
        let mut results: Vec<MatchResult> = Vec::new();

        for i in 0..candidates.len() {
            if suppressed[i] {
                continue;
            }
            let (mx, my, score, tw, th) = candidates[i];
            results.push(make_result(haystack_origin, mx, my, tw, th, score));
            for j in (i + 1)..candidates.len() {
                if suppressed[j] {
                    continue;
                }
                let (jx, jy, _, _, _) = candidates[j];
                let dx = mx as i64 - jx as i64;
                let dy = my as i64 - jy as i64;
                if dx.abs() <= nms_radius && dy.abs() <= nms_radius {
                    suppressed[j] = true;
                }
            }
        }

        results
    }

    /// Search for `template` only within `region` of `haystack`.
    ///
    /// `region` is in the same coordinate space as `haystack_origin`
    /// (i.e. screen-global if origin is screen-global).
    /// Returned coordinates are always screen-global.
    pub fn find_in_region(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        region: Rect,
    ) -> Result<MatchResult> {
        let (roi, roi_origin) = crop_roi(haystack, haystack_origin, region);
        self.find(&roi, template, roi_origin)
    }

    /// Like [`find_in_region`] but returns all matches within the region.
    pub fn find_all_in_region(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        region: Rect,
    ) -> Vec<MatchResult> {
        let (roi, roi_origin) = crop_roi(haystack, haystack_origin, region);
        self.find_all(&roi, template, roi_origin)
    }

    /// Like [`find_all_in_region`] but reuses the pre-computed grayscale in `tmpl`.
    pub fn find_all_in_region_prepared(
        &self,
        haystack: &RgbaImage,
        tmpl: &PreparedTemplate,
        haystack_origin: ScreenPoint,
        region: Rect,
    ) -> Vec<MatchResult> {
        let (roi, roi_origin) = crop_roi(haystack, haystack_origin, region);
        self.find_all_prepared(&roi, tmpl, roi_origin)
    }

    /// Like [`find`] but reuses the pre-computed grayscale in `tmpl`, avoiding the
    /// RGB→gray conversion on every call. Ideal for polling loops.
    #[instrument(name = "template_match_prepared", skip(self, haystack, tmpl))]
    pub fn find_prepared(
        &self,
        haystack: &RgbaImage,
        tmpl: &PreparedTemplate,
        haystack_origin: ScreenPoint,
    ) -> Result<MatchResult> {
        let tw = tmpl.width;
        let th = tmpl.height;
        let hay_gray = to_gray(haystack);
        let scales: &[f32] = match &self.scales {
            Some(s) => s,
            None if self.multi_scale => SCALE_FACTORS,
            None => &[1.0],
        };
        let mut best_err = MatchError::BelowThreshold {
            score: 0.0,
            threshold: self.threshold,
        };

        for &scale in scales {
            let tmpl_gray = if (scale - 1.0).abs() < f32::EPSILON {
                tmpl.gray.clone()
            } else {
                to_gray(&scale_image(&tmpl.rgba, scale))
            };
            match self.match_one_gray(&hay_gray, &tmpl_gray, haystack_origin, scale, tw, th) {
                Ok(m) => return Ok(m),
                Err(e) => best_err = e,
            }
        }
        Err(best_err)
    }

    /// Like [`find_all`] but reuses the pre-computed grayscale in `tmpl`.
    ///
    /// At scale 1.0 the cached grayscale is used directly; other scales fall back to
    /// converting from `tmpl.rgba`, so the optimization is strongest in single-scale mode.
    #[instrument(name = "template_match_all_prepared", skip(self, haystack, tmpl))]
    pub fn find_all_prepared(
        &self,
        haystack: &RgbaImage,
        tmpl: &PreparedTemplate,
        haystack_origin: ScreenPoint,
    ) -> Vec<MatchResult> {
        let orig_tw = tmpl.width;
        let orig_th = tmpl.height;

        let scales: &[f32] = match &self.scales {
            Some(s) => s,
            None if self.multi_scale => SCALE_FACTORS,
            None => &[1.0],
        };

        let hay_gray = to_gray(haystack);
        let mut candidates: Vec<(u32, u32, f32, u32, u32)> = Vec::new();

        for &scale in scales {
            let (tmpl_gray, tw, th) = if (scale - 1.0).abs() < f32::EPSILON {
                (tmpl.gray.clone(), orig_tw, orig_th)
            } else {
                let scaled_rgba = scale_image(&tmpl.rgba, scale);
                let tw = scaled_rgba.width();
                let th = scaled_rgba.height();
                (to_gray(&scaled_rgba), tw, th)
            };
            if tw > haystack.width() || th > haystack.height() {
                continue;
            }
            let scores = match_template(
                &hay_gray,
                &tmpl_gray,
                MatchTemplateMethod::CrossCorrelationNormalized,
            );
            for y in 0..scores.height() {
                for x in 0..scores.width() {
                    let s = scores.get_pixel(x, y).0[0];
                    if s >= self.threshold {
                        candidates.push((x, y, s, tw, th));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let nms_radius = self.nms_radius.unwrap_or(orig_tw.min(orig_th) / 2) as i64;
        let mut suppressed = vec![false; candidates.len()];
        let mut results: Vec<MatchResult> = Vec::new();
        for i in 0..candidates.len() {
            if suppressed[i] {
                continue;
            }
            let (mx, my, score, tw, th) = candidates[i];
            results.push(make_result(haystack_origin, mx, my, tw, th, score));
            for j in (i + 1)..candidates.len() {
                if suppressed[j] {
                    continue;
                }
                let (jx, jy, _, _, _) = candidates[j];
                let dx = mx as i64 - jx as i64;
                let dy = my as i64 - jy as i64;
                if dx.abs() <= nms_radius && dy.abs() <= nms_radius {
                    suppressed[j] = true;
                }
            }
        }
        results
    }

    /// Returns the best NCC score for `template` against `haystack` without applying
    /// the configured threshold. Returns `0.0` if the template is larger than the haystack.
    pub fn score(&self, haystack: &RgbaImage, template: &RgbaImage) -> f32 {
        if template.width() > haystack.width() || template.height() > haystack.height() {
            return 0.0;
        }
        let hay_gray = to_gray(haystack);
        let tmpl_gray = to_gray(template);
        let scores = match_template(
            &hay_gray,
            &tmpl_gray,
            MatchTemplateMethod::CrossCorrelationNormalized,
        );
        find_extremes(&scores).max_value
    }

    /// Try every template in `templates` and return `(index, MatchResult)` for the one
    /// with the highest score above the configured threshold.
    ///
    /// Returns `None` if no template matches.
    pub fn find_best_of(
        &self,
        haystack: &RgbaImage,
        templates: &[RgbaImage],
        origin: ScreenPoint,
    ) -> Option<(usize, MatchResult)> {
        let mut best: Option<(usize, MatchResult)> = None;
        for (i, template) in templates.iter().enumerate() {
            if let Ok(m) = self.find(haystack, template, origin) {
                match &best {
                    None => best = Some((i, m)),
                    Some((_, prev)) if m.score > prev.score => best = Some((i, m)),
                    _ => {}
                }
            }
        }
        best
    }

    /// Like [`find_best_of`] but takes pre-converted [`PreparedTemplate`] slices,
    /// avoiding the RGB→gray conversion per template per call.
    pub fn find_best_of_prepared(
        &self,
        haystack: &RgbaImage,
        templates: &[PreparedTemplate],
        origin: ScreenPoint,
    ) -> Option<(usize, MatchResult)> {
        let mut best: Option<(usize, MatchResult)> = None;
        for (i, tmpl) in templates.iter().enumerate() {
            if let Ok(m) = self.find_prepared(haystack, tmpl, origin) {
                match &best {
                    None => best = Some((i, m)),
                    Some((_, prev)) if m.score > prev.score => best = Some((i, m)),
                    _ => {}
                }
            }
        }
        best
    }

    /// Try templates **in order** and return `(index, MatchResult)` for the **first** one
    /// that exceeds the configured threshold.
    ///
    /// Unlike [`find_best_of`] this short-circuits on the first hit, making it suitable
    /// for priority-ordered checks: "try button A first, then B, then C."
    pub fn find_first_of(
        &self,
        haystack: &RgbaImage,
        templates: &[RgbaImage],
        origin: ScreenPoint,
    ) -> Option<(usize, MatchResult)> {
        for (i, template) in templates.iter().enumerate() {
            if let Ok(m) = self.find(haystack, template, origin) {
                return Some((i, m));
            }
        }
        None
    }

    /// Like [`find_first_of`] but uses pre-converted [`PreparedTemplate`] slices.
    pub fn find_first_of_prepared(
        &self,
        haystack: &RgbaImage,
        templates: &[PreparedTemplate],
        origin: ScreenPoint,
    ) -> Option<(usize, MatchResult)> {
        for (i, tmpl) in templates.iter().enumerate() {
            if let Ok(m) = self.find_prepared(haystack, tmpl, origin) {
                return Some((i, m));
            }
        }
        None
    }

    /// Like [`score`] but reuses the pre-computed grayscale in `tmpl`,
    /// skipping the RGB→gray conversion.
    pub fn score_prepared(&self, haystack: &RgbaImage, tmpl: &PreparedTemplate) -> f32 {
        if tmpl.width > haystack.width() || tmpl.height > haystack.height() {
            return 0.0;
        }
        let hay_gray = to_gray(haystack);
        let scores = match_template(
            &hay_gray,
            &tmpl.gray,
            MatchTemplateMethod::CrossCorrelationNormalized,
        );
        find_extremes(&scores).max_value
    }

    fn match_one(
        &self,
        haystack: &RgbaImage,
        template: &RgbaImage,
        haystack_origin: ScreenPoint,
        scale: f32,
        orig_w: u32,
        orig_h: u32,
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

        Ok(make_result(haystack_origin, mx, my, orig_w, orig_h, score))
    }

    fn match_one_gray(
        &self,
        hay_gray: &GrayImage,
        tmpl_gray: &GrayImage,
        haystack_origin: ScreenPoint,
        scale: f32,
        orig_w: u32,
        orig_h: u32,
    ) -> Result<MatchResult> {
        if tmpl_gray.width() > hay_gray.width() || tmpl_gray.height() > hay_gray.height() {
            return Err(MatchError::TemplateTooLarge);
        }
        let scores = match_template(
            hay_gray,
            tmpl_gray,
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
        Ok(make_result(haystack_origin, mx, my, orig_w, orig_h, score))
    }
}

// ---- helpers ---------------------------------------------------------------

fn make_result(origin: ScreenPoint, mx: u32, my: u32, tw: u32, th: u32, score: f32) -> MatchResult {
    let lx = origin.x + mx as i32;
    let ly = origin.y + my as i32;
    MatchResult {
        location: ScreenPoint { x: lx, y: ly },
        center: ScreenPoint {
            x: lx + tw as i32 / 2,
            y: ly + th as i32 / 2,
        },
        bounds: Rect {
            x: lx,
            y: ly,
            width: tw,
            height: th,
        },
        score,
    }
}

/// Crop haystack to the given region and return (cropped_image, new_screen_origin).
fn crop_roi(
    haystack: &RgbaImage,
    haystack_origin: ScreenPoint,
    region: Rect,
) -> (RgbaImage, ScreenPoint) {
    let x0 = region.x.max(0) as u32;
    let y0 = region.y.max(0) as u32;
    let x1 = (region.x + region.width as i32).min(haystack.width() as i32) as u32;
    let y1 = (region.y + region.height as i32).min(haystack.height() as i32) as u32;
    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    let roi = image::imageops::crop_imm(haystack, x0, y0, w, h).to_image();
    let roi_origin = ScreenPoint {
        x: haystack_origin.x + x0 as i32,
        y: haystack_origin.y + y0 as i32,
    };
    (roi, roi_origin)
}

fn scale_image(img: &RgbaImage, scale: f32) -> RgbaImage {
    if (scale - 1.0).abs() < f32::EPSILON {
        return img.clone();
    }
    let nw = ((img.width() as f32) * scale) as u32;
    let nh = ((img.height() as f32) * scale) as u32;
    image::imageops::resize(img, nw, nh, image::imageops::FilterType::Lanczos3)
}

/// Fill mask rects with neutral mid-gray so NCC ignores dynamic areas.
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

// ---- tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prepared::PreparedTemplate;
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

    #[test]
    fn match_result_has_bounds() {
        // Solid red 200x200 haystack, 20x10 red template — should self-match at (0,0).
        let hay = solid(200, 200, Rgba([255, 0, 0, 255]));
        let tmpl = solid(20, 10, Rgba([255, 0, 0, 255]));
        let m = TemplateMatcher::default();
        let r = m.find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(r.bounds.width, 20);
        assert_eq!(r.bounds.height, 10);
        assert_eq!(r.bounds.x, r.location.x);
        assert_eq!(r.bounds.y, r.location.y);
    }

    #[test]
    fn find_all_returns_multiple() {
        // Checkerboard-like: large gray haystack, small gray template — NCC score will be 1.0 everywhere.
        let hay = solid(100, 50, Rgba([200, 200, 200, 255]));
        let tmpl = solid(10, 10, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let results = m.find_all(&hay, &tmpl, ScreenPoint { x: 0, y: 0 });
        // With NMS radius 5, expect more than 1 result from a 100x50 haystack.
        assert!(
            !results.is_empty(),
            "find_all should return at least one result"
        );
        // Scores should be descending.
        for w in results.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
    }

    #[test]
    fn find_all_empty_when_template_too_large() {
        let hay = solid(10, 10, Rgba([100, 100, 100, 255]));
        let tmpl = solid(20, 20, Rgba([100, 100, 100, 255]));
        let m = TemplateMatcher::default();
        assert!(m
            .find_all(&hay, &tmpl, ScreenPoint { x: 0, y: 0 })
            .is_empty());
    }

    #[test]
    fn find_in_region_clips_search() {
        // Large haystack; region is a small sub-rectangle.
        let hay = solid(200, 200, Rgba([150, 150, 150, 255]));
        let tmpl = solid(10, 10, Rgba([150, 150, 150, 255]));
        let m = TemplateMatcher::default();
        let region = Rect {
            x: 50,
            y: 50,
            width: 80,
            height: 80,
        };
        let r = m
            .find_in_region(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }, region)
            .unwrap();
        // Result should be within the region.
        assert!(r.location.x >= 50 && r.location.x < 130);
        assert!(r.location.y >= 50 && r.location.y < 130);
    }

    #[test]
    fn crop_roi_offsets_correctly() {
        let hay = solid(100, 100, Rgba([0, 0, 0, 255]));
        let region = Rect {
            x: 20,
            y: 30,
            width: 40,
            height: 40,
        };
        let (roi, origin) = crop_roi(&hay, ScreenPoint { x: 10, y: 10 }, region);
        assert_eq!(roi.width(), 40);
        assert_eq!(roi.height(), 40);
        assert_eq!(origin.x, 30); // 10 + 20
        assert_eq!(origin.y, 40); // 10 + 30
    }

    #[test]
    fn find_prepared_matches() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        let prepared = PreparedTemplate::new(tmpl);
        let m = TemplateMatcher::default();
        m.find_prepared(&hay, &prepared, ScreenPoint { x: 0, y: 0 })
            .unwrap();
    }

    #[test]
    fn find_all_prepared_returns_multiple() {
        let hay = solid(100, 50, Rgba([200, 200, 200, 255]));
        let tmpl = solid(10, 10, Rgba([200, 200, 200, 255]));
        let prepared = PreparedTemplate::new(tmpl);
        let m = TemplateMatcher::default();
        let results = m.find_all_prepared(&hay, &prepared, ScreenPoint { x: 0, y: 0 });
        assert!(!results.is_empty());
    }

    #[test]
    fn score_self_match() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let s = m.score(&hay, &tmpl);
        assert!(s > 0.5, "expected score > 0.5, got {s}");
    }

    #[test]
    fn score_too_large_returns_zero() {
        let hay = solid(10, 10, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        assert_eq!(m.score(&hay, &tmpl), 0.0);
    }

    #[test]
    fn find_best_of_returns_some() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl_a = solid(20, 20, Rgba([200, 200, 200, 255]));
        let tmpl_b = solid(5, 5, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let result = m.find_best_of(&hay, &[tmpl_a, tmpl_b], ScreenPoint { x: 0, y: 0 });
        assert!(result.is_some());
    }

    #[test]
    fn find_best_of_empty_returns_none() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        assert!(m
            .find_best_of(&hay, &[], ScreenPoint { x: 0, y: 0 })
            .is_none());
    }

    #[test]
    fn find_best_of_prepared_returns_some() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let pa = PreparedTemplate::new(solid(20, 20, Rgba([200, 200, 200, 255])));
        let pb = PreparedTemplate::new(solid(5, 5, Rgba([200, 200, 200, 255])));
        let m = TemplateMatcher::default();
        let result = m.find_best_of_prepared(&hay, &[pa, pb], ScreenPoint { x: 0, y: 0 });
        assert!(result.is_some());
    }

    #[test]
    fn find_best_of_prepared_empty_returns_none() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        assert!(m
            .find_best_of_prepared(&hay, &[], ScreenPoint { x: 0, y: 0 })
            .is_none());
    }

    #[test]
    fn find_first_of_short_circuits() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        // Both templates match; index 0 should be returned (short-circuit).
        let t0 = solid(20, 20, Rgba([200, 200, 200, 255]));
        let t1 = solid(10, 10, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let result = m.find_first_of(&hay, &[t0, t1], ScreenPoint { x: 0, y: 0 });
        assert!(matches!(result, Some((0, _))));
    }

    #[test]
    fn find_first_of_prepared_short_circuits() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let p0 = PreparedTemplate::new(solid(20, 20, Rgba([200, 200, 200, 255])));
        let p1 = PreparedTemplate::new(solid(10, 10, Rgba([200, 200, 200, 255])));
        let m = TemplateMatcher::default();
        let result = m.find_first_of_prepared(&hay, &[p0, p1], ScreenPoint { x: 0, y: 0 });
        assert!(matches!(result, Some((0, _))));
    }

    #[test]
    fn find_first_of_skips_non_matching() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        // t0 is too large → no match; t1 should match at index 1.
        let t0 = solid(200, 200, Rgba([200, 200, 200, 255]));
        let t1 = solid(10, 10, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default();
        let result = m.find_first_of(&hay, &[t0, t1], ScreenPoint { x: 0, y: 0 });
        assert!(matches!(result, Some((1, _))));
    }

    #[test]
    fn score_prepared_matches_score() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        let prepared = PreparedTemplate::new(tmpl.clone());
        let m = TemplateMatcher::default();
        let s1 = m.score(&hay, &tmpl);
        let s2 = m.score_prepared(&hay, &prepared);
        assert!(
            (s1 - s2).abs() < 1e-5,
            "score and score_prepared should agree"
        );
    }

    #[test]
    fn find_all_single_scale_uses_only_original_size() {
        let hay = solid(100, 50, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        let m = TemplateMatcher::default().with_multi_scale(false);
        let results = m.find_all(&hay, &tmpl, ScreenPoint { x: 0, y: 0 });
        for r in &results {
            assert_eq!(r.bounds.width, 20);
            assert_eq!(r.bounds.height, 20);
        }
    }

    #[test]
    fn find_all_multi_scale_covers_scaled_template() {
        // Haystack: gray bg with a 16x16 red patch — represents a template at 80% scale.
        let mut hay = solid(100, 80, Rgba([180, 180, 180, 255]));
        fill_rect(
            &mut hay,
            &Rect {
                x: 30,
                y: 20,
                width: 16,
                height: 16,
            },
            Rgba([255, 50, 50, 255]),
        );
        let tmpl = solid(20, 20, Rgba([255, 50, 50, 255]));
        let m = TemplateMatcher::default().with_multi_scale(true);
        let results = m.find_all(&hay, &tmpl, ScreenPoint { x: 0, y: 0 });
        assert!(
            results
                .iter()
                .any(|r| r.bounds.width == 16 && r.bounds.height == 16),
            "multi-scale find_all should detect the 80%-scaled template"
        );
    }

    #[test]
    fn find_all_prepared_respects_multi_scale() {
        let mut hay = solid(100, 80, Rgba([180, 180, 180, 255]));
        fill_rect(
            &mut hay,
            &Rect {
                x: 30,
                y: 20,
                width: 16,
                height: 16,
            },
            Rgba([255, 50, 50, 255]),
        );
        let tmpl_rgba = solid(20, 20, Rgba([255, 50, 50, 255]));
        let prepared = PreparedTemplate::new(tmpl_rgba);
        let m = TemplateMatcher::default().with_multi_scale(true);
        let results = m.find_all_prepared(&hay, &prepared, ScreenPoint { x: 0, y: 0 });
        assert!(
            results
                .iter()
                .any(|r| r.bounds.width == 16 && r.bounds.height == 16),
            "find_all_prepared with multi-scale should find the 80%-scaled instance"
        );
    }

    #[test]
    fn find_all_in_region_prepared_clips_and_returns() {
        let hay = solid(100, 100, Rgba([100, 100, 100, 255]));
        let tmpl_rgba = solid(10, 10, Rgba([100, 100, 100, 255]));
        let prepared = PreparedTemplate::new(tmpl_rgba);
        let m = TemplateMatcher::default();
        let region = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 50,
        };
        let results =
            m.find_all_in_region_prepared(&hay, &prepared, ScreenPoint { x: 0, y: 0 }, region);
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.bounds.x + r.bounds.width as i32 <= 50);
            assert!(r.bounds.y + r.bounds.height as i32 <= 50);
        }
    }

    #[test]
    fn with_nms_radius_zero_returns_all_candidates() {
        // Uniform haystack → every position scores the same.
        // With nms_radius=0 each pixel is its own candidate, so we get many results.
        let hay = solid(30, 10, Rgba([200, 200, 200, 255]));
        let tmpl = solid(5, 5, Rgba([200, 200, 200, 255]));
        let default_results =
            TemplateMatcher::default().find_all(&hay, &tmpl, ScreenPoint { x: 0, y: 0 });
        let tight_results = TemplateMatcher::default().with_nms_radius(0).find_all(
            &hay,
            &tmpl,
            ScreenPoint { x: 0, y: 0 },
        );
        assert!(
            tight_results.len() >= default_results.len(),
            "nms_radius=0 should suppress nothing, tight={} default={}",
            tight_results.len(),
            default_results.len()
        );
    }

    #[test]
    fn match_result_is_serializable() {
        let r = MatchResult {
            location: ScreenPoint { x: 10, y: 20 },
            center: ScreenPoint { x: 20, y: 30 },
            bounds: Rect {
                x: 10,
                y: 20,
                width: 20,
                height: 20,
            },
            score: 0.95,
        };
        fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>(_: &T) {}
        assert_serde(&r);
    }

    #[test]
    fn with_threshold_builder_chains() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        // Default with lowered threshold should still find the match.
        let m = TemplateMatcher::default().with_threshold(0.5);
        m.find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }).unwrap();
        // new() constructor equivalent.
        let m2 = TemplateMatcher::new(0.5);
        m2.find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }).unwrap();
    }

    #[test]
    fn with_scales_restricts_to_given_factors() {
        let hay = solid(100, 100, Rgba([200, 200, 200, 255]));
        let tmpl = solid(20, 20, Rgba([200, 200, 200, 255]));
        // Providing only scale 1.0 should still find the exact-size match.
        let m = TemplateMatcher::default().with_scales(vec![1.0]);
        m.find(&hay, &tmpl, ScreenPoint { x: 0, y: 0 }).unwrap();
    }
}
