use serde::{Deserialize, Serialize};

/// Screen-global coordinate. Always absolute pixel coordinates on the display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

impl std::ops::Add for ScreenPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        ScreenPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for ScreenPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        ScreenPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<(i32, i32)> for ScreenPoint {
    fn from((x, y): (i32, i32)) -> Self {
        ScreenPoint { x, y }
    }
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
    /// Returns `true` if `p` is strictly inside this rectangle.
    pub fn contains(&self, p: ScreenPoint) -> bool {
        p.x >= self.x
            && p.x < self.x + self.width as i32
            && p.y >= self.y
            && p.y < self.y + self.height as i32
    }

    /// Returns the center point of this rectangle.
    pub fn center(&self) -> ScreenPoint {
        ScreenPoint {
            x: self.x + self.width as i32 / 2,
            y: self.y + self.height as i32 / 2,
        }
    }

    /// Total pixel area.
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Returns the intersection of two rectangles, or `None` if they don't overlap.
    pub fn intersection(&self, other: Rect) -> Option<Rect> {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = (self.x + self.width as i32).min(other.x + other.width as i32);
        let y1 = (self.y + self.height as i32).min(other.y + other.height as i32);
        if x1 > x0 && y1 > y0 {
            Some(Rect {
                x: x0,
                y: y0,
                width: (x1 - x0) as u32,
                height: (y1 - y0) as u32,
            })
        } else {
            None
        }
    }

    /// Returns the smallest rectangle that contains both `self` and `other`.
    pub fn union(&self, other: Rect) -> Rect {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.width as i32).max(other.x + other.width as i32);
        let y1 = (self.y + self.height as i32).max(other.y + other.height as i32);
        Rect {
            x: x0,
            y: y0,
            width: (x1 - x0) as u32,
            height: (y1 - y0) as u32,
        }
    }

    /// Expands all four sides by `padding` pixels (negative values shrink).
    pub fn expand(&self, padding: i32) -> Rect {
        let w = (self.width as i32 + padding * 2).max(0) as u32;
        let h = (self.height as i32 + padding * 2).max(0) as u32;
        Rect {
            x: self.x - padding,
            y: self.y - padding,
            width: w,
            height: h,
        }
    }

    /// Create a `Rect` from two corner points (order does not matter).
    pub fn from_corners(a: ScreenPoint, b: ScreenPoint) -> Self {
        let x = a.x.min(b.x);
        let y = a.y.min(b.y);
        let w = (a.x.max(b.x) - x) as u32;
        let h = (a.y.max(b.y) - y) as u32;
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    /// Create a `Rect` centered on `center` with the given dimensions.
    pub fn from_center_size(center: ScreenPoint, width: u32, height: u32) -> Self {
        Rect {
            x: center.x - width as i32 / 2,
            y: center.y - height as i32 / 2,
            width,
            height,
        }
    }

    /// Scales the rectangle around its center by `factor`.
    pub fn scale(&self, factor: f32) -> Rect {
        let cx = self.x + self.width as i32 / 2;
        let cy = self.y + self.height as i32 / 2;
        let nw = (self.width as f32 * factor).round() as u32;
        let nh = (self.height as f32 * factor).round() as u32;
        Rect {
            x: cx - nw as i32 / 2,
            y: cy - nh as i32 / 2,
            width: nw,
            height: nh,
        }
    }
}

/// Window-local coordinate. Relative to the window's top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPoint {
    pub x: i32,
    pub y: i32,
}

/// A mask region to exclude from template matching (e.g. timestamps, badges).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskRegion {
    pub rect: Rect,
    pub label: Option<String>,
}

impl MaskRegion {
    /// Create an unlabelled mask region.
    pub fn new(rect: Rect) -> Self {
        MaskRegion { rect, label: None }
    }

    /// Attach a human-readable label (e.g. `"timestamp"`) for debugging.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn rect_contains() {
        let rect = r(10, 10, 100, 50);
        assert!(rect.contains(ScreenPoint { x: 50, y: 30 }));
        assert!(!rect.contains(ScreenPoint { x: 9, y: 30 }));
        assert!(!rect.contains(ScreenPoint { x: 110, y: 30 }));
    }

    #[test]
    fn rect_center() {
        assert_eq!(r(0, 0, 100, 50).center(), ScreenPoint { x: 50, y: 25 });
    }

    #[test]
    fn rect_area() {
        assert_eq!(r(0, 0, 10, 20).area(), 200);
    }

    #[test]
    fn rect_intersection_overlap() {
        let a = r(0, 0, 10, 10);
        let b = r(5, 5, 10, 10);
        assert_eq!(a.intersection(b), Some(r(5, 5, 5, 5)));
    }

    #[test]
    fn rect_intersection_no_overlap() {
        assert!(r(0, 0, 5, 5).intersection(r(10, 10, 5, 5)).is_none());
    }

    #[test]
    fn rect_union() {
        let u = r(0, 0, 5, 5).union(r(10, 10, 5, 5));
        assert_eq!(u, r(0, 0, 15, 15));
    }

    #[test]
    fn rect_expand() {
        assert_eq!(r(10, 10, 20, 20).expand(5), r(5, 5, 30, 30));
        assert_eq!(r(10, 10, 6, 6).expand(-3), r(13, 13, 0, 0));
    }

    #[test]
    fn rect_scale() {
        let scaled = r(0, 0, 100, 100).scale(2.0);
        // center stays at (50,50), size doubles to 200x200 → x=-50,y=-50
        assert_eq!(scaled.width, 200);
        assert_eq!(scaled.height, 200);
    }

    #[test]
    fn screen_point_add() {
        let a = ScreenPoint { x: 10, y: 20 };
        let b = ScreenPoint { x: 5, y: -3 };
        assert_eq!(a + b, ScreenPoint { x: 15, y: 17 });
    }

    #[test]
    fn screen_point_sub() {
        let a = ScreenPoint { x: 10, y: 20 };
        let b = ScreenPoint { x: 3, y: 8 };
        assert_eq!(a - b, ScreenPoint { x: 7, y: 12 });
    }

    #[test]
    fn screen_point_from_tuple() {
        let pt: ScreenPoint = (42, -7).into();
        assert_eq!(pt, ScreenPoint { x: 42, y: -7 });
    }

    #[test]
    fn from_corners_normalizes_order() {
        let a = ScreenPoint { x: 10, y: 20 };
        let b = ScreenPoint { x: 5, y: 8 };
        let rect = Rect::from_corners(a, b);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 8);
        assert_eq!(rect.width, 5);
        assert_eq!(rect.height, 12);
        // Same result when corners are swapped.
        assert_eq!(Rect::from_corners(b, a), rect);
    }

    #[test]
    fn from_center_size_is_centered() {
        let center = ScreenPoint { x: 50, y: 40 };
        let rect = Rect::from_center_size(center, 20, 10);
        assert_eq!(rect.x, 40);
        assert_eq!(rect.y, 35);
        assert_eq!(rect.width, 20);
        assert_eq!(rect.height, 10);
        assert_eq!(rect.center(), center);
    }

    #[test]
    fn mask_region_new_and_label() {
        let rect = r(0, 0, 10, 10);
        let m = MaskRegion::new(rect);
        assert!(m.label.is_none());
        let m = m.with_label("timestamp");
        assert_eq!(m.label.as_deref(), Some("timestamp"));
    }
}
