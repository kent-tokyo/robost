use crate::types::Rect;
use image::RgbaImage;
use std::collections::VecDeque;

/// Result of comparing two screen captures pixel-by-pixel.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffResult {
    /// Number of pixels that exceeded the change threshold.
    pub changed_pixels: u32,
    /// Fraction of compared pixels that changed (0.0–1.0).
    pub changed_ratio: f32,
    /// Bounding boxes of 8-connected groups of changed pixels.
    pub regions: Vec<Rect>,
}

impl DiffResult {
    /// Returns `true` if no pixels changed (i.e. the images are identical within the threshold).
    pub fn is_empty(&self) -> bool {
        self.changed_pixels == 0
    }

    /// Returns the largest changed region by area, or `None` if there are no regions.
    pub fn largest_region(&self) -> Option<&Rect> {
        self.regions.iter().max_by_key(|r| r.area())
    }

    /// Returns the smallest rectangle that contains all changed regions, or `None` if empty.
    pub fn bounding_box(&self) -> Option<Rect> {
        let mut iter = self.regions.iter();
        let first = *iter.next()?;
        Some(iter.fold(first, |acc, r| acc.union(*r)))
    }
}

/// Compare two RGBA images and return changed-pixel statistics plus region clusters.
///
/// A pixel is "changed" when the maximum per-channel RGB difference is at or above
/// `pixel_threshold`. Alpha is ignored.
///
/// When the images have different dimensions only the overlapping area is compared.
///
/// # Example
///
/// ```no_run
/// use robost_vision::diff;
/// use image::open;
///
/// let before = open("before.png").unwrap().into_rgba8();
/// let after  = open("after.png").unwrap().into_rgba8();
/// let result = diff(&before, &after, 15);
/// println!("changed {:.1}%  in {} region(s)", result.changed_ratio * 100.0, result.regions.len());
/// ```
pub fn diff(img_a: &RgbaImage, img_b: &RgbaImage, pixel_threshold: u8) -> DiffResult {
    let w = img_a.width().min(img_b.width());
    let h = img_a.height().min(img_b.height());
    let total = (w * h) as usize;

    if total == 0 {
        return DiffResult {
            changed_pixels: 0,
            changed_ratio: 0.0,
            regions: Vec::new(),
        };
    }

    // Build changed-pixel boolean map
    let mut changed = vec![false; total];
    let mut changed_count = 0u32;

    for y in 0..h {
        for x in 0..w {
            let pa = img_a.get_pixel(x, y);
            let pb = img_b.get_pixel(x, y);
            let dr = pa[0].abs_diff(pb[0]);
            let dg = pa[1].abs_diff(pb[1]);
            let db = pa[2].abs_diff(pb[2]);
            if dr.max(dg).max(db) >= pixel_threshold {
                changed[(y * w + x) as usize] = true;
                changed_count += 1;
            }
        }
    }

    // BFS to find 8-connected components; each component yields one bounding Rect.
    let mut visited = vec![false; total];
    let mut regions: Vec<Rect> = Vec::new();

    for sy in 0..h {
        for sx in 0..w {
            let idx = (sy * w + sx) as usize;
            if !changed[idx] || visited[idx] {
                continue;
            }
            let mut queue = VecDeque::new();
            queue.push_back((sx, sy));
            visited[idx] = true;
            let (mut min_x, mut max_x) = (sx, sx);
            let (mut min_y, mut max_y) = (sy, sy);

            while let Some((cx, cy)) = queue.pop_front() {
                min_x = min_x.min(cx);
                max_x = max_x.max(cx);
                min_y = min_y.min(cy);
                max_y = max_y.max(cy);

                let nx0 = cx.saturating_sub(1);
                let nx1 = (cx + 1).min(w - 1);
                let ny0 = cy.saturating_sub(1);
                let ny1 = (cy + 1).min(h - 1);

                for ny in ny0..=ny1 {
                    for nx in nx0..=nx1 {
                        if nx == cx && ny == cy {
                            continue;
                        }
                        let nidx = (ny * w + nx) as usize;
                        if changed[nidx] && !visited[nidx] {
                            visited[nidx] = true;
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }

            regions.push(Rect {
                x: min_x as i32,
                y: min_y as i32,
                width: max_x - min_x + 1,
                height: max_y - min_y + 1,
            });
        }
    }

    DiffResult {
        changed_pixels: changed_count,
        changed_ratio: changed_count as f32 / total as f32,
        regions,
    }
}

/// Like [`diff`] but compares only the pixels inside `region`.
///
/// `region` is in the same coordinate space as the images (image-local, typically
/// screen-global). Returned `DiffResult.regions` are offset to screen-global coordinates
/// so callers can use them directly without further adjustment.
///
/// Only the overlapping area of `region` and both images is compared.
pub fn diff_in_region(
    img_a: &RgbaImage,
    img_b: &RgbaImage,
    region: Rect,
    pixel_threshold: u8,
) -> DiffResult {
    let max_w = img_a.width().min(img_b.width()) as i32;
    let max_h = img_a.height().min(img_b.height()) as i32;

    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.width as i32).min(max_w);
    let y1 = (region.y + region.height as i32).min(max_h);

    if x1 <= x0 || y1 <= y0 {
        return DiffResult {
            changed_pixels: 0,
            changed_ratio: 0.0,
            regions: Vec::new(),
        };
    }

    let crop_w = (x1 - x0) as u32;
    let crop_h = (y1 - y0) as u32;

    let crop_a = image::imageops::crop_imm(img_a, x0 as u32, y0 as u32, crop_w, crop_h).to_image();
    let crop_b = image::imageops::crop_imm(img_b, x0 as u32, y0 as u32, crop_w, crop_h).to_image();

    let mut result = diff(&crop_a, &crop_b, pixel_threshold);

    // Offset regions from crop-local to screen-global coordinates.
    for r in &mut result.regions {
        r.x += x0;
        r.y += y0;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn solid(w: u32, h: u32, c: Rgba<u8>) -> RgbaImage {
        RgbaImage::from_pixel(w, h, c)
    }

    #[test]
    fn identical_images_no_change() {
        let img = solid(10, 10, Rgba([100, 100, 100, 255]));
        let r = diff(&img, &img, 10);
        assert_eq!(r.changed_pixels, 0);
        assert_eq!(r.changed_ratio, 0.0);
        assert!(r.regions.is_empty());
    }

    #[test]
    fn fully_changed_image() {
        let a = solid(10, 10, Rgba([0, 0, 0, 255]));
        let b = solid(10, 10, Rgba([255, 255, 255, 255]));
        let r = diff(&a, &b, 10);
        assert_eq!(r.changed_pixels, 100);
        assert!((r.changed_ratio - 1.0).abs() < 1e-6);
        assert_eq!(r.regions.len(), 1);
        assert_eq!(r.regions[0].width, 10);
        assert_eq!(r.regions[0].height, 10);
    }

    #[test]
    fn two_separate_regions() {
        let a = solid(20, 10, Rgba([100, 100, 100, 255]));
        let mut b = solid(20, 10, Rgba([100, 100, 100, 255]));
        // Patch top-left 2×2
        for y in 0..2u32 {
            for x in 0..2u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        // Patch top-right 3×2 (well separated)
        for y in 0..2u32 {
            for x in 17..20u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let r = diff(&a, &b, 10);
        assert_eq!(r.changed_pixels, 10); // 4 + 6
        assert_eq!(r.regions.len(), 2);
    }

    #[test]
    fn threshold_filters_small_changes() {
        let a = solid(10, 10, Rgba([100, 100, 100, 255]));
        let b = solid(10, 10, Rgba([105, 105, 105, 255])); // max channel diff = 5
        assert_eq!(diff(&a, &b, 10).changed_pixels, 0); // threshold 10 → no change
        assert_eq!(diff(&a, &b, 5).changed_pixels, 100); // threshold 5 → all changed
    }

    #[test]
    fn diff_in_region_screen_global_coords() {
        let a = solid(100, 100, Rgba([100, 100, 100, 255]));
        let mut b = solid(100, 100, Rgba([100, 100, 100, 255]));
        // Change a 5×5 block at (50, 60)
        for y in 60..65u32 {
            for x in 50..55u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let region = Rect {
            x: 40,
            y: 50,
            width: 30,
            height: 30,
        };
        let r = diff_in_region(&a, &b, region, 10);
        assert_eq!(r.changed_pixels, 25);
        assert_eq!(r.regions.len(), 1);
        // Region x/y must be screen-global (offset by 40, 50 is already handled inside
        // the crop, so the returned rect should start at 50, 60).
        assert_eq!(r.regions[0].x, 50);
        assert_eq!(r.regions[0].y, 60);
    }

    #[test]
    fn diff_in_region_outside_image_returns_empty() {
        let a = solid(10, 10, Rgba([0, 0, 0, 255]));
        let b = solid(10, 10, Rgba([255, 255, 255, 255]));
        // Region entirely outside the 10×10 image
        let region = Rect {
            x: 20,
            y: 20,
            width: 5,
            height: 5,
        };
        let r = diff_in_region(&a, &b, region, 1);
        assert_eq!(r.changed_pixels, 0);
    }

    #[test]
    fn empty_overlap_returns_empty() {
        let a = solid(0, 0, Rgba([0, 0, 0, 255]));
        let b = solid(5, 5, Rgba([255, 255, 255, 255]));
        let r = diff(&a, &b, 1);
        assert_eq!(r.changed_pixels, 0);
        assert!(r.regions.is_empty());
    }

    #[test]
    fn is_empty_reflects_changed_pixels() {
        let img = solid(10, 10, Rgba([100, 100, 100, 255]));
        assert!(diff(&img, &img, 1).is_empty());
        let other = solid(10, 10, Rgba([200, 200, 200, 255]));
        assert!(!diff(&img, &other, 1).is_empty());
    }

    #[test]
    fn largest_region_returns_biggest() {
        let a = solid(20, 10, Rgba([100, 100, 100, 255]));
        let mut b = solid(20, 10, Rgba([100, 100, 100, 255]));
        // 2×2 patch at (0,0) — area 4
        for y in 0..2u32 {
            for x in 0..2u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        // 3×3 patch at (15,0) — area 9
        for y in 0..3u32 {
            for x in 15..18u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let r = diff(&a, &b, 10);
        let largest = r.largest_region().unwrap();
        assert_eq!(largest.area(), 9);
    }

    #[test]
    fn bounding_box_covers_all_regions() {
        let a = solid(20, 20, Rgba([100, 100, 100, 255]));
        let mut b = solid(20, 20, Rgba([100, 100, 100, 255]));
        // patch at top-left (0,0) 2×2
        for y in 0..2u32 {
            for x in 0..2u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        // patch at bottom-right (18,18) 2×2
        for y in 18..20u32 {
            for x in 18..20u32 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let r = diff(&a, &b, 10);
        let bb = r.bounding_box().unwrap();
        assert_eq!(bb.x, 0);
        assert_eq!(bb.y, 0);
        assert_eq!(bb.x + bb.width as i32, 20);
        assert_eq!(bb.y + bb.height as i32, 20);
    }

    #[test]
    fn bounding_box_empty_returns_none() {
        let img = solid(10, 10, Rgba([100, 100, 100, 255]));
        assert!(diff(&img, &img, 1).bounding_box().is_none());
    }
}
