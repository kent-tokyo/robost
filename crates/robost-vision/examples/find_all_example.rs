//! Demonstrates finding multiple occurrences of a template, including at a different scale.
//!
//! Run: `cargo run --example find_all_example`

use image::{Rgba, RgbaImage};
use robost_vision::{ScreenPoint, TemplateMatcher};

/// Non-uniform template: left half red, right half blue.
fn make_template(w: u32, h: u32) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let color = if x < w / 2 {
                Rgba([200, 50, 50, 255])
            } else {
                Rgba([50, 50, 200, 255])
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
    // Template: 20×20 half-red/half-blue.
    let template = make_template(20, 20);
    // Scaled-down version (80 % = 16×16) produced by shrinking the template.
    let template16 =
        image::imageops::resize(&template, 16, 16, image::imageops::FilterType::Lanczos3);

    // Haystack: gray bg with:
    //   • full-size (20×20) patch at (10, 10) and (60, 40)
    //   • 80%-sized (16×16) patch at (130, 70)
    let mut haystack = RgbaImage::from_pixel(200, 120, Rgba([210, 210, 210, 255]));
    place(&mut haystack, &template, 10, 10);
    place(&mut haystack, &template, 60, 40);
    place(&mut haystack, &template16, 130, 70);

    // Single-scale: finds full-size matches only.
    // Threshold 0.99 narrows results to near-perfect matches for a clean demo output.
    let single = TemplateMatcher::default()
        .with_multi_scale(false)
        .with_threshold(0.99);
    let results_single = single.find_all(&haystack, &template, ScreenPoint { x: 0, y: 0 });
    println!("Single-scale results: {}", results_single.len());
    for r in &results_single {
        println!(
            "  {:?}  size {}×{}",
            r.location, r.bounds.width, r.bounds.height
        );
    }

    // Multi-scale: also detects the 80%-sized patch.
    let multi = TemplateMatcher::default()
        .with_multi_scale(true)
        .with_threshold(0.99);
    let results_multi = multi.find_all(&haystack, &template, ScreenPoint { x: 0, y: 0 });
    println!("\nMulti-scale results: {}", results_multi.len());
    for r in &results_multi {
        println!(
            "  {:?}  size {}×{}",
            r.location, r.bounds.width, r.bounds.height
        );
    }

    let has_small = results_multi.iter().any(|r| r.bounds.width == 16);
    println!("\n80%-scaled patch detected by multi-scale: {has_small}");
}
