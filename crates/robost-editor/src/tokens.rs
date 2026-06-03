// Design tokens — single source of truth for colors, spacing, and geometry.
// See DESIGN.md Appendix B.
// Note: egui 0.33 uses CornerRadius (not Rounding); same() takes u8, not f32.
use egui::{Color32, CornerRadius};

// ── Semantic colors ───────────────────────────────────────────────────────────

pub(crate) const ACCENT: Color32 = Color32::from_rgb(0x00, 0x78, 0xD4);
pub(crate) const SUCCESS: Color32 = Color32::from_rgb(0x10, 0x7C, 0x10);
pub(crate) const WARNING: Color32 = Color32::from_rgb(0xC1, 0x9C, 0x00);
pub(crate) const ERROR: Color32 = Color32::from_rgb(0xC4, 0x2B, 0x1C);
#[allow(dead_code)]
pub(crate) const PLUGIN_PURPLE: Color32 = Color32::from_rgb(0x7B, 0x68, 0xEE);

// ── Spacing ───────────────────────────────────────────────────────────────────

pub(crate) const SPACING_XS: f32 = 4.0;
pub(crate) const SPACING_SM: f32 = 8.0;
#[allow(dead_code)]
pub(crate) const SPACING_MD: f32 = 16.0;
#[allow(dead_code)]
pub(crate) const SPACING_LG: f32 = 24.0;

// ── Corner radii ──────────────────────────────────────────────────────────────

pub(crate) const ROUNDING_SM: CornerRadius = CornerRadius::same(4);
pub(crate) const ROUNDING_MD: CornerRadius = CornerRadius::same(8);

// ── Canvas structural colors (DESIGN.md §4 elevation layers) ─────────────────

/// L0 — キャンバス背景
pub(crate) const CANVAS_BG: Color32 = Color32::from_rgb(26, 27, 30);
/// L2 — ノードベース背景（bg_color + 20）
pub(crate) const NODE_BG: Color32 = Color32::from_gray(40);
/// L2 — 選択ノード背景（ACCENT 由来の暗い tint）
pub(crate) const NODE_BG_SELECTED: Color32 = Color32::from_rgb(28, 52, 88);
/// L2 — 実行中ノード背景（WARNING 由来の暗い tint）
pub(crate) const NODE_BG_RUNNING: Color32 = Color32::from_rgb(80, 60, 10);
/// エッジの線色（フロー順序を示すニュートラルな線）
pub(crate) const EDGE_COLOR: Color32 = Color32::from_gray(95);

// ── Geometry ──────────────────────────────────────────────────────────────────

/// Target height for step list rows.
pub(crate) const STEP_ROW_HEIGHT: f32 = 32.0;
#[allow(dead_code)]
pub(crate) const TOOLBAR_HEIGHT: f32 = 40.0;
