pub mod types;
pub use types::{MaskRegion, Rect, ScreenPoint, WindowPoint};

pub mod template_match;
pub use template_match::{MatchError, MatchResult, TemplateMatcher};

#[cfg(feature = "ocr")]
pub mod ocr;

#[cfg(feature = "ml")]
pub mod ml;
