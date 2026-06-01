// ---- flowchart view ---------------------------------------------------------

use eframe::egui;

use crate::i18n::S;
use crate::state::EditorApp;

impl EditorApp {
    pub(crate) fn show_flowchart(&mut self, ui: &mut egui::Ui) {
        const NODE_W: f32 = 300.0;
        const NODE_H: f32 = 42.0;
        const HEAD_H: f32 = 22.0;
        const INDENT: f32 = 24.0;
        const GAP_Y: f32 = 7.0;
        const PAD_X: f32 = 20.0;
        const PAD_TOP: f32 = 16.0;
        const ICON_W: f32 = 22.0;

        let nodes = self.build_flow_nodes();
        let selected = self.selected;

        if nodes.is_empty() {
            let s = S::for_lang(&self.settings.lang);
            ui.centered_and_justified(|ui| {
                ui.label(s.empty_no_steps);
            });
            return;
        }

        let z = self.flow_zoom;
        let nw = NODE_W * z;
        let nh = NODE_H * z;
        let hh = HEAD_H * z;
        let ind = INDENT * z;
        let gap = GAP_Y * z;

        // Pre-compute (x, y, height) for each node
        let mut y_cur = 0.0_f32;
        let positions: Vec<(f32, f32, f32)> = nodes
            .iter()
            .map(|n| {
                let x = n.depth as f32 * ind;
                let h = if n.is_header { hh } else { nh };
                let pos = (x, y_cur, h);
                y_cur += h + gap;
                pos
            })
            .collect();

        let mut toggle_expand: Option<usize> = None;
        let mut click_select: Option<usize> = None;

        let (resp, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        // Zoom on scroll wheel with Ctrl held; plain scroll pans vertically
        if resp.hovered() {
            let (scroll_y, ctrl) = ui.input(|i| (i.smooth_scroll_delta.y, i.modifiers.command));
            if scroll_y != 0.0 {
                if ctrl {
                    let factor = 1.0 + scroll_y * 0.001;
                    self.flow_zoom = (self.flow_zoom * factor).clamp(0.25, 2.0);
                } else {
                    self.flow_pan.y += scroll_y;
                }
            }
        }

        // Pan on primary-mouse drag or middle-mouse drag
        if resp.dragged_by(egui::PointerButton::Primary)
            || resp.dragged_by(egui::PointerButton::Middle)
        {
            self.flow_pan += resp.drag_delta();
        }

        // Auto-scroll to selected node when switching to flowchart view.
        if self.scroll_to_selected {
            self.scroll_to_selected = false;
            if let Some(sel_idx) = selected {
                let mut y = PAD_TOP * z;
                for (node, &(_nx, _ny, h)) in nodes.iter().zip(positions.iter()) {
                    if node.step_idx == sel_idx && !node.is_header {
                        break;
                    }
                    y += h + gap;
                }
                let canvas_h = resp.rect.height();
                self.flow_pan.y = canvas_h / 2.0 - y - (NODE_H * z / 2.0);
            }
        }

        // Clamp pan so at least 40px of content stays on-screen in each direction.
        let margin = 40.0;
        self.flow_pan.y = self.flow_pan.y.clamp(
            -(y_cur - margin).max(0.0),
            (resp.rect.height() - margin).max(0.0),
        );
        self.flow_pan.x = self.flow_pan.x.clamp(
            -(NODE_W * z - margin).max(0.0),
            (resp.rect.width() - margin).max(0.0),
        );

        let origin = resp.rect.min + egui::vec2(PAD_X * z, PAD_TOP * z) + self.flow_pan;
        let click_pos = if resp.clicked() {
            resp.interact_pointer_pos()
        } else {
            None
        };

        for (ni, (node, &(nx, ny, node_h))) in nodes.iter().zip(positions.iter()).enumerate() {
            let node_rect =
                egui::Rect::from_min_size(origin + egui::vec2(nx, ny), egui::vec2(nw, node_h));

            // ── connector from previous node ──────────────────────────
            if ni > 0 {
                let &(prev_x, prev_y, prev_h) = &positions[ni - 1];
                let dot = 14.0 * z;
                let from = origin + egui::vec2(prev_x + dot, prev_y + prev_h);
                let to = origin + egui::vec2(nx + dot, ny);
                let stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(65));

                if (from.x - to.x).abs() < 0.5 {
                    painter.line_segment([from, to], stroke);
                } else {
                    let mid_y = from.y + gap / 2.0;
                    painter.line_segment([from, egui::pos2(from.x, mid_y)], stroke);
                    painter
                        .line_segment([egui::pos2(from.x, mid_y), egui::pos2(to.x, mid_y)], stroke);
                    painter.line_segment([egui::pos2(to.x, mid_y), to], stroke);
                }

                if !node.is_header {
                    painter.arrow(
                        egui::pos2(to.x, to.y - 5.0 * z),
                        egui::vec2(0.0, 5.0 * z),
                        egui::Stroke::new(1.5, egui::Color32::from_gray(65)),
                    );
                }
            }

            // ── branch header (no box) ─────────────────────────────────
            if node.is_header {
                painter.text(
                    node_rect.left_center() + egui::vec2(8.0 * z, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &node.label,
                    egui::FontId::proportional(11.5 * z),
                    egui::Color32::from_gray(128),
                );
                continue;
            }

            // ── node box ───────────────────────────────────────────────
            let is_selected = selected == Some(node.step_idx);
            let bg = if is_selected {
                egui::Color32::from_rgb(28, 52, 88)
            } else {
                egui::Color32::from_gray(40)
            };
            painter.rect_filled(node_rect, 5.0 * z, bg);
            painter.rect_stroke(
                node_rect,
                5.0 * z,
                egui::Stroke::new(
                    1.0,
                    if is_selected {
                        egui::Color32::from_rgb(70, 125, 200)
                    } else {
                        egui::Color32::from_gray(58)
                    },
                ),
                egui::StrokeKind::Inside,
            );

            // live run highlight (gold outer ring)
            if self.current_run_step == Some(node.step_idx) {
                painter.rect_stroke(
                    node_rect.expand(1.5 * z),
                    5.0 * z,
                    egui::Stroke::new(2.5 * z, egui::Color32::from_rgb(249, 226, 175)),
                    egui::StrokeKind::Outside,
                );
            }

            // left color stripe
            painter.rect_filled(
                egui::Rect::from_min_size(node_rect.min, egui::vec2(4.0 * z, nh)),
                0.0,
                node.color,
            );

            // label
            painter.text(
                node_rect.left_center() + egui::vec2(12.0 * z, 0.0),
                egui::Align2::LEFT_CENTER,
                &node.label,
                egui::FontId::proportional(13.0 * z),
                egui::Color32::LIGHT_GRAY,
            );

            // ── expand / collapse icon ─────────────────────────────────
            if let Some(expand_key) = node.expand_key {
                let icon_pos =
                    node_rect.right_center() + egui::vec2(-(ICON_W * z / 2.0 + 5.0 * z), 0.0);
                painter.text(
                    icon_pos,
                    egui::Align2::CENTER_CENTER,
                    if node.is_expanded { "▼" } else { "▶" },
                    egui::FontId::proportional(11.0 * z),
                    egui::Color32::from_gray(170),
                );

                if let Some(cp) = click_pos {
                    if egui::Rect::from_center_size(icon_pos, egui::vec2(ICON_W * z, ICON_W * z))
                        .contains(cp)
                    {
                        toggle_expand = Some(expand_key);
                    }
                }
            }

            // ── click to select ────────────────────────────────────────
            if let Some(cp) = click_pos {
                if node_rect.contains(cp) && toggle_expand.is_none() {
                    click_select = Some(node.step_idx);
                }
            }
        }

        if let Some(key) = toggle_expand {
            if self.expanded_steps.contains(&key) {
                self.expanded_steps.remove(&key);
            } else {
                self.expanded_steps.insert(key);
            }
        } else if let Some(idx) = click_select {
            self.select_step(idx);
        }
    }
}
