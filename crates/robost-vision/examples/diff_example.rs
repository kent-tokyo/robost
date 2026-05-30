//! Demonstrates pixel-level image diffing to detect UI changes.
//!
//! Run: `cargo run --example diff_example`

use image::{Rgba, RgbaImage};
use robost_vision::diff::{diff, diff_in_region};
use robost_vision::Rect;

fn solid(w: u32, h: u32, color: Rgba<u8>) -> RgbaImage {
    RgbaImage::from_pixel(w, h, color)
}

fn main() {
    let before = solid(200, 150, Rgba([230, 230, 230, 255]));

    // Simulate a UI change: a dialog appears in the bottom-right quadrant.
    let mut after = before.clone();
    let dialog = solid(60, 40, Rgba([255, 255, 255, 255]));
    image::imageops::replace(&mut after, &dialog, 130, 100);

    let result = diff(&before, &after, 10);
    println!("Changed pixels : {}", result.changed_pixels);
    println!("Changed ratio  : {:.1}%", result.changed_ratio * 100.0);
    println!("Changed regions: {}", result.regions.len());
    if let Some(bb) = result.bounding_box() {
        println!("Bounding box   : {:?}", bb);
    }

    // Narrow the search to only the right half of the screen.
    let right_half = Rect {
        x: 100,
        y: 0,
        width: 100,
        height: 150,
    };
    let region_result = diff_in_region(&before, &after, right_half, 10);
    println!(
        "\nRight-half diff: {} changed pixels, {} regions",
        region_result.changed_pixels,
        region_result.regions.len()
    );
}
