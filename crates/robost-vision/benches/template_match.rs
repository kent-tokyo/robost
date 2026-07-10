//! Benchmarks quantifying the cost of multi-scale NCC matching (the DPI-resilience
//! feature) versus single-scale matching, plus `find_all`'s non-max-suppression cost
//! when a haystack contains several occurrences of the template.
//!
//! Uses the same synthetic-image generation approach as `examples/find_all_example.rs`
//! so no binary image fixtures are needed. Run with `cargo bench -p robost-vision`.

use criterion::{criterion_group, criterion_main, Criterion};
use image::{Rgba, RgbaImage};
use robost_vision::{ScreenPoint, TemplateMatcher};
use std::hint::black_box;
use std::time::Duration;

/// Non-uniform template (left half red, right half blue) so NCC has real structure to match,
/// not a flat color that would match everywhere.
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

/// A 480x270 haystack — representative of matching within a single window/region
/// rather than a full-screen scan — with a single template occurrence at native scale.
fn single_match_haystack(template: &RgbaImage) -> RgbaImage {
    let mut haystack = RgbaImage::from_pixel(480, 270, Rgba([210, 210, 210, 255]));
    place(&mut haystack, template, 220, 115);
    haystack
}

/// Same haystack, but the occurrence is rendered at 80% of the template's size —
/// simulating a UI captured at a different DPI/scale than the template. `find`'s
/// scale loop returns on the first scale clearing the threshold, so a single-scale
/// matcher would simply fail to find this; a multi-scale matcher must try (and pay
/// for) the 1.0 scale attempt before succeeding at 0.8.
fn scaled_match_haystack(template: &RgbaImage) -> RgbaImage {
    let scaled = image::imageops::resize(
        template,
        (template.width() as f32 * 0.8) as u32,
        (template.height() as f32 * 0.8) as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let mut haystack = RgbaImage::from_pixel(480, 270, Rgba([210, 210, 210, 255]));
    place(&mut haystack, &scaled, 220, 115);
    haystack
}

/// Same haystack, but with the template placed at five different locations,
/// to exercise `find_all`'s O(n^2) non-max-suppression pass.
fn multi_match_haystack(template: &RgbaImage) -> RgbaImage {
    let mut haystack = RgbaImage::from_pixel(480, 270, Rgba([210, 210, 210, 255]));
    for (ox, oy) in [(20, 20), (150, 60), (220, 115), (350, 30), (420, 200)] {
        place(&mut haystack, template, ox, oy);
    }
    haystack
}

fn bench_find_single_scale(c: &mut Criterion) {
    let template = make_template(40, 40);
    let haystack = single_match_haystack(&template);
    let matcher = TemplateMatcher::default().with_multi_scale(false);

    c.bench_function("find_single_scale", |b| {
        b.iter(|| {
            matcher
                .find(
                    black_box(&haystack),
                    black_box(&template),
                    ScreenPoint { x: 0, y: 0 },
                )
                .unwrap()
        })
    });
}

fn bench_find_multi_scale(c: &mut Criterion) {
    let template = make_template(40, 40);
    let haystack = scaled_match_haystack(&template);
    // A high threshold ensures the mis-sized scale-1.0 attempt (40x40 window over a
    // 32x32 patch plus border) doesn't clear it, forcing a real fallthrough to 0.8 —
    // without this, NCC's tolerance for this simple two-color template can pass at
    // scale 1.0 anyway, silently skipping the second scale attempt we want to measure.
    let matcher = TemplateMatcher::default()
        .with_multi_scale(true)
        .with_threshold(0.99);

    c.bench_function("find_multi_scale", |b| {
        b.iter(|| {
            matcher
                .find(
                    black_box(&haystack),
                    black_box(&template),
                    ScreenPoint { x: 0, y: 0 },
                )
                .unwrap()
        })
    });
}

fn bench_find_all_multiple_matches(c: &mut Criterion) {
    let template = make_template(40, 40);
    let haystack = multi_match_haystack(&template);
    let matcher = TemplateMatcher::default()
        .with_multi_scale(true)
        .with_threshold(0.9);

    c.bench_function("find_all_multiple_matches", |b| {
        b.iter(|| {
            matcher.find_all(
                black_box(&haystack),
                black_box(&template),
                ScreenPoint { x: 0, y: 0 },
            )
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(3));
    targets = bench_find_single_scale, bench_find_multi_scale, bench_find_all_multiple_matches
}
criterion_main!(benches);
