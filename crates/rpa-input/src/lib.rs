use enigo::{Axis, Button, Enigo, Key, Keyboard, Mouse, Settings};
use rpa_template::ScreenPoint;
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("enigo error: {0}")]
    Enigo(#[from] enigo::NewConError),
    #[error("input send error: {0}")]
    Send(String),
    #[error("window focus error: {0}")]
    Focus(String),
}

pub type Result<T> = std::result::Result<T, InputError>;

/// Scroll direction for `InputController::scroll`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDir {
    Up,
    Down,
    Left,
    Right,
}

pub struct InputController {
    enigo: Enigo,
}

impl InputController {
    pub fn new() -> Result<Self> {
        Ok(Self {
            enigo: Enigo::new(&Settings::default())?,
        })
    }

    /// Move mouse and left-click at `point` (screen-global coords).
    #[instrument(name = "click", fields(x = point.x, y = point.y), skip(self))]
    pub fn click(&mut self, point: ScreenPoint) -> Result<()> {
        self.move_to(point)?;
        self.enigo
            .button(Button::Left, enigo::Direction::Click)
            .map_err(|e| InputError::Send(e.to_string()))?;
        Ok(())
    }

    /// Move mouse and right-click at `point`.
    #[instrument(name = "right_click", fields(x = point.x, y = point.y), skip(self))]
    pub fn right_click(&mut self, point: ScreenPoint) -> Result<()> {
        self.move_to(point)?;
        self.enigo
            .button(Button::Right, enigo::Direction::Click)
            .map_err(|e| InputError::Send(e.to_string()))?;
        Ok(())
    }

    /// Move mouse and double-click at `point`.
    #[instrument(name = "double_click", fields(x = point.x, y = point.y), skip(self))]
    pub fn double_click(&mut self, point: ScreenPoint) -> Result<()> {
        self.move_to(point)?;
        self.enigo
            .button(Button::Left, enigo::Direction::Click)
            .map_err(|e| InputError::Send(e.to_string()))?;
        self.enigo
            .button(Button::Left, enigo::Direction::Click)
            .map_err(|e| InputError::Send(e.to_string()))?;
        Ok(())
    }

    /// Move the mouse cursor to `point` (screen-global coords).
    #[instrument(name = "move_mouse", fields(x = point.x, y = point.y), skip(self))]
    pub fn move_mouse(&mut self, point: ScreenPoint) -> Result<()> {
        self.move_to(point)
    }

    /// Click and drag from `from` to `to`, holding the button for `hold_ms`.
    #[instrument(name = "drag", fields(fx = from.x, fy = from.y, tx = to.x, ty = to.y), skip(self))]
    pub fn drag(&mut self, from: ScreenPoint, to: ScreenPoint, hold_ms: u64) -> Result<()> {
        self.move_to(from)?;
        self.enigo
            .button(Button::Left, enigo::Direction::Press)
            .map_err(|e| InputError::Send(e.to_string()))?;
        if hold_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(hold_ms));
        }
        self.move_to(to)?;
        self.enigo
            .button(Button::Left, enigo::Direction::Release)
            .map_err(|e| InputError::Send(e.to_string()))?;
        Ok(())
    }

    /// Scroll in the given direction by `amount` units.
    #[instrument(name = "scroll", fields(?direction, amount), skip(self))]
    pub fn scroll(&mut self, direction: ScrollDir, amount: i32) -> Result<()> {
        let (axis, length) = match direction {
            ScrollDir::Up => (Axis::Vertical, -amount),
            ScrollDir::Down => (Axis::Vertical, amount),
            ScrollDir::Left => (Axis::Horizontal, -amount),
            ScrollDir::Right => (Axis::Horizontal, amount),
        };
        self.enigo
            .scroll(length, axis)
            .map_err(|e| InputError::Send(e.to_string()))
    }

    fn move_to(&mut self, point: ScreenPoint) -> Result<()> {
        self.enigo
            .move_mouse(point.x, point.y, enigo::Coordinate::Abs)
            .map_err(|e| InputError::Send(e.to_string()))
    }

    /// Type a plain-text string.
    #[instrument(name = "type_text", skip(self, text))]
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo
            .text(text)
            .map_err(|e| InputError::Send(e.to_string()))?;
        Ok(())
    }

    /// Press a single key (tap: down + up).
    #[instrument(name = "press_key", fields(?key), skip(self))]
    pub fn press_key(&mut self, key: Key) -> Result<()> {
        self.enigo
            .key(key, enigo::Direction::Click)
            .map_err(|e| InputError::Send(e.to_string()))?;
        Ok(())
    }

    /// Press a key combination.
    /// All keys except the last are held as modifiers (Press); the last is clicked
    /// (Click = down + up); then modifiers are released in reverse order.
    #[instrument(name = "key_combo", fields(?keys), skip(self))]
    pub fn key_combo(&mut self, keys: &[Key]) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        let (modifiers, tail) = keys.split_at(keys.len() - 1);
        let main = tail[0];

        for &m in modifiers {
            self.enigo
                .key(m, enigo::Direction::Press)
                .map_err(|e| InputError::Send(e.to_string()))?;
        }
        self.enigo
            .key(main, enigo::Direction::Click)
            .map_err(|e| InputError::Send(e.to_string()))?;
        for &m in modifiers.iter().rev() {
            self.enigo
                .key(m, enigo::Direction::Release)
                .map_err(|e| InputError::Send(e.to_string()))?;
        }
        Ok(())
    }

    /// Bring the window whose title contains `title` to the foreground,
    /// then perform `action`. Ensures input reaches the right window.
    pub fn with_focus<F, T>(&mut self, title: &str, action: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        focus_window(title)?;
        action(self)
    }
}

/// Perform a window management action (focus, maximize, minimize, close).
///
/// `action` must be one of `"focus"`, `"maximize"`, `"minimize"`, `"close"`.
/// Uses platform-specific APIs: Win32 on Windows, AppleScript on macOS, wmctrl on Linux.
pub fn control_window(title: &str, action: &str) -> Result<()> {
    #[cfg(windows)]
    {
        windows_control(title, action)
    }
    #[cfg(target_os = "macos")]
    {
        macos_control(title, action)
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        linux_control(title, action)
    }
}

/// Platform-specific window focus implementation.
fn focus_window(title: &str) -> Result<()> {
    #[cfg(windows)]
    {
        windows_focus(title)
    }
    #[cfg(target_os = "macos")]
    {
        macos_focus(title)
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        linux_focus(title)
    }
}

#[cfg(windows)]
fn windows_focus(title: &str) -> Result<()> {
    windows_control(title, "focus")
}

#[cfg(windows)]
fn windows_control(title: &str, action: &str) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, PostMessageW, SetForegroundWindow, ShowWindow,
        LPARAM, SW_MAXIMIZE, SW_MINIMIZE, WPARAM, WM_CLOSE,
    };
    use windows::core::PCWSTR;

    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let hwnd = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr())) };
    if hwnd.0 == 0 {
        return Err(InputError::Focus(format!("window not found: {title}")));
    }
    match action {
        "focus" => { unsafe { SetForegroundWindow(hwnd) }; }
        "maximize" => { unsafe { ShowWindow(hwnd, SW_MAXIMIZE) }; }
        "minimize" => { unsafe { ShowWindow(hwnd, SW_MINIMIZE) }; }
        "close" => {
            unsafe { PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) }
                .map_err(|e| InputError::Focus(e.to_string()))?;
        }
        other => return Err(InputError::Focus(format!("unknown window action: {other}"))),
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_focus(title: &str) -> Result<()> {
    macos_control(title, "focus")
}

#[cfg(target_os = "macos")]
fn macos_control(title: &str, action: &str) -> Result<()> {
    let script = match action {
        "focus" => format!(
            r#"tell application "System Events"
                set frontApp to first application process whose (name of windows) contains "{title}"
                set frontmost of frontApp to true
            end tell"#
        ),
        "maximize" => format!(
            r#"tell application "System Events"
                set frontApp to first application process whose (name of windows) contains "{title}"
                set frontmost of frontApp to true
                tell window 1 of frontApp to set zoomed to true
            end tell"#
        ),
        "minimize" => format!(
            r#"tell application "System Events"
                set frontApp to first application process whose (name of windows) contains "{title}"
                tell window 1 of frontApp to set miniaturized to true
            end tell"#
        ),
        "close" => format!(
            r#"tell application "System Events"
                set frontApp to first application process whose (name of windows) contains "{title}"
                tell window 1 of frontApp to close
            end tell"#
        ),
        other => return Err(InputError::Focus(format!("unknown window action: {other}"))),
    };
    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| InputError::Focus(e.to_string()))?;
    if !status.success() {
        return Err(InputError::Focus(format!(
            "osascript failed: {action} on '{title}'"
        )));
    }
    Ok(())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn linux_focus(title: &str) -> Result<()> {
    linux_control(title, "focus")
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn linux_control(title: &str, action: &str) -> Result<()> {
    // Requires wmctrl to be installed (apt install wmctrl).
    let args: &[&str] = match action {
        "focus" => &["-a", title],
        "maximize" => &["-r", title, "-b", "add,maximized_vert,maximized_horz"],
        "minimize" => &["-r", title, "-b", "add,hidden"],
        "close" => &["-c", title],
        other => return Err(InputError::Focus(format!("unknown window action: {other}"))),
    };
    let status = std::process::Command::new("wmctrl")
        .args(args)
        .status()
        .map_err(|e| InputError::Focus(e.to_string()))?;
    if !status.success() {
        return Err(InputError::Focus(format!(
            "wmctrl failed: {action} on '{title}'"
        )));
    }
    Ok(())
}
