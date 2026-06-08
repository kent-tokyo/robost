// ---- canvas view ------------------------------------------------------------

use eframe::egui;
use std::collections::HashMap;

use crate::flow_helpers::{
    default_canvas_cols, default_canvas_pos, get_inner_steps, get_step_key, step_matches,
    step_summary, NODE_H, NODE_W,
};
use crate::state::EditorApp;
use crate::tokens;
use crate::types::{CanvasComment, CanvasContextAction, ConfirmAction, ViewMode};

pub(crate) fn layout_path(scenario_path: &std::path::Path) -> std::path::PathBuf {
    let mut p = scenario_path.to_path_buf();
    let fname = p
        .file_name()
        .map(|n| format!("{}.layout.json", n.to_string_lossy()))
        .unwrap_or_else(|| "layout.json".into());
    p.set_file_name(fname);
    p
}

impl EditorApp {
    pub(crate) fn ensure_canvas_layout(&mut self) {
        const GAP_X: f32 = 80.0;
        const GAP_Y: f32 = 60.0;
        const COLS: usize = 5;
        for idx in 0..self.steps.len() {
            self.canvas_positions.entry(idx).or_insert_with(|| {
                let col = idx % COLS;
                let row = idx / COLS;
                egui::pos2(
                    col as f32 * (NODE_W + GAP_X) + 40.0,
                    row as f32 * (NODE_H + GAP_Y) + 40.0,
                )
            });
        }
    }

    pub(crate) fn canvas_shift_positions(
        positions: &mut HashMap<usize, egui::Pos2>,
        at: usize,
        delta: isize,
    ) {
        let mut keys: Vec<usize> = positions.keys().cloned().collect();
        if delta > 0 {
            // Sort descending so we don't clobber higher indices when shifting up
            keys.sort_unstable_by(|a, b| b.cmp(a));
            for k in keys {
                if k >= at {
                    let v = positions.remove(&k).unwrap();
                    positions.insert(k + 1, v);
                }
            }
        } else if delta < 0 {
            // Sort ascending so removal doesn't affect later keys
            keys.sort_unstable();
            for k in keys {
                if k == at {
                    positions.remove(&k);
                } else if k > at {
                    let v = positions.remove(&k).unwrap();
                    positions.insert(k - 1, v);
                }
            }
        }
    }

    /// Shift run-state indices in `canvas_error_steps`, `canvas_completed_steps`,
    /// and `expanded_steps` when a step at `at` is deleted (delta == -1 only).
    /// Call this once per deleted index, in high-to-low order (mirrors
    /// `canvas_shift_positions`).
    pub(crate) fn canvas_shift_run_state(
        error_steps: &mut std::collections::HashMap<usize, String>,
        completed_steps: &mut std::collections::HashSet<usize>,
        expanded: &mut std::collections::HashSet<usize>,
        at: usize,
    ) {
        // Remove the entry being deleted, then decrement all keys above it.
        error_steps.remove(&at);
        let shifted: std::collections::HashMap<usize, String> = error_steps
            .drain()
            .map(|(k, v)| (if k > at { k - 1 } else { k }, v))
            .collect();
        *error_steps = shifted;

        completed_steps.remove(&at);
        let shifted_c: std::collections::HashSet<usize> = completed_steps
            .drain()
            .map(|k| if k > at { k - 1 } else { k })
            .collect();
        *completed_steps = shifted_c;

        expanded.remove(&at);
        let shifted_e: std::collections::HashSet<usize> = expanded
            .drain()
            .map(|k| if k > at { k - 1 } else { k })
            .collect();
        *expanded = shifted_e;
    }

    pub(crate) fn save_canvas_layout(&self) {
        let Some(ref path) = self.path else { return };
        let lpath = layout_path(path);
        let mut positions = serde_json::Map::new();
        for (k, v) in &self.canvas_positions {
            positions.insert(k.to_string(), serde_json::json!([v.x, v.y]));
        }
        let comments_json: Vec<serde_json::Value> = self
            .canvas_comments
            .iter()
            .map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null))
            .collect();
        let layout = serde_json::json!({
            "positions": positions,
            "comments": comments_json,
        });
        if let Ok(text) = serde_json::to_string(&layout) {
            if let Err(e) = std::fs::write(&lpath, &text) {
                tracing::warn!("canvas layout save failed: {e} — {}", lpath.display());
            }
        }
    }

    pub(crate) fn load_canvas_layout(&mut self, scenario_path: &std::path::Path) {
        self.canvas_positions.clear();
        self.canvas_comments.clear();
        self.canvas_comment_next_id = 1;
        let lpath = layout_path(scenario_path);
        let Ok(text) = std::fs::read_to_string(&lpath) else {
            return;
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        if let Some(obj) = val["positions"].as_object() {
            for (k, v) in obj {
                let Ok(idx) = k.parse::<usize>() else {
                    continue;
                };
                let Some(arr) = v.as_array() else { continue };
                if arr.len() == 2 {
                    let x = arr[0].as_f64().unwrap_or(0.0) as f32;
                    let y = arr[1].as_f64().unwrap_or(0.0) as f32;
                    self.canvas_positions.insert(idx, egui::pos2(x, y));
                }
            }
        }
        if let Some(comments_arr) = val["comments"].as_array() {
            for cv in comments_arr {
                if let Ok(c) = serde_json::from_value::<CanvasComment>(cv.clone()) {
                    let id = c.id;
                    self.canvas_comments.push(c);
                    if id >= self.canvas_comment_next_id {
                        self.canvas_comment_next_id = id + 1;
                    }
                }
            }
        }
    }

    pub(crate) fn canvas_fit_view(&mut self, viewport_size: egui::Vec2) {
        let n = self.steps.len();
        if n == 0 {
            return;
        }
        let margin = 40.0_f32;

        // Use the same default-position formula as show_canvas so nodes without a stored
        // position (e.g. freshly loaded scenarios) are included in the bounding box.
        let cols = default_canvas_cols(n);
        let pos_of = |i: usize| {
            self.canvas_positions
                .get(&i)
                .copied()
                .unwrap_or_else(|| default_canvas_pos(i, cols))
        };
        let min_x = (0..n).map(|i| pos_of(i).x).fold(f32::INFINITY, f32::min);
        let min_y = (0..n).map(|i| pos_of(i).y).fold(f32::INFINITY, f32::min);
        let max_x = (0..n)
            .map(|i| pos_of(i).x)
            .fold(f32::NEG_INFINITY, f32::max)
            + NODE_W;
        let max_y = (0..n)
            .map(|i| pos_of(i).y)
            .fold(f32::NEG_INFINITY, f32::max)
            + NODE_H;

        let content_w = max_x - min_x + margin * 2.0;
        let content_h = max_y - min_y + margin * 2.0;

        // When the minimap is visible (n >= 15), shrink effective width so fit-view
        // does not place content behind the minimap overlay.
        const MM_W: f32 = 160.0;
        const MM_MARGIN: f32 = 8.0;
        let minimap_active = n > 0 && (n >= 15 || self.settings.minimap_show);
        let effective_w = if minimap_active {
            (viewport_size.x - MM_W - MM_MARGIN * 2.0).max(viewport_size.x * 0.5)
        } else {
            viewport_size.x
        };
        let zoom_x = effective_w / content_w;
        let zoom_y = viewport_size.y / content_h;
        self.canvas_zoom = zoom_x.min(zoom_y).clamp(0.25, 2.0);

        // Center in the visible area (minimap occupies bottom-right when active)
        let vis_w = if minimap_active {
            (viewport_size.x - MM_W - MM_MARGIN * 2.0).max(viewport_size.x * 0.5)
        } else {
            viewport_size.x
        };
        let z = self.canvas_zoom;
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        self.canvas_pan = egui::vec2(
            vis_w / 2.0 / z - center_x,
            viewport_size.y / 2.0 / z - center_y,
        );
    }

    /// Shift `@N` numeric suffixes in form_edit_buffers when steps are inserted (delta>0)
    /// or removed (delta<0) at position `at`. Mirrors canvas_shift_positions behaviour.
    pub(crate) fn form_edit_buffers_shift(
        buffers: &mut HashMap<String, String>,
        at: usize,
        delta: isize,
    ) {
        // Keys have format "{field}@{step_idx}" or "{field}@{step_idx}@{suffix}".
        // We extract the step index as the segment between the first '@' and the
        // next '@' (or end of string), so compound keys like "keys@5@add" are handled.
        let keys: Vec<String> = buffers.keys().cloned().collect();
        let mut to_move: Vec<(String, usize)> = keys
            .iter()
            .filter_map(|k| {
                let first_at = k.find('@')?;
                let remainder = &k[first_at + 1..];
                let end = remainder.find('@').unwrap_or(remainder.len());
                let n: usize = remainder[..end].parse().ok()?;
                if n >= at {
                    Some((k.clone(), n))
                } else {
                    None
                }
            })
            .collect();
        if delta > 0 {
            // Sort descending to avoid key collisions when renaming in-place
            to_move.sort_by_key(|b| std::cmp::Reverse(b.1));
            for (key, n) in to_move {
                if let Some(val) = buffers.remove(&key) {
                    let first_at = key.find('@').unwrap();
                    let remainder = &key[first_at + 1..];
                    let end = remainder.find('@').unwrap_or(remainder.len());
                    let suffix = &remainder[end..]; // "" or "@add" etc.
                    let prefix = &key[..first_at];
                    buffers.insert(format!("{prefix}@{}{suffix}", n + 1), val);
                }
            }
        } else {
            to_move.sort_by_key(|a| a.1);
            for (key, n) in to_move {
                if n == at {
                    buffers.remove(&key);
                } else if let Some(val) = buffers.remove(&key) {
                    let first_at = key.find('@').unwrap();
                    let remainder = &key[first_at + 1..];
                    let end = remainder.find('@').unwrap_or(remainder.len());
                    let suffix = &remainder[end..];
                    let prefix = &key[..first_at];
                    buffers.insert(format!("{prefix}@{}{suffix}", n - 1), val);
                }
            }
        }
    }

    pub(crate) fn canvas_align_left(&mut self) {
        if self.multi_selected.len() < 2 {
            return;
        }
        let min_x = self
            .multi_selected
            .iter()
            .filter_map(|i| self.canvas_positions.get(i))
            .map(|p| p.x)
            .fold(f32::INFINITY, f32::min);
        let any_change = self.multi_selected.iter().any(|i| {
            self.canvas_positions
                .get(i)
                .map(|p| p.x != min_x)
                .unwrap_or(false)
        });
        if !any_change {
            return;
        }
        self.push_undo();
        for &i in &self.multi_selected {
            if let Some(p) = self.canvas_positions.get_mut(&i) {
                p.x = min_x;
            }
        }
        self.canvas_layout_dirty = true;
    }

    pub(crate) fn canvas_align_top(&mut self) {
        if self.multi_selected.len() < 2 {
            return;
        }
        let min_y = self
            .multi_selected
            .iter()
            .filter_map(|i| self.canvas_positions.get(i))
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min);
        let any_change = self.multi_selected.iter().any(|i| {
            self.canvas_positions
                .get(i)
                .map(|p| p.y != min_y)
                .unwrap_or(false)
        });
        if !any_change {
            return;
        }
        self.push_undo();
        for &i in &self.multi_selected {
            if let Some(p) = self.canvas_positions.get_mut(&i) {
                p.y = min_y;
            }
        }
        self.canvas_layout_dirty = true;
    }

    pub(crate) fn canvas_distribute_h(&mut self) {
        if self.multi_selected.len() < 3 {
            return;
        }
        self.push_undo();
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_by(|&a, &b| {
            let ax = self.canvas_positions.get(&a).map(|p| p.x).unwrap_or(0.0);
            let bx = self.canvas_positions.get(&b).map(|p| p.x).unwrap_or(0.0);
            ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
        });
        let left_x = self
            .canvas_positions
            .get(&indices[0])
            .map(|p| p.x)
            .unwrap_or(0.0);
        let right_x = self
            .canvas_positions
            .get(indices.last().unwrap())
            .map(|p| p.x)
            .unwrap_or(NODE_W);
        // Gap-based distribution: equal whitespace between nodes (accounts for NODE_W).
        // span = left-edge-of-first to right-edge-of-last
        let span = (right_x + NODE_W) - left_x;
        let total_node_w = indices.len() as f32 * NODE_W;
        let min_gap = 20.0_f32;
        if span > total_node_w + min_gap * (indices.len() - 1) as f32 {
            // Normal: fit within existing bounding box
            let gap = (span - total_node_w) / (indices.len() - 1) as f32;
            let step = NODE_W + gap;
            for (rank, &i) in indices.iter().enumerate() {
                if let Some(p) = self.canvas_positions.get_mut(&i) {
                    p.x = left_x + rank as f32 * step;
                }
            }
        } else {
            // Nodes are too close / overlapping: expand around the cluster centre
            let center_x = (left_x + right_x + NODE_W) / 2.0;
            let step = NODE_W + min_gap;
            let total_w = indices.len() as f32 * NODE_W + (indices.len() - 1) as f32 * min_gap;
            let new_left = center_x - total_w / 2.0;
            for (rank, &i) in indices.iter().enumerate() {
                if let Some(p) = self.canvas_positions.get_mut(&i) {
                    p.x = new_left + rank as f32 * step;
                }
            }
        }
        self.canvas_layout_dirty = true;
        self.save_canvas_layout();
    }

    pub(crate) fn canvas_distribute_v(&mut self) {
        if self.multi_selected.len() < 3 {
            return;
        }
        self.push_undo();
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_by(|&a, &b| {
            let ay = self.canvas_positions.get(&a).map(|p| p.y).unwrap_or(0.0);
            let by_ = self.canvas_positions.get(&b).map(|p| p.y).unwrap_or(0.0);
            ay.partial_cmp(&by_).unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_y = self
            .canvas_positions
            .get(&indices[0])
            .map(|p| p.y)
            .unwrap_or(0.0);
        let bottom_y = self
            .canvas_positions
            .get(indices.last().unwrap())
            .map(|p| p.y)
            .unwrap_or(NODE_H);
        let span = (bottom_y + NODE_H) - top_y;
        let total_node_h = indices.len() as f32 * NODE_H;
        let min_gap = 20.0_f32;
        if span > total_node_h + min_gap * (indices.len() - 1) as f32 {
            let gap = (span - total_node_h) / (indices.len() - 1) as f32;
            let step = NODE_H + gap;
            for (rank, &i) in indices.iter().enumerate() {
                if let Some(p) = self.canvas_positions.get_mut(&i) {
                    p.y = top_y + rank as f32 * step;
                }
            }
        } else {
            let center_y = (top_y + bottom_y + NODE_H) / 2.0;
            let step = NODE_H + min_gap;
            let total_h = indices.len() as f32 * NODE_H + (indices.len() - 1) as f32 * min_gap;
            let new_top = center_y - total_h / 2.0;
            for (rank, &i) in indices.iter().enumerate() {
                if let Some(p) = self.canvas_positions.get_mut(&i) {
                    p.y = new_top + rank as f32 * step;
                }
            }
        }
        self.canvas_layout_dirty = true;
    }

    pub(crate) fn show_canvas(&mut self, ui: &mut egui::Ui) {
        use crate::types::{category_color, step_key_category};
        use egui::{epaint::CubicBezierShape, Align2, Color32, FontId, Rect, Sense, Stroke, Vec2};

        // ── Theme-aware canvas colors (VS Code–derived) ──────────────────────
        let dark = ui.visuals().dark_mode;
        let cv_bg = if dark {
            tokens::CANVAS_BG
        } else {
            Color32::from_gray(248)
        };
        // Node backgrounds — flat, no blending with category color
        let node_bg = if dark {
            tokens::NODE_BG
        } else {
            Color32::from_gray(255)
        };
        let node_bg_disabled = if dark {
            tokens::NODE_BG_DISABLED
        } else {
            tokens::NODE_BG_DISABLED_LIGHT
        };
        let node_bg_selected = if dark {
            tokens::NODE_BG_SELECTED
        } else {
            tokens::NODE_BG_SELECTED_LIGHT
        };
        let node_bg_running = if dark {
            tokens::NODE_BG_RUNNING
        } else {
            tokens::NODE_BG_RUNNING_LIGHT
        };
        // Node border — near-invisible (1 step lighter/darker than node bg)
        let node_border = if dark {
            Color32::from_gray(50)
        } else {
            Color32::from_gray(210)
        };
        // No shadow (VS Code is flat)
        let node_text = if dark {
            tokens::FG_DEFAULT
        } else {
            Color32::from_gray(25)
        };
        let node_text_dim = if dark {
            tokens::FG_DIM
        } else {
            Color32::from_gray(110)
        };
        let node_text_minimal = if dark {
            Color32::from_gray(120)
        } else {
            Color32::from_gray(150)
        };
        // Start/End terminal pill colors
        let start_pill_bg = tokens::SUCCESS;
        let start_pill_text = Color32::WHITE;
        let end_pill_bg = if dark {
            Color32::from_rgb(55, 55, 70)
        } else {
            Color32::from_gray(155)
        };
        let end_pill_text = Color32::WHITE;
        // Edge connection lines (theme-aware)
        let edge_color = if dark {
            tokens::EDGE_COLOR
        } else {
            Color32::from_gray(180)
        };

        // Pre-compute search state so it can be used throughout without borrow conflicts
        let search_query = self.canvas_search.to_lowercase();
        let search_active = !search_query.is_empty();

        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let origin = resp.rect.min;
        let z = self.canvas_zoom;
        let n = self.steps.len();
        let default_cols = default_canvas_cols(n);

        // ── Minimap layout constants (precomputed to gate input before the node loop) ──
        const MM_W: f32 = 160.0;
        const MM_H: f32 = 100.0;
        const MM_MARGIN: f32 = 8.0;
        const MM_PAD: f32 = 5.0;
        let mm_rect = egui::Rect::from_min_size(
            resp.rect.max - egui::vec2(MM_W + MM_MARGIN, MM_H + MM_MARGIN),
            egui::vec2(MM_W, MM_H),
        );
        let minimap_active = n > 0 && (n >= 15 || self.settings.minimap_show);
        let (mm_scale, mm_offset, mm_content_min) = if minimap_active {
            let mut min_cx = f32::MAX;
            let mut min_cy = f32::MAX;
            let mut max_cx = f32::MIN;
            let mut max_cy = f32::MIN;
            for i in 0..n {
                let p = self
                    .canvas_positions
                    .get(&i)
                    .copied()
                    .unwrap_or_else(|| default_canvas_pos(i, default_cols));
                min_cx = min_cx.min(p.x);
                min_cy = min_cy.min(p.y);
                max_cx = max_cx.max(p.x + NODE_W);
                max_cy = max_cy.max(p.y + NODE_H);
            }
            let content_w = (max_cx - min_cx).max(1.0);
            let content_h = (max_cy - min_cy).max(1.0);
            let mm_inner = mm_rect.shrink(MM_PAD);
            let sx = mm_inner.width() / content_w;
            let sy = mm_inner.height() / content_h;
            let s = sx.min(sy);
            let ox = mm_inner.min.x + (mm_inner.width() - content_w * s) / 2.0;
            let oy = mm_inner.min.y + (mm_inner.height() - content_h * s) / 2.0;
            (s, egui::pos2(ox, oy), egui::pos2(min_cx, min_cy))
        } else {
            (1.0, egui::Pos2::ZERO, egui::Pos2::ZERO)
        };
        let to_mm = move |p: egui::Pos2| -> egui::Pos2 {
            egui::pos2(
                mm_offset.x + (p.x - mm_content_min.x) * mm_scale,
                mm_offset.y + (p.y - mm_content_min.y) * mm_scale,
            )
        };
        let from_mm = move |p: egui::Pos2| -> egui::Pos2 {
            egui::pos2(
                (p.x - mm_offset.x) / mm_scale + mm_content_min.x,
                (p.y - mm_offset.y) / mm_scale + mm_content_min.y,
            )
        };
        // self.minimap_dragging persists across frames so drag-exit doesn't drop the latch
        let cursor_over_minimap = minimap_active
            && (self.minimap_dragging
                || ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|p| mm_rect.contains(p))
                    .unwrap_or(false));

        // Canvas background (theme-aware)
        painter.rect_filled(resp.rect, 0.0, cv_bg);

        // Background grid (every 40px in canvas space).
        // Uses line_segment instead of circle_filled to avoid O(W*H/grid²) draw calls
        // at low zoom — at z=0.25 circles would be ~20k calls per frame.
        {
            let grid = 40.0 * z;
            // Skip grid entirely when zoomed out below 20% (grid < 8px — invisible anyway)
            if grid >= 8.0 {
                let offset_x = (self.canvas_pan.x * z).rem_euclid(grid);
                let offset_y = (self.canvas_pan.y * z).rem_euclid(grid);
                let grid_color = if dark {
                    Color32::from_gray(42)
                } else {
                    Color32::from_gray(230)
                };
                let stroke = egui::Stroke::new(1.0, grid_color);
                let mut x = resp.rect.min.x + offset_x;
                while x <= resp.rect.max.x {
                    painter.line_segment(
                        [
                            egui::pos2(x, resp.rect.min.y),
                            egui::pos2(x, resp.rect.max.y),
                        ],
                        stroke,
                    );
                    x += grid;
                }
                let mut y = resp.rect.min.y + offset_y;
                while y <= resp.rect.max.y {
                    painter.line_segment(
                        [
                            egui::pos2(resp.rect.min.x, y),
                            egui::pos2(resp.rect.max.x, y),
                        ],
                        stroke,
                    );
                    y += grid;
                }
            }
        }

        // ── zoom / pan ─────────────────────────────────────────────────────
        if resp.hovered() && !cursor_over_minimap {
            let (scroll, ctrl, shift) = ui.input(|i| {
                (
                    i.smooth_scroll_delta,
                    i.modifiers.command,
                    i.modifiers.shift,
                )
            });
            if ctrl && scroll.y != 0.0 {
                let z_old = self.canvas_zoom;
                let scroll_clamped = scroll.y.clamp(-20.0, 20.0);
                let z_new = (z_old * (1.0 + scroll_clamped * 0.001)).clamp(0.25, 2.0);
                // Keep the canvas point under the cursor fixed: pan += cursor_offset * (1/z_new - 1/z_old)
                if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                    self.canvas_pan += (cursor - origin) * (1.0 / z_new - 1.0 / z_old);
                }
                self.canvas_zoom = z_new;
            } else {
                if scroll.y != 0.0 {
                    self.canvas_pan.y += scroll.y / z;
                }
                if scroll.x != 0.0 {
                    self.canvas_pan.x += scroll.x / z;
                }
            }
            // Pinch-to-zoom (macOS trackpad)
            let pinch = ui.input(|i| i.zoom_delta());
            if pinch != 1.0 {
                let z_old = self.canvas_zoom;
                let z_new = (z_old * pinch).clamp(0.25, 2.0);
                if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                    self.canvas_pan += (cursor - origin) * (1.0 / z_new - 1.0 / z_old);
                }
                self.canvas_zoom = z_new;
            }
            // Visual cue: crosshair cursor when Shift is held over background (lasso mode ready).
            // Suppressed when the pointer is over a node — Shift+click on a node is range-select.
            if shift && self.canvas_dragging.is_none() {
                let over_node = ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|cursor| {
                        (0..n).any(|i| {
                            let p = self
                                .canvas_positions
                                .get(&i)
                                .copied()
                                .unwrap_or_else(|| default_canvas_pos(i, default_cols));
                            egui::Rect::from_min_size(
                                origin + (p.to_vec2() + self.canvas_pan) * z,
                                egui::vec2(NODE_W * z, NODE_H * z),
                            )
                            .contains(cursor)
                        })
                    })
                    .unwrap_or(false);
                if !over_node {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                }
            }
        }
        // Click on empty background clears selection (not when clicking minimap)
        if resp.clicked() && self.canvas_dragging.is_none() && !cursor_over_minimap {
            self.selected = None;
            self.multi_selected.clear();
            self.canvas_selection_anchor = None;
        }
        // Middle-mouse pan (industry standard — Figma / Blender compatible)
        if resp.dragged_by(egui::PointerButton::Middle) && !cursor_over_minimap {
            self.canvas_pan += resp.drag_delta() / z;
        }

        // Pan on background drag, or lasso when Shift is held (not over minimap).
        // Checking dragged_by (not just drag_started) handles the race condition where the
        // user presses Shift slightly after beginning a drag — the lasso starts from the
        // current pointer position and the already-accumulated pan delta stays.
        if resp.dragged_by(egui::PointerButton::Primary)
            && self.canvas_dragging.is_none()
            && !cursor_over_minimap
        {
            let shift = ui.input(|i| i.modifiers.shift);
            if shift && self.canvas_lasso.is_none() {
                // Start lasso (converts ongoing pan to lasso on Shift press)
                let start = resp.interact_pointer_pos().unwrap_or(resp.rect.min);
                self.canvas_lasso = Some((start, start));
                self.canvas_lasso_additive = true; // shift guard above guarantees true
            } else if let Some((start, _)) = self.canvas_lasso {
                let cursor = resp.interact_pointer_pos().unwrap_or(resp.rect.min);
                self.canvas_lasso = Some((start, cursor));
            } else {
                self.canvas_pan += resp.drag_delta() / z;
            }
        }

        // Collect node screen positions
        let screen_positions: Vec<egui::Pos2> = (0..n)
            .map(|i| {
                let p = self
                    .canvas_positions
                    .get(&i)
                    .copied()
                    .unwrap_or_else(|| default_canvas_pos(i, default_cols));
                origin + (p.to_vec2() + self.canvas_pan) * z
            })
            .collect();

        // ── Start / End terminals (Power Automate style) ──────────────────
        // Drawn before edges so they sit behind all regular nodes.
        const SE_W: f32 = 80.0; // logical pill width
        const SE_H: f32 = 22.0; // logical pill height
        const SE_GAP: f32 = 28.0; // gap between pill and nearest node

        if n > 0 {
            let first_sp = screen_positions[0];
            let last_sp = screen_positions[n - 1];
            let ctrl_v = Vec2::new(0.0, SE_GAP * z * 0.4);

            // ── Start pill ──
            let start_c = first_sp + Vec2::new(NODE_W * z / 2.0, -(SE_GAP * z + SE_H * z / 2.0));
            painter.rect_filled(
                Rect::from_center_size(start_c, Vec2::new(SE_W * z, SE_H * z)),
                SE_H * z / 2.0,
                start_pill_bg,
            );
            if z >= 0.5 {
                let s = crate::i18n::S::for_lang(&self.settings.lang);
                painter.text(
                    start_c,
                    Align2::CENTER_CENTER,
                    s.hint_start_terminal,
                    FontId::proportional(10.0 * z),
                    start_pill_text,
                );
            }
            // Edge Start → first node
            let se_from = start_c + Vec2::new(0.0, SE_H * z / 2.0);
            let se_to = first_sp + Vec2::new(NODE_W * z / 2.0, 0.0);
            painter.add(egui::Shape::CubicBezier(
                CubicBezierShape::from_points_stroke(
                    [se_from, se_from + ctrl_v, se_to - ctrl_v, se_to],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new((1.5 * z).max(1.0), edge_color),
                ),
            ));
            painter.arrow(
                se_to - Vec2::new(0.0, 6.0 * z),
                Vec2::new(0.0, 6.0 * z),
                Stroke::new((1.5 * z).max(1.0), edge_color),
            );

            // ── End pill ──
            let end_c =
                last_sp + Vec2::new(NODE_W * z / 2.0, NODE_H * z + SE_GAP * z + SE_H * z / 2.0);
            painter.rect_filled(
                Rect::from_center_size(end_c, Vec2::new(SE_W * z, SE_H * z)),
                SE_H * z / 2.0,
                end_pill_bg,
            );
            if z >= 0.5 {
                let s = crate::i18n::S::for_lang(&self.settings.lang);
                painter.text(
                    end_c,
                    Align2::CENTER_CENTER,
                    s.hint_end_terminal,
                    FontId::proportional(10.0 * z),
                    end_pill_text,
                );
            }
            // Edge last node → End
            let ee_from = last_sp + Vec2::new(NODE_W * z / 2.0, NODE_H * z);
            let ee_to = end_c - Vec2::new(0.0, SE_H * z / 2.0);
            painter.add(egui::Shape::CubicBezier(
                CubicBezierShape::from_points_stroke(
                    [ee_from, ee_from + ctrl_v, ee_to - ctrl_v, ee_to],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new((1.5 * z).max(1.0), edge_color),
                ),
            ));
            painter.arrow(
                ee_to - Vec2::new(0.0, 6.0 * z),
                Vec2::new(0.0, 6.0 * z),
                Stroke::new((1.5 * z).max(1.0), edge_color),
            );
        }

        // ── Draw edges (behind nodes) ──────────────────────────────────────
        let mut insert_after_idx: Option<usize> = None;
        for i in 0..n.saturating_sub(1) {
            let step_key = get_step_key(&self.steps[i]);
            let is_compound = matches!(
                step_key,
                "if" | "foreach"
                    | "repeat"
                    | "while"
                    | "do_while"
                    | "try_catch"
                    | "group"
                    | "switch"
            );
            let expand_offset = if self.expanded_steps.contains(&i) {
                let branches = get_inner_steps(&self.steps[i]);
                if branches.is_empty() {
                    0.0
                } else {
                    (branches.len() as f32 * 18.0 + 8.0) * z
                }
            } else {
                0.0
            };
            let from =
                screen_positions[i] + Vec2::new(NODE_W * z / 2.0, NODE_H * z + expand_offset);
            let to = screen_positions[i + 1] + Vec2::new(NODE_W * z / 2.0, 0.0);
            let dy = to.y - from.y;
            let ctrl_mag = if dy.abs() > 20.0 * z {
                40.0 * z
            } else {
                dy.abs() * 0.5 + 10.0 * z
            };
            let ctrl = if dy >= 0.0 {
                Vec2::new(0.0, ctrl_mag)
            } else {
                Vec2::new(0.0, -ctrl_mag)
            };
            let stroke_color = if is_compound {
                Color32::from_rgb(120, 100, 60)
            } else {
                edge_color
            };
            // Ensure edges remain visible at extreme zoom-out (min 1px)
            let stroke_width = if is_compound {
                (1.0 * z).max(1.0)
            } else {
                (1.5 * z).max(1.0)
            };
            painter.add(egui::Shape::CubicBezier(
                CubicBezierShape::from_points_stroke(
                    [from, from + ctrl, to - ctrl, to],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(stroke_width, stroke_color),
                ),
            ));
            painter.arrow(
                to - Vec2::new(0.0, 6.0 * z),
                Vec2::new(0.0, 5.0 * z),
                Stroke::new(stroke_width, stroke_color),
            );
            // Branch label for compound steps — anchored just below the source node
            if is_compound && z >= 0.6 {
                let branch_label = match get_step_key(&self.steps[i]) {
                    "if" => Some("then/else"),
                    "foreach" | "while" | "do_while" | "repeat" | "group" => Some("do"),
                    "try_catch" => Some("try/catch"),
                    _ => None,
                };
                if let Some(lbl) = branch_label {
                    painter.text(
                        from + Vec2::new(8.0 * z, 4.0 * z),
                        Align2::LEFT_TOP,
                        lbl,
                        FontId::proportional(9.0 * z),
                        Color32::from_rgb(150, 130, 80),
                    );
                }
            }
            // Midpoint "+" insertion button — only rendered when pointer is near this edge
            if z >= 0.6 {
                let mid = from.lerp(to, 0.5);
                let near_hover = ui
                    .input(|inp| inp.pointer.hover_pos())
                    .is_some_and(|hp| hp.distance(mid) <= 40.0 * z);
                if near_hover {
                    let btn_size = (14.0 * z).max(18.0);
                    let btn_rect =
                        egui::Rect::from_center_size(mid, egui::vec2(btn_size, btn_size));
                    let btn_resp = ui.allocate_rect(btn_rect, Sense::click());
                    if btn_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if btn_resp.hovered() || btn_resp.clicked() {
                        painter.circle_filled(mid, btn_size / 2.0, Color32::from_rgb(50, 90, 160));
                        painter.text(
                            mid,
                            Align2::CENTER_CENTER,
                            "+",
                            FontId::proportional(10.0 * z),
                            Color32::WHITE,
                        );
                        if btn_resp.clicked() {
                            insert_after_idx = Some(i);
                        }
                    } else {
                        painter.circle_filled(
                            mid,
                            btn_size / 2.0,
                            Color32::from_rgba_premultiplied(50, 80, 140, 80),
                        );
                        painter.text(
                            mid,
                            Align2::CENTER_CENTER,
                            "+",
                            FontId::proportional(10.0 * z),
                            if dark {
                                Color32::from_gray(220)
                            } else {
                                Color32::from_gray(80)
                            },
                        );
                    }
                }
            }
        }

        if let Some(ins_idx) = insert_after_idx {
            self.select_step(ins_idx);
            self.add_menu_open = true;
            self.add_menu_just_opened = true;
            self.add_filter.clear();
        }

        // Draw active edge-drag line (pending connection preview)
        if let Some((src_idx, end_pos)) = self.canvas_edge_drag {
            if let Some(&src_sp) = screen_positions.get(src_idx) {
                let from_p = src_sp + Vec2::new(NODE_W * z / 2.0, NODE_H * z);
                let dy = (end_pos.y - from_p.y).abs().max(40.0 * z);
                let ctrl = Vec2::new(0.0, dy * 0.5);
                painter.add(egui::Shape::CubicBezier(
                    CubicBezierShape::from_points_stroke(
                        [from_p, from_p + ctrl, end_pos - ctrl, end_pos],
                        false,
                        Color32::TRANSPARENT,
                        Stroke::new(2.0 * z, tokens::ACCENT),
                    ),
                ));
                painter.circle_filled(from_p, 5.0 * z, tokens::ACCENT);
                painter.circle_filled(end_pos, 5.0 * z, tokens::ACCENT);
                // Label anchored to the source port — doesn't move with cursor
                let s = crate::i18n::S::for_lang(&self.settings.lang);
                painter.text(
                    from_p + egui::Vec2::new(10.0 * z, -12.0 * z),
                    egui::Align2::LEFT_BOTTOM,
                    s.hint_drag_move,
                    egui::FontId::proportional(10.0 * z),
                    tokens::ACCENT,
                );
            }
        }

        // ── Collect interactions (to avoid borrow conflicts) ───────────────
        let mut drag_started_node: Option<(usize, egui::Vec2)> = None;
        let mut drag_delta_node: Option<(usize, egui::Pos2)> = None;
        let mut drag_ended = false;
        let mut canvas_click_modifier: Option<(usize, bool, bool)> = None; // (idx, shift, cmd)
        let mut double_clicked_node: Option<usize> = None;
        let mut canvas_ctx_action: Option<CanvasContextAction> = None;
        let mut badge_toggle_idx: Option<usize> = None;
        let mut panel_click_step: Option<(usize, String)> = None; // (parent_idx, branch_name)
        let mut edge_drag_start_idx: Option<usize> = None;
        let mut edge_drag_done = false;
        let mut edge_drag_pos_update: Option<egui::Pos2> = None;

        for (idx, _step) in self.steps.iter().enumerate() {
            let screen_pos = screen_positions[idx];
            let node_rect = Rect::from_min_size(screen_pos, Vec2::new(NODE_W * z, NODE_H * z));
            let node_resp = ui.allocate_rect(node_rect, Sense::click_and_drag());

            // ── Interaction collection (always, even for off-screen nodes) ──
            if node_resp.drag_started() {
                let cursor = ui.input(|i| i.pointer.interact_pos()).unwrap_or(screen_pos);
                let port_center = screen_pos + Vec2::new(NODE_W * z / 2.0, NODE_H * z);
                if cursor.distance(port_center) <= 10.0 * z {
                    edge_drag_start_idx = Some(idx);
                } else {
                    drag_started_node = Some((idx, cursor - screen_pos));
                }
            }
            if node_resp.dragged() {
                if self.canvas_edge_drag.map(|(i, _)| i) == Some(idx) {
                    edge_drag_pos_update =
                        Some(ui.input(|i| i.pointer.interact_pos()).unwrap_or(screen_pos));
                } else if let Some((di, _)) = self.canvas_dragging {
                    if di == idx {
                        let cursor = ui.input(|i| i.pointer.interact_pos()).unwrap_or(screen_pos);
                        let drag_offset = self.canvas_dragging.unwrap().1;
                        // Convert screen cursor to canvas space: subtract origin and drag
                        // offset (both in screen pixels), divide by zoom, subtract pan.
                        let screen_rel = cursor - origin - drag_offset;
                        let canvas_pos = egui::pos2(
                            screen_rel.x / z - self.canvas_pan.x,
                            screen_rel.y / z - self.canvas_pan.y,
                        );
                        drag_delta_node = Some((idx, canvas_pos));
                    }
                }
            }
            if node_resp.drag_stopped() {
                if self.canvas_edge_drag.map(|(i, _)| i) == Some(idx) {
                    edge_drag_done = true;
                } else {
                    drag_ended = true;
                }
            }
            if node_resp.clicked() {
                let click_pos = node_resp.interact_pointer_pos().unwrap_or_default();
                let badge_rect = egui::Rect::from_min_size(
                    node_rect.max - Vec2::new(38.0 * z, 22.0 * z),
                    Vec2::new(34.0 * z, 20.0 * z),
                );
                let sk = get_step_key(&self.steps[idx]);
                let is_cpd = matches!(
                    sk,
                    "if" | "foreach"
                        | "repeat"
                        | "while"
                        | "do_while"
                        | "try_catch"
                        | "group"
                        | "switch"
                );
                // ✕ delete button in top-right corner (visible on hover at z >= 0.6)
                let x_btn_center = node_rect.min + Vec2::new(NODE_W * z - 9.0 * z, 9.0 * z);
                let x_btn_rect =
                    egui::Rect::from_center_size(x_btn_center, egui::vec2(14.0 * z, 14.0 * z));
                if z >= 0.6 && x_btn_rect.contains(click_pos) {
                    canvas_ctx_action = Some(CanvasContextAction::Delete(idx));
                } else if is_cpd && z >= 0.7 && badge_rect.contains(click_pos) {
                    badge_toggle_idx = Some(idx);
                } else {
                    let (shift, cmd) = ui.input(|i| (i.modifiers.shift, i.modifiers.command));
                    canvas_click_modifier = Some((idx, shift, cmd));
                }
            }
            if node_resp.double_clicked() {
                double_clicked_node = Some(idx);
            }

            // Right-click context menu
            node_resp.context_menu(|ui| {
                let s = crate::i18n::S::for_lang(&self.settings.lang);
                {
                    let is_dis = crate::state::EditorApp::step_is_disabled(&self.steps[idx]);
                    let toggle_label = if is_dis { s.node_enable } else { s.node_disable };
                    if ui.button(toggle_label)
                        .on_hover_text(if is_dis { "Enable this step" } else { "Disable this step" })
                        .clicked()
                    {
                        canvas_ctx_action = Some(CanvasContextAction::ToggleEnabled(idx));
                        ui.close();
                    }
                    ui.separator();
                }
                if ui.button(s.ctx_copy)
                    .on_hover_text("Cmd+C")
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::CopySelected);
                    ui.close();
                }
                if ui.button(s.ctx_cut)
                    .on_hover_text("Cmd+X")
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::CutSelected);
                    ui.close();
                }
                if ui.button(s.ctx_duplicate)
                    .on_hover_text("Cmd+D")
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::Duplicate(idx));
                    ui.close();
                }
                if !self.step_clipboard.is_empty()
                    && ui.button(s.ctx_paste)
                        .on_hover_text("Cmd+V")
                        .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::Paste);
                    ui.close();
                }
                ui.separator();
                if ui.button(s.ctx_delete)
                    .on_hover_text("Delete")
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::Delete(idx));
                    ui.close();
                }
                ui.separator();
                if ui.button(s.ctx_open_in_list)
                    .on_hover_text("Switch to List view")
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::OpenInList(idx));
                    ui.close();
                }
                if self.run_child.is_none() {
                    let label = s.ctx_run_from_here.replace("{}", &(idx + 1).to_string());
                    if ui
                        .button(&label)
                        .on_hover_text("Execute this step and subsequent steps only")
                        .clicked()
                    {
                        canvas_ctx_action = Some(CanvasContextAction::RunFrom(idx));
                        ui.close();
                    }
                }
                if self.multi_selected.len() >= 2 && self.multi_selected.contains(&idx) {
                    ui.separator();
                    let align_label = s.ctx_align_label.replace("{}", &self.multi_selected.len().to_string());
                    ui.weak(&align_label);
                    if ui.button(s.ctx_align_left)
                        .on_hover_text("Align selected nodes to the leftmost")
                        .clicked()
                    {
                        canvas_ctx_action = Some(CanvasContextAction::AlignLeft);
                        ui.close();
                    }
                    if ui.button(s.ctx_align_top)
                        .on_hover_text("Align selected nodes to the topmost")
                        .clicked()
                    {
                        canvas_ctx_action = Some(CanvasContextAction::AlignTop);
                        ui.close();
                    }
                    if self.multi_selected.len() >= 3 {
                        if ui.button(s.ctx_distribute_h)
                            .on_hover_text("Distribute selected nodes horizontally")
                            .clicked()
                        {
                            canvas_ctx_action = Some(CanvasContextAction::DistributeH);
                            ui.close();
                        }
                        if ui.button(s.ctx_distribute_v)
                            .on_hover_text("Distribute selected nodes vertically")
                            .clicked()
                        {
                            canvas_ctx_action = Some(CanvasContextAction::DistributeV);
                            ui.close();
                        }
                    }
                }
            });

            // Hoist these before culling so the tooltip works for partially off-screen nodes.
            let hovered = node_resp.hovered();
            let step = &self.steps[idx];
            let full_label = step_summary(step);

            // Cursor feedback: PointingHand on delete button, Grab elsewhere, Grabbing while dragging
            let x_btn_center = node_rect.min + Vec2::new(NODE_W * z - 9.0 * z, 9.0 * z);
            let x_btn_hovered = z >= 0.6
                && ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|p| p.distance(x_btn_center) <= 9.0 * z)
                    .unwrap_or(false);
            if self.canvas_dragging.map(|(di, _)| di) == Some(idx) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else if hovered && x_btn_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            } else if hovered && self.canvas_edge_drag.is_none() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }

            // Show tooltip on hover: error > truncated label > double-click hint
            if hovered {
                if let Some(err_msg) = self.canvas_error_steps.get(&idx) {
                    let msg = err_msg.clone();
                    node_resp.show_tooltip_ui(|ui| {
                        ui.colored_label(tokens::ERROR, &msg);
                    });
                } else if full_label.chars().count() > 32 {
                    node_resp.show_tooltip_ui(|ui| {
                        ui.label(&full_label);
                    });
                } else if self.selected != Some(idx) {
                    let s = crate::i18n::S::for_lang(&self.settings.lang);
                    node_resp.on_hover_text(s.hint_dblclick_list);
                }
            }

            // ── Draw (skip nodes fully outside the viewport) ────────────────
            if !resp.rect.intersects(node_rect) {
                continue;
            }

            let selected = self.selected == Some(idx);
            let is_multi_only = self.multi_selected.contains(&idx) && !selected;
            let is_running = self.current_run_step == Some(idx);
            let is_disabled = crate::state::EditorApp::step_is_disabled(step);
            let key = get_step_key(step);
            let cat_color = if is_disabled {
                if dark {
                    Color32::from_gray(55)
                } else {
                    Color32::from_gray(185)
                }
            } else {
                category_color(step_key_category(key))
            };
            // Blend category color into base background
            let base_bg = if selected {
                node_bg_selected
            } else if is_running {
                node_bg_running
            } else if is_disabled {
                node_bg_disabled
            } else {
                node_bg
            };
            // VS Code flat style: no background blending with category color.
            // Category color is expressed ONLY by the left stripe (3 px).
            // No drop shadow.
            let (border, border_w) = if selected {
                (tokens::ACCENT, (1.5 * z).max(1.0))
            } else if is_multi_only {
                (tokens::ACCENT.gamma_multiply(0.7), (1.5 * z).max(1.0))
            } else {
                (node_border, 0.5) // near-invisible for non-selected nodes
            };

            painter.rect_filled(node_rect, tokens::ROUNDING_CARD, base_bg);
            painter.rect_stroke(
                node_rect,
                tokens::ROUNDING_CARD,
                Stroke::new(border_w, border),
                egui::StrokeKind::Inside,
            );
            // Snap flash: teal border for SNAP_FLASH_MS after a grid-snap event.
            if let Some(&flash_at) = self.snap_flashes.get(&idx) {
                let flash_dur = std::time::Duration::from_millis(crate::tokens::SNAP_FLASH_MS);
                if flash_at.elapsed() < flash_dur {
                    let alpha = {
                        let t = flash_at.elapsed().as_secs_f32() / flash_dur.as_secs_f32();
                        ((1.0 - t) * 255.0) as u8
                    };
                    painter.rect_stroke(
                        node_rect.expand(2.0 * z),
                        tokens::ROUNDING_CARD,
                        Stroke::new(
                            (2.0 * z).max(1.5),
                            crate::tokens::SNAP_FLASH.gamma_multiply(alpha as f32 / 255.0),
                        ),
                        egui::StrokeKind::Outside,
                    );
                }
            }
            if is_multi_only {
                painter.rect_stroke(
                    node_rect.expand(1.5 * z),
                    tokens::ROUNDING_CARD,
                    egui::Stroke::new((1.5 * z).max(1.5), tokens::ACCENT.gamma_multiply(0.6)),
                    egui::StrokeKind::Outside,
                );
            }
            // Hover: subtle background tint (VS Code: no border on hover)
            if hovered && !selected && !is_multi_only {
                painter.rect_filled(
                    node_rect,
                    tokens::ROUNDING_CARD,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 8),
                );
            }

            // ✕ delete button — top-right corner, visible on hover at z >= 0.6
            if hovered && z >= 0.6 {
                let x_center = node_rect.min + Vec2::new(NODE_W * z - 9.0 * z, 9.0 * z);
                let x_is_hovered = ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|p| p.distance(x_center) <= 7.0 * z)
                    .unwrap_or(false);
                let x_bg = if x_is_hovered {
                    Color32::from_rgb(210, 60, 60)
                } else {
                    Color32::from_rgba_premultiplied(160, 60, 60, 180)
                };
                painter.circle_filled(x_center, 7.0 * z, x_bg);
                painter.text(
                    x_center,
                    Align2::CENTER_CENTER,
                    "×",
                    FontId::proportional(8.0 * z),
                    Color32::WHITE, // White on red delete button for high contrast
                );
            }

            // Left color stripe — thin (3 px), category color only element.
            let stripe =
                Rect::from_min_size(node_rect.min, Vec2::new((3.0 * z).max(2.0), NODE_H * z));
            painter.rect_filled(stripe, tokens::ROUNDING_CARD, cat_color);

            // Progressive node content based on zoom level
            if z < 0.5 {
                // Minimal: only step index centered — text too small to read otherwise
                painter.text(
                    node_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{}", idx + 1),
                    FontId::proportional(10.0 * z),
                    node_text_minimal,
                );
            } else {
                let (idx_y, label_y) = if z >= 1.5 {
                    (NODE_H * z * 0.22, NODE_H * z * 0.52)
                } else {
                    (NODE_H * z * 0.3, NODE_H * z * 0.68)
                };
                // Disabled overlay: "⊘" in top-right
                if is_disabled && z >= 0.5 {
                    painter.text(
                        node_rect.right_top() + Vec2::new(-6.0 * z, 6.0 * z),
                        Align2::RIGHT_TOP,
                        "⊘",
                        FontId::proportional((11.0 * z).max(9.0)),
                        Color32::from_rgba_premultiplied(180, 80, 80, 200),
                    );
                }
                painter.text(
                    node_rect.min + Vec2::new(10.0 * z, idx_y),
                    Align2::LEFT_CENTER,
                    format!("{}", idx + 1),
                    FontId::proportional((9.0 * z).max(9.0)),
                    if is_disabled {
                        node_text_dim
                    } else {
                        node_text
                    },
                );
                const MAX_LABEL_CHARS: usize = 32;
                let label = if full_label.chars().count() > MAX_LABEL_CHARS {
                    let s: String = full_label.chars().take(MAX_LABEL_CHARS - 1).collect();
                    format!("{}…", s)
                } else {
                    full_label.clone()
                };
                painter.text(
                    node_rect.min + Vec2::new(10.0 * z, label_y),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional((11.0 * z).max(10.0)),
                    if is_disabled {
                        Color32::from_gray(100)
                    } else {
                        Color32::from_gray(228)
                    },
                );
                // High-zoom third line: step type
                if z >= 1.5 {
                    painter.text(
                        node_rect.min + Vec2::new(10.0 * z, NODE_H * z * 0.82),
                        Align2::LEFT_CENTER,
                        key,
                        FontId::proportional(8.5 * z),
                        Color32::from_rgba_premultiplied(160, 170, 200, 170),
                    );
                }
            }

            // Badge: show child step count for compound steps; click to expand/collapse
            let step_key_badge = get_step_key(step);
            let is_compound_node = matches!(
                step_key_badge,
                "if" | "foreach"
                    | "repeat"
                    | "while"
                    | "do_while"
                    | "try_catch"
                    | "group"
                    | "switch"
            );
            if is_compound_node {
                let branches = get_inner_steps(step);
                let child_count: usize = branches.iter().map(|(_, v)| v.len()).sum();
                if child_count > 0 {
                    let is_expanded = self.expanded_steps.contains(&idx);
                    if z >= 0.7 {
                        let badge_text = if is_expanded {
                            format!("-{child_count}")
                        } else {
                            format!("+{child_count}")
                        };
                        let badge_pos = node_rect.max - Vec2::new(4.0 * z, 2.0 * z);
                        painter.text(
                            badge_pos,
                            Align2::RIGHT_BOTTOM,
                            badge_text,
                            FontId::proportional(9.0 * z),
                            Color32::from_rgba_premultiplied(255, 200, 80, 200),
                        );
                    }
                    // Expand panel when toggled open (capped at 8 rows)
                    if is_expanded && !branches.is_empty() {
                        const MAX_ROWS: usize = 8;
                        let has_more = branches.len() > MAX_ROWS;
                        let visible = branches.len().min(MAX_ROWS);
                        let row_count = visible + usize::from(has_more);
                        let row_h = 18.0 * z;
                        let panel_h = row_count as f32 * row_h + 6.0 * z;
                        let panel_top_if_above = node_rect.min.y - panel_h - 2.0 * z;
                        let panel_y = if panel_top_if_above >= origin.y {
                            panel_top_if_above
                        } else {
                            node_rect.max.y + 2.0 * z
                        };
                        let panel_rect = egui::Rect::from_min_size(
                            egui::Pos2::new(node_rect.min.x, panel_y),
                            Vec2::new(NODE_W * z, panel_h),
                        );
                        painter.rect_filled(panel_rect, 4.0 * z, Color32::from_rgb(28, 28, 32));
                        painter.rect_stroke(
                            panel_rect,
                            4.0 * z,
                            Stroke::new(1.0 * z, Color32::from_rgb(60, 60, 80)),
                            egui::StrokeKind::Outside,
                        );
                        for (row, (label, steps_vec)) in branches.iter().take(MAX_ROWS).enumerate()
                        {
                            let row_y = panel_rect.min.y + 3.0 * z + row as f32 * row_h;
                            let row_rect = Rect::from_min_size(
                                egui::Pos2::new(node_rect.min.x, row_y),
                                Vec2::new(NODE_W * z, row_h),
                            );
                            let row_resp = ui.allocate_rect(row_rect, Sense::click());
                            if row_resp.hovered() {
                                painter.rect_filled(
                                    row_rect,
                                    2.0 * z,
                                    Color32::from_rgba_premultiplied(255, 255, 255, 18),
                                );
                            }
                            if row_resp.clicked() {
                                panel_click_step = Some((idx, label.to_string()));
                            }
                            // Small "+" on the right side of each branch row to add a step
                            let add_btn_size = 14.0 * z;
                            let add_btn_rect = Rect::from_min_size(
                                egui::Pos2::new(
                                    node_rect.max.x - add_btn_size - 4.0 * z,
                                    row_y + (row_h - add_btn_size) / 2.0,
                                ),
                                Vec2::splat(add_btn_size),
                            );
                            let add_resp = ui.allocate_rect(add_btn_rect, Sense::click());
                            let add_color = if add_resp.hovered() {
                                tokens::ACCENT
                            } else {
                                Color32::from_gray(120)
                            };
                            painter.text(
                                add_btn_rect.center(),
                                Align2::CENTER_CENTER,
                                "+",
                                FontId::proportional(10.0 * z),
                                add_color,
                            );
                            if add_resp.clicked() {
                                // Open add-step picker targeting this specific branch
                                self.add_branch_target = Some((idx, label.to_string()));
                                self.add_menu_open = true;
                                self.add_menu_just_opened = true;
                            }
                            let text_color = if row_resp.hovered() {
                                Color32::from_rgb(220, 210, 180)
                            } else {
                                Color32::from_rgb(180, 170, 150)
                            };
                            let text = format!("{}  {} steps", label, steps_vec.len());
                            painter.text(
                                egui::Pos2::new(node_rect.min.x + 8.0 * z, row_y + row_h * 0.5),
                                Align2::LEFT_CENTER,
                                &text,
                                FontId::proportional(9.5 * z),
                                text_color,
                            );
                        }
                        if has_more {
                            let more = branches.len() - MAX_ROWS;
                            let row_y = panel_rect.min.y + 3.0 * z + visible as f32 * row_h;
                            painter.text(
                                egui::Pos2::new(node_rect.min.x + 8.0 * z, row_y + row_h * 0.5),
                                Align2::LEFT_CENTER,
                                format!("… {} more", more),
                                FontId::proportional(9.5 * z),
                                Color32::from_gray(120),
                            );
                        }
                    }
                }
            }
            if is_running {
                painter.rect_stroke(
                    node_rect.expand(2.0 * z),
                    4.0 * z,
                    egui::Stroke::new((2.0 * z).max(1.0), tokens::WARNING),
                    egui::StrokeKind::Outside,
                );
            }
            if self.canvas_error_steps.contains_key(&idx) {
                painter.rect_stroke(
                    node_rect.expand((2.0 * z).max(1.0)),
                    4.0 * z,
                    egui::Stroke::new((2.0 * z).max(1.0), tokens::ERROR),
                    egui::StrokeKind::Outside,
                );
            }

            // Green left stripe for completed steps (only when not running/errored)
            if self.canvas_completed_steps.contains(&idx)
                && !is_running
                && !self.canvas_error_steps.contains_key(&idx)
            {
                let stripe = egui::Rect::from_min_size(
                    node_rect.min,
                    Vec2::new((6.0 * z).max(3.0), NODE_H * z),
                );
                painter.rect_filled(stripe, 4.0 * z, tokens::SUCCESS);
            }

            // Output port dot at bottom-center (edge drag / reorder)
            let port_center = node_rect.min + Vec2::new(NODE_W * z / 2.0, NODE_H * z);
            let port_hovered = ui
                .input(|i| i.pointer.hover_pos())
                .map(|p| p.distance(port_center) <= 10.0 * z)
                .unwrap_or(false);
            let in_edge_drag = self.canvas_edge_drag.is_some();
            if z >= 0.5 {
                let dot_color = if port_hovered && !in_edge_drag {
                    Color32::from_rgb(180, 200, 255)
                } else if in_edge_drag {
                    Color32::from_rgb(100, 180, 255)
                } else {
                    Color32::from_rgba_premultiplied(140, 140, 160, 90)
                };
                // Small circle = output port (drag to reorder)
                painter.circle_filled(port_center, (3.5 * z).max(3.0), dot_color);
                // Drag affordance is communicated by the Grab cursor (set above).
                if port_hovered && !in_edge_drag {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    let tip_rect =
                        egui::Rect::from_center_size(port_center, egui::vec2(20.0 * z, 12.0 * z));
                    let tip_resp = ui.allocate_rect(tip_rect, Sense::hover());
                    let s = crate::i18n::S::for_lang(&self.settings.lang);
                    tip_resp.on_hover_text(s.hint_drag_reorder);
                }
            }
            // Input port highlight when an edge drag is hovering over this node
            if in_edge_drag {
                let hover_over = ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|p| node_rect.contains(p))
                    .unwrap_or(false);
                if hover_over && self.canvas_edge_drag.map(|(i, _)| i) != Some(idx) {
                    // Horizontal "insert before" indicator line across the top of the target node
                    let line_y = node_rect.min.y - 3.0 * z;
                    painter.line_segment(
                        [
                            egui::pos2(node_rect.min.x, line_y),
                            egui::pos2(node_rect.max.x, line_y),
                        ],
                        egui::Stroke::new(2.5 * z, tokens::SUCCESS),
                    );
                    // Small diamond at the left end
                    let d = 4.0 * z;
                    let cx = node_rect.min.x;
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            egui::pos2(cx, line_y - d),
                            egui::pos2(cx + d, line_y),
                            egui::pos2(cx, line_y + d),
                            egui::pos2(cx - d, line_y),
                        ],
                        tokens::SUCCESS,
                        egui::Stroke::NONE,
                    ));
                }
            }
            // Search filter overlay: highlight matches, dim non-matches
            // Error/running nodes are never dimmed (state priority: error > running > search)
            if search_active {
                let is_match = step_matches(step, &search_query);
                let is_special = is_running || self.canvas_error_steps.contains_key(&idx);
                if !is_match && !is_special {
                    painter.rect_filled(
                        node_rect,
                        4.0 * z,
                        Color32::from_rgba_premultiplied(0, 0, 0, 155),
                    );
                } else if is_match {
                    painter.rect_stroke(
                        node_rect.expand(3.0 * z),
                        4.0 * z,
                        Stroke::new(2.0 * z, Color32::from_rgb(80, 160, 255)),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }

        // ── Edge-drag state updates ────────────────────────────────────────
        if let Some(src) = edge_drag_start_idx {
            let cursor = ui.input(|i| i.pointer.interact_pos()).unwrap_or_default();
            self.canvas_edge_drag = Some((src, cursor));
        }
        if let Some(pos) = edge_drag_pos_update {
            if let Some((_, ref mut p)) = self.canvas_edge_drag {
                *p = pos;
            }
        }
        if edge_drag_done {
            if let Some((src_idx, end_pos)) = self.canvas_edge_drag.take() {
                // Find target node
                let mut target: Option<usize> = None;
                for (tgt_idx, &sp) in screen_positions.iter().enumerate() {
                    let nr = Rect::from_min_size(sp, Vec2::new(NODE_W * z, NODE_H * z));
                    if nr.contains(end_pos) && tgt_idx != src_idx {
                        target = Some(tgt_idx);
                        break;
                    }
                }
                if let Some(tgt_idx) = target {
                    self.push_undo();
                    let src_pos = self.canvas_positions.get(&src_idx).copied();
                    let step = self.steps.remove(src_idx);
                    let insert_at = if tgt_idx > src_idx {
                        tgt_idx - 1
                    } else {
                        tgt_idx
                    };
                    Self::canvas_shift_positions(&mut self.canvas_positions, src_idx, -1);
                    Self::form_edit_buffers_shift(&mut self.form_edit_buffers, src_idx, -1);
                    Self::canvas_shift_positions(&mut self.canvas_positions, insert_at, 1);
                    Self::form_edit_buffers_shift(&mut self.form_edit_buffers, insert_at, 1);
                    if let Some(pos) = src_pos {
                        self.canvas_positions.insert(insert_at, pos);
                    }
                    self.steps.insert(insert_at, step);
                    self.selected = Some(insert_at);
                    self.dirty = true;
                    self.canvas_layout_dirty = true;
                }
            }
        }

        // Apply interactions after the draw loop
        if let Some((idx, offset)) = drag_started_node {
            self.canvas_dragging = Some((idx, offset));
        }
        if let Some((idx, canvas_pos)) = drag_delta_node {
            if !self.undo_pushed_for_current_drag {
                self.push_undo_forced();
                self.undo_pushed_for_current_drag = true;
            }
            // Apply snap during drag for single-node moves; for multi-select, compute
            // delta from unsnapped positions so relative offsets are preserved exactly.
            // Snap is also applied once at drag_stopped for the final position.
            let snapped_pos = if self.settings.canvas_snap {
                const SNAP: f32 = 40.0;
                egui::pos2(
                    (canvas_pos.x / SNAP).round() * SNAP,
                    (canvas_pos.y / SNAP).round() * SNAP,
                )
            } else {
                canvas_pos
            };
            // Record snap flash when the position actually changed due to snapping.
            if self.settings.canvas_snap && snapped_pos != canvas_pos {
                self.snap_flashes.insert(idx, std::time::Instant::now());
            }
            if self.multi_selected.len() > 1 && self.multi_selected.contains(&idx) {
                let old_pos = self
                    .canvas_positions
                    .get(&idx)
                    .copied()
                    .unwrap_or(canvas_pos);
                let delta = canvas_pos - old_pos;
                let selected_indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
                for sel_idx in selected_indices {
                    if let Some(pos) = self.canvas_positions.get(&sel_idx).copied() {
                        self.canvas_positions.insert(sel_idx, pos + delta);
                    }
                }
            } else {
                self.canvas_positions.insert(idx, snapped_pos);
            }
        }
        if drag_ended {
            // Apply snap once at drag release so delta math stays exact during the drag.
            // Only if the drag actually moved (undo_pushed_for_current_drag is set by
            // drag_delta_node) — a press-and-release click must not mutate positions.
            if self.settings.canvas_snap && self.undo_pushed_for_current_drag {
                const SNAP: f32 = 40.0;
                let snap_pos = |p: egui::Pos2| {
                    egui::pos2((p.x / SNAP).round() * SNAP, (p.y / SNAP).round() * SNAP)
                };
                let snap_indices: Vec<usize> = if self.multi_selected.len() > 1 {
                    self.multi_selected.iter().cloned().collect()
                } else if let Some((di, _)) = self.canvas_dragging {
                    vec![di]
                } else {
                    vec![]
                };
                for si in snap_indices {
                    if let Some(p) = self.canvas_positions.get_mut(&si) {
                        *p = snap_pos(*p);
                    }
                }
            }
            self.canvas_dragging = None;
            self.undo_pushed_for_current_drag = false;
            self.canvas_layout_dirty = true;
        }
        if let Some((idx, shift, cmd)) = canvas_click_modifier {
            if cmd {
                // Toggle selection; anchor only moves when adding to selection
                if self.multi_selected.contains(&idx) {
                    self.multi_selected.remove(&idx);
                    if self.selected == Some(idx) {
                        self.selected = self.multi_selected.iter().next().cloned();
                    }
                    // Do not move anchor on deselect — keep existing anchor for next Shift+click
                } else {
                    self.push_undo();
                    self.flush_edit();
                    self.multi_selected.insert(idx);
                    self.selected = Some(idx);
                    self.selected_child = None;
                    if let Some(step) = self.steps.get(idx) {
                        self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
                    }
                    self.canvas_selection_anchor = Some(idx);
                }
            } else if shift {
                // Use persistent anchor so range-select still works after background clear
                let anchor = self.canvas_selection_anchor.unwrap_or(idx);
                let (lo, hi) = if anchor <= idx {
                    (anchor, idx)
                } else {
                    (idx, anchor)
                };
                self.multi_selected = (lo..=hi).collect();
                self.selected = Some(idx);
                // anchor stays unchanged on Shift+click
            } else {
                self.select_step(idx);
                self.canvas_selection_anchor = Some(idx);
            }
        }
        if let Some(idx) = double_clicked_node {
            self.select_step(idx);
            self.view_mode = ViewMode::List;
            self.selected_child = None;
            self.scroll_to_selected = true;
        }
        if let Some(idx) = badge_toggle_idx {
            if self.expanded_steps.contains(&idx) {
                self.expanded_steps.remove(&idx);
            } else {
                self.expanded_steps.insert(idx);
            }
        }
        if let Some((idx, branch_name)) = panel_click_step {
            // Switch to List view and open the branch editor directly.
            // This gives a direct path from the canvas expand badge to the
            // branch property form, avoiding the manual "find step → open
            // accordion" detour.
            use crate::types::ViewMode;
            self.view_mode = ViewMode::List;
            self.select_step(idx);
            self.selected_child = Some((branch_name, 0));
            self.scroll_to_selected = true;
        }

        // Background right-click menu (shown when clicking empty canvas space, not on a node)
        resp.context_menu(|ui| {
            let s = crate::i18n::S::for_lang(&self.settings.lang);
            if ui.button(s.ctx_add_step_here).clicked() {
                // Convert screen position to canvas space
                if let Some(click_screen) = ui.input(|i| i.pointer.interact_pos()) {
                    let canvas_pos = egui::pos2(
                        (click_screen.x - origin.x) / z - self.canvas_pan.x,
                        (click_screen.y - origin.y) / z - self.canvas_pan.y,
                    );
                    self.canvas_pending_insert_pos = Some(canvas_pos);
                }
                self.add_menu_open = true;
                self.add_menu_just_opened = true;
                self.add_filter.clear();
                ui.close();
            }
            if ui.button(s.ctx_add_comment).clicked() {
                if let Some(click_screen) = ui.input(|i| i.pointer.interact_pos()) {
                    let canvas_pos = egui::pos2(
                        (click_screen.x - origin.x) / z - self.canvas_pan.x,
                        (click_screen.y - origin.y) / z - self.canvas_pan.y,
                    );
                    canvas_ctx_action = Some(CanvasContextAction::AddComment(canvas_pos));
                }
                ui.close();
            }
            ui.separator();
            if ui.button(s.ctx_select_all).clicked() {
                canvas_ctx_action = Some(CanvasContextAction::SelectAll);
                ui.close();
            }
            if !self.step_clipboard.is_empty() && ui.button(s.ctx_paste).clicked() {
                canvas_ctx_action = Some(CanvasContextAction::Paste);
                ui.close();
            }
            if self.multi_selected.len() >= 2 {
                ui.separator();
                let align_label = s.ctx_align_label.replace("{}", &self.multi_selected.len().to_string());
                ui.weak(&align_label);
                if ui.button(s.ctx_align_left).clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::AlignLeft);
                    ui.close();
                }
                if ui.button(s.ctx_align_top).clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::AlignTop);
                    ui.close();
                }
                if self.multi_selected.len() >= 3 {
                    if ui.button(s.ctx_distribute_h).clicked() {
                        canvas_ctx_action = Some(CanvasContextAction::DistributeH);
                        ui.close();
                    }
                    if ui.button(s.ctx_distribute_v).clicked() {
                        canvas_ctx_action = Some(CanvasContextAction::DistributeV);
                        ui.close();
                    }
                }
            }
            // ── Canvas view controls ──────────────────────────────────────
            {
                use egui_phosphor::regular as ph;
                ui.separator();
                if ui
                    .button(format!(
                        "{} {}",
                        ph::ARROW_COUNTER_CLOCKWISE,
                        s.canvas_reset
                    ))
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::CanvasReset);
                    ui.close();
                }
                if ui
                    .button(format!("{} {}", ph::ARROWS_OUT, s.canvas_fit))
                    .clicked()
                {
                    canvas_ctx_action = Some(CanvasContextAction::CanvasFit);
                    ui.close();
                }
                if ui
                    .selectable_label(self.settings.canvas_snap, s.canvas_snap)
                    .clicked()
                {
                    self.settings.canvas_snap = !self.settings.canvas_snap;
                    crate::settings::save_settings(&self.settings);
                    ui.close();
                }
                if ui
                    .selectable_label(self.settings.minimap_show, s.minimap_label)
                    .clicked()
                {
                    self.settings.minimap_show = !self.settings.minimap_show;
                    crate::settings::save_settings(&self.settings);
                    ui.close();
                }
                ui.label(format!("{:.0}%", self.canvas_zoom * 100.0));
            }
        });

        // Handle context menu actions from the node loop and background menu
        if let Some(act) = canvas_ctx_action {
            match act {
                CanvasContextAction::Delete(idx) => {
                    if self.multi_selected.len() > 1 && self.multi_selected.contains(&idx) {
                        let count = self.multi_selected.len();
                        self.confirm_dialog = Some(ConfirmAction::DeleteSteps(count));
                    } else if idx < self.steps.len() {
                        self.confirm_dialog = Some(ConfirmAction::DeleteStep(idx));
                    }
                }
                CanvasContextAction::Duplicate(idx) => {
                    if idx < self.steps.len() {
                        self.push_undo();
                        let step = self.steps[idx].clone();
                        let insert_at = idx + 1;
                        self.steps.insert(insert_at, step);
                        Self::canvas_shift_positions(&mut self.canvas_positions, insert_at, 1);
                        Self::form_edit_buffers_shift(&mut self.form_edit_buffers, insert_at, 1);
                        if let Some(orig_pos) = self.canvas_positions.get(&idx).copied() {
                            self.canvas_positions.insert(
                                insert_at,
                                egui::pos2(orig_pos.x + 30.0, orig_pos.y + 30.0),
                            );
                        }
                        self.dirty = true;
                    }
                }
                CanvasContextAction::OpenInList(idx) => {
                    self.select_step(idx);
                    self.view_mode = ViewMode::List;
                    self.scroll_to_selected = true;
                    self.selected_child = None;
                }
                CanvasContextAction::RunFrom(idx) => {
                    self.run_from_step(idx);
                }
                CanvasContextAction::CopySelected => self.copy_selected_steps(),
                CanvasContextAction::CutSelected => {
                    self.copy_selected_steps();
                    self.delete_selected_steps();
                }
                CanvasContextAction::ToggleEnabled(idx) => {
                    if idx < self.steps.len() {
                        self.push_undo();
                        Self::toggle_step_enabled(&mut self.steps[idx]);
                        self.dirty = true;
                    }
                }
                CanvasContextAction::Paste => self.paste_steps(),
                CanvasContextAction::AlignLeft => self.canvas_align_left(),
                CanvasContextAction::AlignTop => self.canvas_align_top(),
                CanvasContextAction::DistributeH => self.canvas_distribute_h(),
                CanvasContextAction::DistributeV => self.canvas_distribute_v(),
                CanvasContextAction::SelectAll => {
                    self.multi_selected = (0..self.steps.len()).collect();
                    self.selected = self.multi_selected.iter().min().cloned();
                }
                CanvasContextAction::AddComment(pos) => {
                    let id = self.canvas_comment_next_id;
                    self.canvas_comment_next_id += 1;
                    self.canvas_comments.push(CanvasComment {
                        id,
                        x: pos.x,
                        y: pos.y,
                        w: 180.0,
                        h: 80.0,
                        text: String::new(),
                        color: [255, 235, 100, 200],
                    });
                    self.canvas_editing_comment = Some(self.canvas_comments.len() - 1);
                    self.canvas_layout_dirty = true;
                }
                CanvasContextAction::CanvasReset => {
                    self.push_undo();
                    self.canvas_positions.clear();
                    self.ensure_canvas_layout();
                    self.canvas_layout_dirty = true;
                }
                CanvasContextAction::CanvasFit => {
                    self.canvas_fit_view(self.canvas_viewport_size);
                }
            }
        }

        // Empty state hint + add-step button
        if n == 0 {
            let s = crate::i18n::S::for_lang(&self.settings.lang);
            let center = resp.rect.center();
            if self.path.is_none() {
                painter.text(
                    center + egui::vec2(0.0, -14.0),
                    Align2::CENTER_CENTER,
                    s.empty_canvas_no_file,
                    FontId::proportional(13.0),
                    tokens::TEXT_MUTED,
                );
                painter.text(
                    center + egui::vec2(0.0, 10.0),
                    Align2::CENTER_CENTER,
                    s.hint_cmd_new_open,
                    FontId::proportional(11.0),
                    tokens::TEXT_MUTED_SECONDARY,
                );
            } else {
                // Clickable "add first step" button in the canvas center
                egui::Area::new(egui::Id::new("canvas_add_first_step"))
                    .fixed_pos(center + egui::vec2(-70.0, -18.0))
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        if ui
                            .add(
                                egui::Button::new(s.btn_add_first_step)
                                    .min_size(egui::vec2(140.0, 36.0)),
                            )
                            .on_hover_text(s.hint_add_step_shortcut)
                            .clicked()
                        {
                            self.add_menu_open = true;
                            self.add_menu_just_opened = true;
                        }
                    });
            }
            painter.text(
                center + egui::vec2(0.0, 34.0),
                Align2::CENTER_CENTER,
                "? キーでキーボードショートカット一覧",
                FontId::proportional(10.0),
                Color32::from_gray(50),
            );
        }

        // Persistent "+" button below the last node — always visible so users can
        // always add at the end without hunting for an edge midpoint.
        if n > 0 && !screen_positions.is_empty() {
            let last_top = screen_positions[n - 1];
            let btn_center = last_top + egui::vec2(NODE_W * z / 2.0, NODE_H * z + 14.0 * z);
            egui::Area::new(egui::Id::new("canvas_add_after_last"))
                .fixed_pos(btn_center - egui::vec2(14.0, 14.0))
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    if ui
                        .add(
                            egui::Button::new("+")
                                .min_size(egui::Vec2::splat(28.0))
                                .corner_radius(egui::CornerRadius::same(14)),
                        )
                        .on_hover_text("末尾にステップを追加 (Cmd+Shift+A)")
                        .clicked()
                    {
                        self.add_menu_open = true;
                        self.add_menu_just_opened = true;
                    }
                });
        }

        // Draw active lasso rectangle
        if let Some((lasso_start, lasso_end)) = self.canvas_lasso {
            let lasso_rect = egui::Rect::from_two_pos(lasso_start, lasso_end);
            painter.rect_filled(
                lasso_rect,
                2.0,
                Color32::from_rgba_premultiplied(80, 120, 220, 30),
            );
            painter.rect_stroke(
                lasso_rect,
                2.0,
                Stroke::new(1.0, Color32::from_rgb(100, 140, 255)),
                egui::StrokeKind::Inside,
            );
        }

        // Finalize lasso selection on drag release (guard against minimap drag sharing the frame)
        if resp.drag_stopped() && !self.minimap_dragging {
            if let Some((lasso_start, lasso_end)) = self.canvas_lasso.take() {
                let lasso_rect = egui::Rect::from_two_pos(lasso_start, lasso_end);
                if lasso_rect.width() > 10.0 || lasso_rect.height() > 10.0 {
                    if !self.canvas_lasso_additive {
                        self.multi_selected.clear();
                    }
                    for (i, &sp) in screen_positions.iter().enumerate() {
                        let nr = egui::Rect::from_min_size(sp, Vec2::new(NODE_W * z, NODE_H * z));
                        if lasso_rect.intersects(nr) {
                            self.multi_selected.insert(i);
                        }
                    }
                    if !self.multi_selected.is_empty() {
                        self.selected = self.multi_selected.iter().min().cloned();
                        self.canvas_selection_anchor = self.selected;
                    }
                }
                self.canvas_lasso_additive = false;
            }
        }

        // ── Canvas comment nodes (sticky notes) ──────────────────────────────
        {
            let mut delete_comment_idx: Option<usize> = None;
            let mut drag_comment: Option<(usize, egui::Pos2)> = None;
            let mut edit_comment: Option<usize> = None;
            let n_comments = self.canvas_comments.len();

            for ci in 0..n_comments {
                let c = &self.canvas_comments[ci];
                let screen_x = origin.x + (c.x + self.canvas_pan.x) * z;
                let screen_y = origin.y + (c.y + self.canvas_pan.y) * z;
                let screen_w = c.w * z;
                let screen_h = c.h * z;
                let comment_rect = egui::Rect::from_min_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(screen_w, screen_h),
                );
                if !resp.rect.intersects(comment_rect) {
                    continue;
                }

                let [cr, cg, cb, ca] = c.color;
                let fill = egui::Color32::from_rgba_premultiplied(
                    (cr as u16 * ca as u16 / 255) as u8,
                    (cg as u16 * ca as u16 / 255) as u8,
                    (cb as u16 * ca as u16 / 255) as u8,
                    ca,
                );
                painter.rect_filled(comment_rect, 3.0 * z, fill);
                painter.rect_stroke(
                    comment_rect,
                    3.0 * z,
                    egui::Stroke::new(1.0, tokens::COMMENT_BORDER),
                    egui::StrokeKind::Inside,
                );

                let is_editing = self.canvas_editing_comment == Some(ci);

                if is_editing && z >= 0.5 {
                    // Inline text editor — rendered via egui Area
                    let edit_pos = comment_rect.min + egui::vec2(4.0 * z, 4.0 * z);
                    let text_ref = &mut self.canvas_comments[ci].text;
                    let te = egui::TextEdit::multiline(text_ref)
                        .desired_width(screen_w - 8.0 * z)
                        .desired_rows(((screen_h - 8.0 * z) / 14.0).max(1.0) as usize)
                        .font(egui::FontId::proportional((11.0 * z).max(9.0)));
                    let te_resp = ui.put(
                        egui::Rect::from_min_size(
                            edit_pos,
                            egui::vec2(screen_w - 8.0 * z, screen_h - 8.0 * z),
                        ),
                        te,
                    );
                    // Esc: cancel editing (discard changes)
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.canvas_editing_comment = None;
                        self.canvas_layout_dirty = true;
                    }
                    // Lost focus or explicit finish: confirm editing
                    else if te_resp.lost_focus() {
                        self.canvas_editing_comment = None;
                        self.canvas_layout_dirty = true;
                    }
                } else if z >= 0.4 {
                    // Display text
                    let text_rect = comment_rect.shrink(4.0 * z);
                    painter.text(
                        text_rect.min,
                        egui::Align2::LEFT_TOP,
                        &c.text,
                        egui::FontId::proportional((11.0 * z).max(9.0)),
                        tokens::COMMENT_TEXT,
                    );
                }

                // 📝 label in top-right corner
                if z >= 0.5 {
                    painter.text(
                        comment_rect.right_top() + egui::vec2(-4.0 * z, 2.0 * z),
                        egui::Align2::RIGHT_TOP,
                        "📝",
                        egui::FontId::proportional(9.0 * z),
                        tokens::COMMENT_ICON_COLOR,
                    );
                }

                let comment_resp = ui.allocate_rect(comment_rect, egui::Sense::click_and_drag());

                // Cursor feedback for comments
                if self.canvas_comment_drag.map(|(i, _)| i) == Some(ci) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                } else if comment_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }

                if comment_resp.drag_started() {
                    let cursor = ui
                        .input(|i| i.pointer.interact_pos())
                        .unwrap_or(comment_rect.min);
                    let offset = cursor - comment_rect.min;
                    self.canvas_comment_drag = Some((ci, offset));
                    // Stop editing when drag starts
                    self.canvas_editing_comment = None;
                }
                if comment_resp.dragged() {
                    if let Some((drag_ci, offset)) = self.canvas_comment_drag {
                        if drag_ci == ci {
                            if let Some(cursor) = ui.input(|i| i.pointer.interact_pos()) {
                                let new_screen = cursor - offset;
                                drag_comment = Some((
                                    ci,
                                    egui::pos2(
                                        (new_screen.x - origin.x) / z - self.canvas_pan.x,
                                        (new_screen.y - origin.y) / z - self.canvas_pan.y,
                                    ),
                                ));
                            }
                        }
                    }
                }
                if comment_resp.drag_stopped() {
                    self.canvas_comment_drag = None;
                    self.canvas_layout_dirty = true;
                }
                if comment_resp.double_clicked() {
                    edit_comment = Some(ci);
                }
                comment_resp.context_menu(|ui| {
                    if ui.button("🗑 削除").clicked() {
                        delete_comment_idx = Some(ci);
                        ui.close();
                    }
                    ui.separator();
                    let colors: &[(&str, [u8; 4])] = &[
                        ("🟡 黄", [255, 235, 100, 200]),
                        ("🔵 青", [100, 160, 255, 200]),
                        ("🟢 緑", [100, 220, 130, 200]),
                        ("🔴 赤", [255, 120, 100, 200]),
                    ];
                    for (label, color) in colors {
                        if ui.button(*label).clicked() {
                            self.canvas_comments[ci].color = *color;
                            self.canvas_layout_dirty = true;
                            ui.close();
                        }
                    }
                });
            }

            // Apply deferred comment mutations
            if let Some((ci, new_pos)) = drag_comment {
                if ci < self.canvas_comments.len() {
                    self.canvas_comments[ci].x = new_pos.x;
                    self.canvas_comments[ci].y = new_pos.y;
                }
            }
            if let Some(ci) = edit_comment {
                self.canvas_editing_comment = Some(ci);
            }
            if let Some(ci) = delete_comment_idx {
                if ci < self.canvas_comments.len() {
                    self.canvas_comments.remove(ci);
                    if self.canvas_editing_comment == Some(ci) {
                        self.canvas_editing_comment = None;
                    } else if let Some(edit_idx) = self.canvas_editing_comment {
                        if edit_idx > ci {
                            self.canvas_editing_comment = Some(edit_idx - 1);
                        }
                    }
                    self.canvas_layout_dirty = true;
                }
            }
        }

        // Minimap (shown when there are 15+ steps; layout data precomputed above)
        if minimap_active {
            let mm_inner = mm_rect.shrink(MM_PAD);
            painter.rect_filled(
                mm_rect,
                4.0,
                Color32::from_rgba_premultiplied(18, 18, 22, 220),
            );
            painter.rect_stroke(
                mm_rect,
                4.0,
                Stroke::new(1.0, Color32::from_gray(55)),
                egui::StrokeKind::Inside,
            );
            // "MAP" label
            painter.text(
                mm_rect.left_top() + egui::vec2(4.0, 2.0),
                Align2::LEFT_TOP,
                "MAP",
                FontId::proportional(7.0),
                Color32::from_gray(70),
            );

            // Draw nodes as small rects (reuse precomputed to_mm)
            for i in 0..n {
                let p = self
                    .canvas_positions
                    .get(&i)
                    .copied()
                    .unwrap_or_else(|| default_canvas_pos(i, default_cols));
                let mm_min = to_mm(p);
                let mm_node = egui::Rect::from_min_size(
                    mm_min,
                    egui::vec2((NODE_W * mm_scale).max(2.0), (NODE_H * mm_scale).max(2.0)),
                );
                let is_sel = self.selected == Some(i) || self.multi_selected.contains(&i);
                let node_color = if self.current_run_step == Some(i) {
                    tokens::WARNING
                } else if self.canvas_error_steps.contains_key(&i) {
                    tokens::ERROR
                } else if is_sel {
                    Color32::from_rgb(100, 140, 255)
                } else {
                    // Reflect the same category color used on canvas, darkened for minimap scale
                    let cat = category_color(step_key_category(get_step_key(&self.steps[i])));
                    let [r, g, b, _] = cat.to_array();
                    Color32::from_rgb(
                        (r as u16 * 2 / 3) as u8,
                        (g as u16 * 2 / 3) as u8,
                        (b as u16 * 2 / 3) as u8,
                    )
                };
                painter.rect_filled(mm_node, 1.0, node_color);
            }

            // Draw viewport rectangle
            let vp_canvas_min = egui::pos2(
                (resp.rect.min.x - origin.x) / z - self.canvas_pan.x,
                (resp.rect.min.y - origin.y) / z - self.canvas_pan.y,
            );
            let vp_canvas_max = egui::pos2(
                (resp.rect.max.x - origin.x) / z - self.canvas_pan.x,
                (resp.rect.max.y - origin.y) / z - self.canvas_pan.y,
            );
            let mm_vp = egui::Rect::from_min_max(to_mm(vp_canvas_min), to_mm(vp_canvas_max));
            let mm_vp_clipped = mm_vp.intersect(mm_inner);
            if !mm_vp_clipped.is_negative() {
                painter.rect_filled(
                    mm_vp_clipped,
                    2.0,
                    Color32::from_rgba_premultiplied(255, 255, 255, 15),
                );
                painter.rect_stroke(
                    mm_vp_clipped,
                    2.0,
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(200, 210, 255, 160)),
                    egui::StrokeKind::Inside,
                );
            }

            // Latch minimap drag so pointer can exit the rect without losing input
            if resp.drag_started() {
                if let Some(ptr) = resp.interact_pointer_pos() {
                    if mm_rect.contains(ptr) {
                        self.minimap_dragging = true;
                    }
                }
            }
            if resp.drag_stopped() {
                self.minimap_dragging = false;
            }
            // Navigate: click inside rect OR latched drag (works even after pointer exits)
            if let Some(ptr) = resp.interact_pointer_pos() {
                let mm_click = resp.clicked() && mm_rect.contains(ptr);
                let mm_drag =
                    self.minimap_dragging && resp.dragged_by(egui::PointerButton::Primary);
                if mm_click || mm_drag {
                    let canvas_pt = from_mm(ptr);
                    self.canvas_pan = resp.rect.size() / 2.0 / z - canvas_pt.to_vec2();
                }
            }
        }

        // ── Canvas node search bar overlay ────────────────────────────────
        if self.canvas_search_open {
            let bar_w = 320.0_f32;
            let bar_pos = resp.rect.center_top() + egui::vec2(-bar_w / 2.0, 10.0);
            let mut search_text = std::mem::take(&mut self.canvas_search);
            let q = search_text.to_lowercase();
            // Collect all matching step indices
            let matches: Vec<usize> = if q.is_empty() {
                vec![]
            } else {
                self.steps
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| step_matches(s, &q))
                    .map(|(i, _)| i)
                    .collect()
            };
            let matched_count = matches.len();
            let total = self.steps.len();
            // Show "cur/total" if the selected node is in the match list
            let count_label = if q.is_empty() {
                format!("{total}")
            } else if matched_count == 0 {
                "一致なし".to_string()
            } else {
                let cur_pos = self
                    .selected
                    .and_then(|sel| matches.iter().position(|&i| i == sel));
                match cur_pos {
                    Some(p) => format!("{}/{matched_count}", p + 1),
                    None => format!("{matched_count}/{total}"),
                }
            };
            egui::Area::new(egui::Id::new("canvas_search_bar"))
                .fixed_pos(bar_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(egui::Margin::symmetric(8, 5))
                        .show(ui, |ui| {
                            ui.set_min_width(bar_w - 16.0);
                            ui.horizontal(|ui| {
                                let te = ui.add(
                                    egui::TextEdit::singleline(&mut search_text)
                                        .hint_text(
                                            "🔍 ノード検索... (Enter: 次  ⇧Enter: 前  Esc: 閉じる)",
                                        )
                                        .desired_width(bar_w - 90.0),
                                );
                                if !te.has_focus() {
                                    te.request_focus();
                                }
                                let count_color = if !q.is_empty() && matched_count == 0 {
                                    egui::Color32::from_rgb(220, 80, 80)
                                } else {
                                    egui::Color32::from_gray(150)
                                };
                                ui.label(
                                    egui::RichText::new(&count_label).color(count_color).small(),
                                );
                                // Clear button
                                if !search_text.is_empty()
                                    && ui
                                        .small_button(
                                            egui::RichText::new("×")
                                                .color(egui::Color32::from_gray(160)),
                                        )
                                        .on_hover_text("クリア")
                                        .clicked()
                                {
                                    search_text.clear();
                                }
                            });
                        });
                });
            // Enter → next match after selected; Shift+Enter → previous match
            let shift_enter =
                ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter));
            let plain_enter =
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            if !search_text.is_empty() && !matches.is_empty() {
                let jump_to = if plain_enter {
                    let cur = self.selected.unwrap_or(0);
                    let pos = matches.iter().position(|&i| i > cur).unwrap_or(0);
                    Some(matches[pos])
                } else if shift_enter {
                    let cur = self.selected.unwrap_or(0);
                    let pos = matches
                        .iter()
                        .rposition(|&i| i < cur)
                        .unwrap_or(matches.len() - 1);
                    Some(matches[pos])
                } else {
                    None
                };
                if let Some(match_idx) = jump_to {
                    let cols = default_canvas_cols(self.steps.len());
                    let pos = self
                        .canvas_positions
                        .get(&match_idx)
                        .copied()
                        .unwrap_or_else(|| default_canvas_pos(match_idx, cols));
                    let z = self.canvas_zoom;
                    let vp = resp.rect.size();
                    self.canvas_pan = egui::vec2(
                        vp.x / 2.0 / z - pos.x - NODE_W / 2.0,
                        vp.y / 2.0 / z - pos.y - NODE_H / 2.0,
                    );
                    self.select_step(match_idx);
                }
            }
            self.canvas_search = search_text;
        }

        // Canvas keyboard shortcut help overlay (toggled by `?`)
        if self.canvas_help_open {
            egui::Window::new("キャンバス ショートカット")
                .id(egui::Id::new("canvas_help_window"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("canvas_help_overlay_grid")
                        .num_columns(2)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            let rows: &[(&str, &str)] = &[
                                ("Ctrl+スクロール / ピンチ", "ズーム"),
                                ("Cmd+0", "全体表示 (fit to view)"),
                                ("Cmd+1", "100% ズーム + 中央"),
                                ("背景ドラッグ / 中ボタンドラッグ", "パン"),
                                ("Shift+背景ドラッグ", "ラッソ選択"),
                                ("クリック", "選択"),
                                ("Cmd+クリック", "選択をトグル"),
                                ("Shift+クリック", "範囲選択"),
                                ("Cmd+A", "全選択"),
                                ("↑ / ↓", "前/次ノードへ移動"),
                                ("Cmd+↑ / Cmd+↓", "ステップを前後に移動"),
                                ("Delete / Backspace", "選択ステップを削除"),
                                ("Cmd+C / X / V / D", "コピー/カット/貼付/複製"),
                                ("Cmd+Z / Shift+Z", "アンドゥ / リドゥ"),
                                ("Cmd+G", "ノード検索"),
                                ("Cmd+Shift+A", "ステップ追加メニュー"),
                                ("ダブルクリック", "List ビューで開く"),
                                ("右クリック", "コンテキストメニュー"),
                                ("ドラッグハンドル", "ステップを並び替え"),
                                ("?", "このヘルプを閉じる"),
                            ];
                            for (key, desc) in rows {
                                ui.strong(*key);
                                ui.label(*desc);
                                ui.end_row();
                            }
                        });
                    if ui.button("閉じる").clicked() {
                        self.canvas_help_open = false;
                    }
                });
        }

        self.canvas_viewport_size = resp.rect.size();
    }
}
