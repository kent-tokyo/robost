pub mod local;

pub use local::LocalBackend;

use image::RgbaImage;
use rpa_template::{ScreenPoint, Target};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("capture error: {0}")]
    Capture(#[from] rpa_capture::CaptureError),
    #[error("input error: {0}")]
    Input(#[from] rpa_input::InputError),
    #[error("not supported on this backend: {0}")]
    NotSupported(String),
}

impl BackendError {
    /// Returns true when the error is a "window not found" capture failure.
    /// Used by the engine to detect RDP reconnect scenarios.
    pub fn is_window_not_found(&self) -> bool {
        matches!(
            self,
            BackendError::Capture(rpa_capture::CaptureError::WindowNotFound(_))
        )
    }
}

pub type Result<T> = std::result::Result<T, BackendError>;

/// Abstracts local and remote desktop backends.
/// All coordinates at the Backend boundary are screen-global.
pub trait Backend: Send + Sync {
    /// Capture a screenshot of the given target.
    fn capture(&self, target: &Target) -> Result<RgbaImage>;

    /// Capture the given target and also return its screen-global top-left origin.
    /// For `Target::Screen` the origin is (0, 0).
    fn capture_with_origin(&self, target: &Target) -> Result<(RgbaImage, ScreenPoint)> {
        let origin = match target {
            Target::Window { title_contains } => rpa_capture::window_origin(title_contains)?,
            Target::Process { name } => rpa_capture::window_origin(name)?,
            Target::Region(rect) => ScreenPoint { x: rect.x, y: rect.y },
            _ => ScreenPoint { x: 0, y: 0 },
        };
        let img = self.capture(target)?;
        Ok((img, origin))
    }

    /// Left-click at the given screen-global point.
    fn click(&self, point: ScreenPoint) -> Result<()>;

    /// Right-click at the given screen-global point.
    fn right_click(&self, point: ScreenPoint) -> Result<()>;

    /// Double-click at the given screen-global point.
    fn double_click(&self, point: ScreenPoint) -> Result<()>;

    /// Type plain text.
    fn type_text(&self, text: &str) -> Result<()>;

    /// Press a key by name (e.g. "Tab", "Enter", "F5").
    fn press_key(&self, key: &str) -> Result<()>;

    /// Perform a window management action.
    ///
    /// `action` is one of `"focus"`, `"maximize"`, `"minimize"`, `"close"`.
    /// The default impl returns `NotSupported`; override in concrete backends.
    fn control_window(&self, title_contains: &str, action: &str) -> Result<()> {
        Err(BackendError::NotSupported(format!(
            "control_window({action})"
        )))
    }

    /// Move the mouse cursor to the given screen-global point without clicking.
    fn move_mouse(&self, point: ScreenPoint) -> Result<()> {
        Err(BackendError::NotSupported("move_mouse".into()))
    }

    /// Click and drag from `from` to `to`, holding the button for `hold_ms` milliseconds.
    fn drag(&self, from: ScreenPoint, to: ScreenPoint, hold_ms: u64) -> Result<()> {
        Err(BackendError::NotSupported("drag".into()))
    }

    /// Scroll in the given direction by `amount` units.
    /// `direction` is one of `"up"`, `"down"`, `"left"`, `"right"`.
    fn scroll(&self, direction: &str, amount: i32) -> Result<()> {
        Err(BackendError::NotSupported("scroll".into()))
    }

    /// Press a key combination. All keys except the last are held as modifiers.
    /// Key names are case-insensitive: `"ctrl"`, `"alt"`, `"shift"`, `"meta"`,
    /// `"tab"`, `"enter"`, `"f1"`–`"f12"`, arrow keys (`"up"`, `"down"`, etc.),
    /// `"home"`, `"end"`, `"pageup"`, `"pagedown"`, or a single character.
    fn key_combo(&self, keys: &[&str]) -> Result<()> {
        Err(BackendError::NotSupported("key_combo".into()))
    }
}
