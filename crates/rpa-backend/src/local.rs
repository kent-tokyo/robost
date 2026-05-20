use std::sync::Mutex;

use image::RgbaImage;
use rpa_input::InputController;
use rpa_template::{ScreenPoint, Target};
use tracing::{instrument, warn};

use crate::{Backend, BackendError, Result};

pub struct LocalBackend {
    /// None if input controller failed to initialize (e.g. no Accessibility permission on macOS).
    input: Option<Mutex<InputController>>,
}

impl LocalBackend {
    pub fn new() -> Result<Self> {
        let input = match InputController::new() {
            Ok(c) => Some(Mutex::new(c)),
            Err(e) => {
                warn!("input controller unavailable (click/type/press will fail): {e}");
                None
            }
        };
        Ok(Self { input })
    }

    fn with_input<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut InputController) -> rpa_input::Result<T>,
    {
        let mutex = self.input.as_ref().ok_or_else(|| {
            BackendError::NotSupported(
                "input not available (Accessibility permission required on macOS)".to_owned(),
            )
        })?;
        let mut guard = mutex.lock().unwrap();
        f(&mut guard).map_err(BackendError::Input)
    }
}

impl Backend for LocalBackend {
    #[instrument(name = "local_capture", skip(self))]
    fn capture(&self, target: &Target) -> Result<RgbaImage> {
        Ok(rpa_capture::capture(target)?)
    }

    #[instrument(name = "local_click", fields(x = point.x, y = point.y), skip(self))]
    fn click(&self, point: ScreenPoint) -> Result<()> {
        self.with_input(|c| c.click(point))
    }

    #[instrument(name = "local_right_click", fields(x = point.x, y = point.y), skip(self))]
    fn right_click(&self, point: ScreenPoint) -> Result<()> {
        self.with_input(|c| c.right_click(point))
    }

    #[instrument(name = "local_double_click", fields(x = point.x, y = point.y), skip(self))]
    fn double_click(&self, point: ScreenPoint) -> Result<()> {
        self.with_input(|c| c.double_click(point))
    }

    #[instrument(name = "local_type_text", skip(self, text))]
    fn type_text(&self, text: &str) -> Result<()> {
        self.with_input(|c| c.type_text(text))
    }

    #[instrument(name = "local_press_key", fields(key), skip(self))]
    fn press_key(&self, key: &str) -> Result<()> {
        let k = parse_key(key)
            .ok_or_else(|| BackendError::NotSupported(format!("unknown key: {key}")))?;
        self.with_input(|c| c.press_key(k))
    }

    #[instrument(
        name = "local_control_window",
        fields(title_contains, action),
        skip(self)
    )]
    fn control_window(&self, title_contains: &str, action: &str) -> Result<()> {
        rpa_input::control_window(title_contains, action).map_err(BackendError::Input)
    }

    #[instrument(name = "local_move_mouse", fields(x = point.x, y = point.y), skip(self))]
    fn move_mouse(&self, point: ScreenPoint) -> Result<()> {
        self.with_input(|c| c.move_mouse(point))
    }

    #[instrument(name = "local_drag", fields(fx = from.x, fy = from.y, tx = to.x, ty = to.y, hold_ms), skip(self))]
    fn drag(&self, from: ScreenPoint, to: ScreenPoint, hold_ms: u64) -> Result<()> {
        self.with_input(|c| c.drag(from, to, hold_ms))
    }

    #[instrument(name = "local_scroll", fields(direction, amount), skip(self))]
    fn scroll(&self, direction: &str, amount: i32) -> Result<()> {
        use rpa_input::ScrollDir;
        let dir = match direction {
            "up" => ScrollDir::Up,
            "down" => ScrollDir::Down,
            "left" => ScrollDir::Left,
            "right" => ScrollDir::Right,
            other => {
                return Err(crate::BackendError::NotSupported(format!(
                    "unknown scroll direction: {other}"
                )))
            }
        };
        self.with_input(|c| c.scroll(dir, amount))
    }

    #[instrument(name = "local_key_combo", fields(?keys), skip(self))]
    fn key_combo(&self, keys: &[&str]) -> Result<()> {
        let parsed: Vec<enigo::Key> = keys
            .iter()
            .map(|k| {
                parse_key(k).ok_or_else(|| BackendError::NotSupported(format!("unknown key: {k}")))
            })
            .collect::<Result<Vec<_>>>()?;
        self.with_input(|c| c.key_combo(&parsed))
    }
}

/// Map a key name string (case-insensitive) to an `enigo::Key`.
/// Accepts modifier names, named keys, function keys, and single characters.
fn parse_key(name: &str) -> Option<enigo::Key> {
    use enigo::Key::*;
    match name.to_ascii_lowercase().as_str() {
        // modifiers
        "ctrl" | "control" => Some(Control),
        "alt" => Some(Alt),
        "shift" => Some(Shift),
        "meta" | "win" | "cmd" | "command" | "super" => Some(Meta),
        // basic navigation / editing
        "tab" => Some(Tab),
        "enter" | "return" => Some(Return),
        "escape" | "esc" => Some(Escape),
        "space" => Some(Space),
        "backspace" => Some(Backspace),
        "delete" | "del" => Some(Delete),
        // extended navigation
        "home" => Some(Home),
        "end" => Some(End),
        "pageup" | "page_up" => Some(PageUp),
        "pagedown" | "page_down" => Some(PageDown),
        "up" => Some(UpArrow),
        "down" => Some(DownArrow),
        "left" => Some(LeftArrow),
        "right" => Some(RightArrow),
        // function keys
        "f1" => Some(F1),
        "f2" => Some(F2),
        "f3" => Some(F3),
        "f4" => Some(F4),
        "f5" => Some(F5),
        "f6" => Some(F6),
        "f7" => Some(F7),
        "f8" => Some(F8),
        "f9" => Some(F9),
        "f10" => Some(F10),
        "f11" => Some(F11),
        "f12" => Some(F12),
        // single unicode character
        s if s.chars().count() == 1 => Some(Unicode(s.chars().next().unwrap())),
        _ => std::option::Option::None,
    }
}
