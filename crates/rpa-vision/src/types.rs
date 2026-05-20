use serde::{Deserialize, Serialize};

/// Screen-global coordinate. Always absolute pixel coordinates on the display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

/// Window-local coordinate. Relative to the window's top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPoint {
    pub x: i32,
    pub y: i32,
}

/// Axis-aligned rectangle in screen-global coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn contains(&self, p: ScreenPoint) -> bool {
        p.x >= self.x
            && p.x < self.x + self.width as i32
            && p.y >= self.y
            && p.y < self.y + self.height as i32
    }
}

/// A mask region to exclude from template matching (e.g. timestamps, badges).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskRegion {
    pub rect: Rect,
    pub label: Option<String>,
}
