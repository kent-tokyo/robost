//! Demonstrates single-template matching with synthetic images.
//!
//! Run: `cargo run --example basic_match`

use image::{Rgba, RgbaImage};
use robost_vision::{MatchError, ScreenPoint, TemplateMatcher};

/// Create a non-uniform template: left half red, right half blue.
/// NCC requires texture to score uniquely; a solid color matches everywhere.
fn make_template(w: u32, h: u32) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let color = if x < w / 2 {
                Rgba([200, 50, 50, 255]) // red
            } else {
                Rgba([50, 50, 200, 255]) // blue
            };
            img.put_pixel(x, y, color);
        }
    }
    img
}

fn place(canvas: &mut RgbaImage, patch: &RgbaImage, ox: u32, oy: u32) {
    for (px, py, pixel) in patch.enumerate_pixels() {
        canvas.put_pixel(ox + px, oy + py, *pixel);
    }
}

fn main() {
    let template = make_template(30, 20);

    // Build a 200×150 gray haystack and embed the template at (80, 60).
    let mut haystack = RgbaImage::from_pixel(200, 150, Rgba([220, 220, 220, 255]));
    place(&mut haystack, &template, 80, 60);

    let matcher = TemplateMatcher::default(); // threshold 0.87, multi-scale on
    match matcher.find(&haystack, &template, ScreenPoint { x: 0, y: 0 }) {
        Ok(result) => {
            println!("Found at {:?} (score {:.3})", result.location, result.score);
            assert_eq!(result.location.x, 80);
            assert_eq!(result.location.y, 60);
            println!("OK — location matches expected (80, 60)");
        }
        Err(MatchError::BelowThreshold { score, threshold }) => {
            eprintln!("No match: best score {score:.3} < threshold {threshold:.2}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
