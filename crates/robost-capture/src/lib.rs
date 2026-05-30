use image::RgbaImage;
use robost_template::{Rect, ScreenPoint, Target};
use thiserror::Error;
use tracing::instrument;
use xcap::{Monitor, Window};

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("xcap error: {0}")]
    Xcap(#[from] xcap::XCapError),
    #[error("no monitor found")]
    NoMonitor,
    #[error("window not found: {0}")]
    WindowNotFound(String),
    #[error("region out of bounds")]
    RegionOutOfBounds,
}

pub type Result<T> = std::result::Result<T, CaptureError>;

/// Initialise DPI awareness. Must be called once at startup on Windows.
pub fn init_dpi() {
    #[cfg(windows)]
    {
        use windows::Win32::UI::HiDpi::{SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE};
        unsafe {
            let _ = SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
        }
    }
}

/// Capture the primary monitor.
#[instrument(name = "capture_screen")]
pub fn capture_screen() -> Result<RgbaImage> {
    let monitors = Monitor::all()?;
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .ok_or(CaptureError::NoMonitor)?;
    Ok(monitor.capture_image()?)
}

/// Capture a specific monitor by index.
#[instrument(name = "capture_screen_index", fields(index))]
pub fn capture_screen_index(index: usize) -> Result<RgbaImage> {
    let monitors = Monitor::all()?;
    let monitor = monitors
        .into_iter()
        .nth(index)
        .ok_or(CaptureError::NoMonitor)?;
    Ok(monitor.capture_image()?)
}

fn find_window(title_contains: &str) -> Result<Window> {
    Window::all()?
        .into_iter()
        .find(|w| {
            w.title()
                .map(|t| t.contains(title_contains))
                .unwrap_or(false)
        })
        .ok_or_else(|| CaptureError::WindowNotFound(title_contains.to_owned()))
}

/// Capture a window whose title contains `title_contains`.
#[instrument(name = "capture_window", fields(title_contains))]
pub fn capture_window(title_contains: &str) -> Result<RgbaImage> {
    Ok(find_window(title_contains)?.capture_image()?)
}

/// Return the screen-global top-left corner of a window.
/// Used as `haystack_origin` so template match results are already screen-global.
pub fn window_origin(title_contains: &str) -> Result<ScreenPoint> {
    let w = find_window(title_contains)?;
    Ok(ScreenPoint {
        x: w.x().unwrap_or(0),
        y: w.y().unwrap_or(0),
    })
}

/// Capture using a [`Target`].
pub fn capture(target: &Target) -> Result<RgbaImage> {
    match target {
        Target::Screen => capture_screen(),
        Target::ScreenIndex { index } => capture_screen_index(*index),
        Target::Window { title_contains } => capture_window(title_contains),
        Target::Process { name } => capture_window(name),
        Target::Region(rect) => capture_region(rect),
        Target::WindowClass { class_name } => capture_window_class(class_name),
    }
}

/// Capture a window identified by its Win32 class name.
/// On non-Windows platforms, falls back to the full screen capture.
fn capture_window_class(class_name: &str) -> Result<RgbaImage> {
    #[cfg(target_os = "windows")]
    {
        // Find the first top-level window with the given class name via xcap title search.
        // xcap does not expose class names directly, so we enumerate all windows and
        // fall back to the title-based search using the class_name string as a title fragment.
        // Prefer the class_name as the search key; behaviour is best-effort on Windows.
        capture_window(class_name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = class_name;
        capture_screen()
    }
}

/// Return the RGB colour of the pixel at `(x, y)` in an already-captured image.
///
/// Coordinates are relative to the image's top-left corner.
/// Returns `None` when the coordinates are out of bounds.
pub fn pixel_at(img: &RgbaImage, x: u32, y: u32) -> Option<(u8, u8, u8)> {
    if x < img.width() && y < img.height() {
        let p = img.get_pixel(x, y);
        Some((p[0], p[1], p[2]))
    } else {
        None
    }
}

fn capture_region(rect: &Rect) -> Result<RgbaImage> {
    let full = capture_screen()?;
    let x = rect.x.max(0) as u32;
    let y = rect.y.max(0) as u32;
    if x + rect.width > full.width() || y + rect.height > full.height() {
        return Err(CaptureError::RegionOutOfBounds);
    }
    Ok(image::imageops::crop_imm(&full, x, y, rect.width, rect.height).to_image())
}
