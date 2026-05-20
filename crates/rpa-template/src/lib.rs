use serde::{Deserialize, Serialize};

pub use image::RgbaImage;
pub use rpa_vision::{MaskRegion, Rect, ScreenPoint, WindowPoint};

/// Identifies what to capture or interact with.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    /// The entire primary screen.
    Screen,
    /// A specific screen by index.
    ScreenIndex { index: usize },
    /// Window matched by title substring.
    Window { title_contains: String },
    /// Window matched by process name.
    Process { name: String },
    /// Explicit screen region.
    Region(Rect),
}

/// Anchor point with pixel offset — used to define click coordinates relative to a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    /// Offset from the matched template's top-left in window-local coords.
    pub offset: WindowPoint,
    pub label: Option<String>,
}

/// Metadata saved alongside a captured template image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMeta {
    pub label: String,
    /// Relative path to the PNG file (same directory as the YAML).
    pub image_path: String,
    #[serde(default)]
    pub masks: Vec<MaskRegion>,
    #[serde(default)]
    pub anchors: Vec<Anchor>,
    /// DPI scale at capture time (e.g. 1.0, 1.25, 1.5).
    #[serde(default = "default_dpi_scale")]
    pub dpi_scale: f32,
}

fn default_dpi_scale() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains() {
        let r = Rect {
            x: 10,
            y: 10,
            width: 100,
            height: 50,
        };
        assert!(r.contains(ScreenPoint { x: 50, y: 30 }));
        assert!(!r.contains(ScreenPoint { x: 9, y: 30 }));
        assert!(!r.contains(ScreenPoint { x: 110, y: 30 }));
    }
}
