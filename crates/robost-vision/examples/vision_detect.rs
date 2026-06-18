#[cfg(all(feature = "ml-vision", target_os = "macos"))]
fn main() {
    use robost_vision::ml::vision::VisionDetector;
    use robost_vision::{load_rgba, ScreenPoint};

    let path = std::env::args().nth(1).unwrap_or_else(|| "/tmp/screen_test.png".into());
    let query = std::env::args().nth(2).unwrap_or_default();

    let img = load_rgba(&path).expect("load image");
    let detector = VisionDetector::new();
    let origin = ScreenPoint { x: 0, y: 0 };

    let boxes = detector.detect(&img, origin).expect("detect");
    println!("Found {} text boxes", boxes.len());
    for b in &boxes {
        println!("  ({:4},{:4}) {:3}×{:3}  conf={:.2}  {:?}",
            b.bounds.x, b.bounds.y, b.bounds.width, b.bounds.height,
            b.confidence, b.text);
    }

    if !query.is_empty() {
        match boxes.iter().find(|b| b.text.contains(&query)) {
            Some(b) => println!("\n[FOUND] {:?} → center ({}, {})",
                query, b.bounds.x + b.bounds.width as i32 / 2,
                b.bounds.y + b.bounds.height as i32 / 2),
            None => println!("\n[NOT FOUND] {:?}", query),
        }
    }
}

#[cfg(not(all(feature = "ml-vision", target_os = "macos")))]
fn main() {
    eprintln!("Requires --features ml-vision on macOS");
}
