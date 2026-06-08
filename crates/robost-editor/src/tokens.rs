// Design tokens — single source of truth for colors, spacing, and geometry.
// Palette derived from VS Code Dark+ (colorRegistry.ts, workbench/common/theme.ts).
// Rule: hover/selection = background fill change only. Never add a border on hover.
use egui::{Color32, CornerRadius};

// ── Layout regions — Dark mode ────────────────────────────────────────────────

/// Leftmost 48 px activity bar. Stays dark in both themes for contrast.
pub(crate) const ACTIVITY_BAR_BG: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
/// Sidebar panel background — dark mode.
pub(crate) const SIDEBAR_BG: Color32 = Color32::from_rgb(0x25, 0x25, 0x26);
/// Canvas / List / Flow editor area — dark mode.
pub(crate) const EDITOR_BG: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
/// Status bar — accent blue (same in both themes).
pub(crate) const STATUSBAR_BG: Color32 = Color32::from_rgb(0x00, 0x7A, 0xCC);

// ── Layout regions — Light mode ───────────────────────────────────────────────

pub(crate) const SIDEBAR_BG_LIGHT: Color32 = Color32::from_rgb(0xF3, 0xF3, 0xF3);
pub(crate) const EDITOR_BG_LIGHT: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const PANEL_BG: Color32 = EDITOR_BG;
pub(crate) const PANEL_BG_LIGHT: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xFA);
pub(crate) const TOOLBAR_BG: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x2B);
pub(crate) const TOOLBAR_BG_LIGHT: Color32 = Color32::from_rgb(0xF7, 0xF7, 0xF7);
pub(crate) const BORDER: Color32 = Color32::from_rgb(0x3C, 0x3C, 0x3C);
pub(crate) const BORDER_LIGHT: Color32 = Color32::from_rgb(0xD6, 0xD6, 0xD6);

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return the sidebar background for the current theme.
#[inline]
pub(crate) fn sidebar_bg(dark: bool) -> Color32 {
    if dark {
        SIDEBAR_BG
    } else {
        SIDEBAR_BG_LIGHT
    }
}
/// Return the editor background for the current theme.
#[inline]
pub(crate) fn editor_bg(dark: bool) -> Color32 {
    if dark {
        EDITOR_BG
    } else {
        EDITOR_BG_LIGHT
    }
}
#[inline]
pub(crate) fn panel_bg(dark: bool) -> Color32 {
    if dark {
        PANEL_BG
    } else {
        PANEL_BG_LIGHT
    }
}
#[inline]
pub(crate) fn toolbar_bg(dark: bool) -> Color32 {
    if dark {
        TOOLBAR_BG
    } else {
        TOOLBAR_BG_LIGHT
    }
}
#[inline]
pub(crate) fn border(dark: bool) -> Color32 {
    if dark {
        BORDER
    } else {
        BORDER_LIGHT
    }
}

// ── Text ──────────────────────────────────────────────────────────────────────

pub(crate) const FG_DEFAULT: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const FG_DIM: Color32 = Color32::from_rgb(0x85, 0x85, 0x85);

// ── Interaction states (background fills — no borders) ────────────────────────

/// Row hover background (VS Code list.hoverBackground).
pub(crate) const LIST_HOVER: Color32 = Color32::from_rgb(0x2A, 0x2D, 0x2E);
/// Active selection background (VS Code list.activeSelectionBackground).
pub(crate) const LIST_SELECTION: Color32 = Color32::from_rgb(0x04, 0x39, 0x5E);
/// Unfocused / inactive selection (VS Code list.inactiveSelectionBackground).
#[allow(dead_code)]
pub(crate) const LIST_INACTIVE: Color32 = Color32::from_rgb(0x37, 0x37, 0x3D);
/// Keyboard focus outline — used as outline only, not fill.
#[allow(dead_code)]
pub(crate) const FOCUS_BORDER: Color32 = Color32::from_rgb(0x00, 0x7F, 0xD4);

// ── Semantic ──────────────────────────────────────────────────────────────────

pub(crate) const ACCENT: Color32 = Color32::from_rgb(0x00, 0x78, 0xD4);
pub(crate) const SUCCESS: Color32 = Color32::from_rgb(0x6C, 0xCB, 0x5F);
pub(crate) const WARNING: Color32 = Color32::from_rgb(0xFC, 0xE1, 0x00);
pub(crate) const ERROR: Color32 = Color32::from_rgb(0xFF, 0x99, 0xA4);
/// Teal border flash shown for 300 ms when a node snaps to the grid.
pub(crate) const SNAP_FLASH: Color32 = Color32::from_rgb(0x20, 0xC0, 0xA0);
#[allow(dead_code)]
pub(crate) const BADGE_BG: Color32 = Color32::from_rgb(0x4D, 0x4D, 0x4D);

// ── Canvas structural colors ──────────────────────────────────────────────────

/// Canvas background (== EDITOR_BG; alias for clarity in canvas.rs).
pub(crate) const CANVAS_BG: Color32 = EDITOR_BG;
/// Step node base background.
pub(crate) const NODE_BG: Color32 = Color32::from_rgb(0x2D, 0x2D, 0x2D);
/// Disabled node background (dark mode).
pub(crate) const NODE_BG_DISABLED: Color32 = Color32::from_gray(30);
/// Disabled node background (light mode).
pub(crate) const NODE_BG_DISABLED_LIGHT: Color32 = Color32::from_gray(240);
/// Selected node background.
pub(crate) const NODE_BG_SELECTED: Color32 = LIST_SELECTION;
/// Selected node background (light mode).
pub(crate) const NODE_BG_SELECTED_LIGHT: Color32 = Color32::from_rgb(0xCC, 0xE0, 0xFF);
/// Running node background.
pub(crate) const NODE_BG_RUNNING: Color32 = Color32::from_rgb(0x30, 0x28, 0x00);
/// Running node background (light mode).
pub(crate) const NODE_BG_RUNNING_LIGHT: Color32 = Color32::from_rgb(0xFF, 0xF5, 0xCC);
/// Flow edge / connection line color.
pub(crate) const EDGE_COLOR: Color32 = Color32::from_rgb(0x5F, 0x5F, 0x5F);

// ── Comment / annotation colors ───────────────────────────────────────────────

/// Comment sticky note background (yellow with alpha).
#[allow(dead_code)]
pub(crate) const COMMENT_FILL: Color32 = Color32::from_rgba_premultiplied(255, 220, 80, 35);
/// Comment border (darker yellow).
pub(crate) const COMMENT_BORDER: Color32 = Color32::from_rgba_premultiplied(180, 160, 50, 200);
/// Comment text color (dark mode).
pub(crate) const COMMENT_TEXT: Color32 = Color32::from_gray(30);
/// Comment icon (📝 indicator in corner).
pub(crate) const COMMENT_ICON_COLOR: Color32 = Color32::from_rgba_premultiplied(100, 80, 0, 180);

// ── Empty state / muted text ──────────────────────────────────────────────────

/// Muted text (gray, used for empty state descriptions).
pub(crate) const TEXT_MUTED: Color32 = Color32::from_gray(100);
/// Secondary muted text (lighter, for additional context).
pub(crate) const TEXT_MUTED_SECONDARY: Color32 = Color32::from_gray(65);

// ── Spacing (4 px grid) ───────────────────────────────────────────────────────

pub(crate) const SPACING_XS: f32 = 4.0;
pub(crate) const SPACING_SM: f32 = 8.0;
#[allow(dead_code)]
pub(crate) const SPACING_MD: f32 = 16.0;
#[allow(dead_code)]
pub(crate) const SPACING_LG: f32 = 24.0;
pub(crate) const ITEM_SPACING_X: f32 = 10.0;
pub(crate) const ITEM_SPACING_Y: f32 = 6.0;
pub(crate) const BUTTON_PADDING_X: f32 = 10.0;
pub(crate) const BUTTON_PADDING_Y: f32 = 5.0;
pub(crate) const INTERACT_HEIGHT_MIN: f32 = 26.0;
pub(crate) const WINDOW_MARGIN: i8 = 12;
pub(crate) const MENU_MARGIN: i8 = 8;

// ── Corner radii (VS Code: 0-2 px UI, 4 px cards) ────────────────────────────

/// Small UI elements: buttons, list items, widgets.
pub(crate) const ROUNDING_UI: CornerRadius = CornerRadius::same(2);
/// Cards and panels: canvas nodes, settings panels.
pub(crate) const ROUNDING_CARD: CornerRadius = CornerRadius::same(4);
/// Kept as aliases so remaining code compiles without mass-rename.
#[allow(dead_code)]
pub(crate) const ROUNDING_SM: CornerRadius = ROUNDING_UI;
#[allow(dead_code)]
pub(crate) const ROUNDING_MD: CornerRadius = ROUNDING_CARD;

// ── Layout ────────────────────────────────────────────────────────────────────

pub(crate) const ACTIVITY_BAR_WIDTH: f32 = 48.0;
pub(crate) const STEP_ROW_HEIGHT: f32 = 28.0;
#[allow(dead_code)]
pub(crate) const TOOLBAR_HEIGHT: f32 = 34.0;
pub(crate) const STATUS_BAR_HEIGHT: f32 = 22.0;
#[allow(dead_code)]
pub(crate) const SIDEBAR_WIDTH_DEFAULT: f32 = 240.0;
#[allow(dead_code)]
pub(crate) const SIDEBAR_WIDTH_MIN: f32 = 180.0;
pub(crate) const INSPECTOR_WIDTH_DEFAULT: f32 = 340.0;
pub(crate) const INSPECTOR_WIDTH_MIN: f32 = 280.0;
pub(crate) const INSPECTOR_WIDTH_COLLAPSED: f32 = 48.0;
pub(crate) const INSPECTOR_WIDTH_MAX: f32 = 560.0;
pub(crate) const AI_GENERATED_NODE_GAP_Y: f32 = 56.0;

// ── Interaction ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub(crate) const DRAG_THRESHOLD: f32 = 4.0;
pub(crate) const SNAP_FLASH_MS: u64 = 300;
