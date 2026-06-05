// ---- eframe::App impl -------------------------------------------------------

use std::collections::{HashMap, HashSet};

use eframe::egui;

use crate::ai_integration::extract_yaml_blocks;
use crate::flow_helpers::{
    count_child_steps, default_canvas_cols, default_canvas_pos, get_step_key, step_display_name,
    step_summary, NODE_H, NODE_W,
};
use crate::i18n::S;
use crate::settings::save_settings;
use crate::state::EditorApp;
use crate::step_templates::{step_icon, StepTemplate, STEP_TEMPLATES};
use crate::types::{
    category_color, step_key_category, AiMessage, BottomTab, ConfirmAction, DragPayload, LogEntry,
    LogLevel, PropView, StepAction, ViewMode,
};

impl eframe::App for EditorApp {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Panels are created via ctx below — no-op here.
    }

    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Re-apply theme every frame so our choice overrides any system dark-mode
        // detection that eframe/winit may trigger after startup.
        crate::apply_style(ctx, &self.settings.theme);

        let s = S::for_lang(&self.settings.lang);
        if self.bottom_tab != BottomTab::Variables {
            self.var_highlight = None;
        }
        // ── OS window close button ────────────────────────────────────────────
        if ctx.input(|i| i.viewport().close_requested())
            && self.dirty
            && self.confirm_dialog.is_none()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_dialog = Some(ConfirmAction::Quit);
        }

        // ── Keyboard shortcuts ────────────────────────────────────────────────
        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Z,
            )
        }) {
            self.redo();
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
            self.undo();
        }
        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            )
        }) {
            self.save_file_as();
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save_file();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            if self.run_child.is_some() {
                self.stop_run();
            } else {
                self.run_scenario();
            }
        }
        if self.run_child.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::R))
        {
            self.run_scenario();
        }
        if self.run_child.is_none()
            && ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::F5,
                )
            })
        {
            self.run_selection();
        }
        // Canvas-mode Delete/Backspace: route through confirm for both single and bulk
        if self.view_mode == ViewMode::Canvas
            && !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Delete)
                    || i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace)
            })
        {
            if self.multi_selected.len() > 1 {
                let count = self.multi_selected.len();
                self.confirm_dialog = Some(ConfirmAction::DeleteSteps(count));
            } else if let Some(idx) = self.selected {
                if idx < self.steps.len() {
                    self.confirm_dialog = Some(ConfirmAction::DeleteStep(idx));
                }
            }
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && self.prop_view != PropView::Yaml
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete))
        {
            if self.multi_selected.len() > 1 {
                let count = self.multi_selected.len();
                self.confirm_dialog = Some(ConfirmAction::DeleteSteps(count));
            } else if let Some(idx) = self.selected {
                if idx < self.steps.len() {
                    self.confirm_dialog = Some(ConfirmAction::DeleteStep(idx));
                }
            }
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::C))
        {
            self.copy_selected_steps();
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::X))
        {
            self.copy_selected_steps();
            self.delete_selected_steps();
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::V))
        {
            self.paste_steps();
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::D))
        {
            self.copy_selected_steps();
            self.paste_steps();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.add_menu_open = false;
            if self.view_mode == ViewMode::Canvas {
                if self.canvas_help_open {
                    self.canvas_help_open = false;
                } else if self.canvas_search_open {
                    self.canvas_search_open = false;
                    self.canvas_search.clear();
                } else {
                    self.selected = None;
                    self.multi_selected.clear();
                    self.canvas_lasso = None;
                    self.canvas_edge_drag = None;
                }
            }
        }
        if self.view_mode == ViewMode::Canvas
            && !self.add_menu_open
            && !self.steps.is_empty()
            && !self.canvas_search_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::A))
        {
            self.multi_selected = (0..self.steps.len()).collect();
            self.selected = Some(0);
            self.canvas_selection_anchor = Some(0);
        }
        if self.view_mode == ViewMode::Canvas
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::G))
        {
            if self.canvas_search_open {
                self.canvas_search_open = false;
                self.canvas_search.clear();
            } else {
                self.canvas_search_open = true;
            }
        }
        // `?` key toggles the canvas help overlay
        if self.view_mode == ViewMode::Canvas
            && !self.canvas_search_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Questionmark))
        {
            self.canvas_help_open = !self.canvas_help_open;
        }
        if self.view_mode == ViewMode::Canvas
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num0))
        {
            self.canvas_fit_view(self.canvas_viewport_size);
        }
        if self.view_mode == ViewMode::Canvas
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num1))
        {
            self.canvas_zoom = 1.0;
            let n = self.steps.len();
            if n > 0 {
                let positions = &self.canvas_positions;
                let default_cols = default_canvas_cols(n);
                let pos_of = |i: usize| {
                    positions
                        .get(&i)
                        .copied()
                        .unwrap_or_else(|| default_canvas_pos(i, default_cols))
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
                let vp = self.canvas_viewport_size;
                const MM_W_0: f32 = 160.0;
                const MM_MARGIN_0: f32 = 8.0;
                let vis_w = if n >= 15 {
                    (vp.x - MM_W_0 - MM_MARGIN_0 * 2.0).max(vp.x * 0.5)
                } else {
                    vp.x
                };
                self.canvas_pan = egui::vec2(
                    vis_w / 2.0 - (min_x + max_x) / 2.0,
                    vp.y / 2.0 - (min_y + max_y) / 2.0,
                );
            }
        }
        // Cmd+F: open canvas search (standard convention); step add is Cmd+Shift+A only
        if self.view_mode == ViewMode::Canvas
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F))
        {
            if self.canvas_search_open {
                self.canvas_search_open = false;
                self.canvas_search.clear();
            } else {
                self.canvas_search_open = true;
            }
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::ArrowUp))
        {
            if self.multi_selected.len() > 1 {
                let mut sorted: Vec<usize> = self.multi_selected.iter().cloned().collect();
                sorted.sort_unstable();
                let min_sel = sorted[0];
                let max_sel = *sorted.last().unwrap();
                if min_sel > 0 {
                    self.push_undo();
                    // Move the step just above the block to below the block
                    let above_pos = self.canvas_positions.remove(&(min_sel - 1));
                    let step = self.steps.remove(min_sel - 1);
                    let insert_at = max_sel; // block shifts to [min-1..max-1], insert after = max
                    Self::canvas_shift_positions(&mut self.canvas_positions, min_sel - 1, -1);
                    Self::canvas_shift_positions(&mut self.canvas_positions, insert_at, 1);
                    self.steps.insert(insert_at, step);
                    if let Some(p) = above_pos {
                        self.canvas_positions.insert(insert_at, p);
                    }
                    self.multi_selected = self.multi_selected.iter().map(|&i| i - 1).collect();
                    self.selected = self.selected.map(|i| i.saturating_sub(1));
                    self.dirty = true;
                }
            } else if let Some(i) = self.selected {
                if i > 0 {
                    self.push_undo();
                    self.steps.swap(i, i - 1);
                    let pos_a = self.canvas_positions.remove(&i);
                    let pos_b = self.canvas_positions.remove(&(i - 1));
                    if let Some(p) = pos_a {
                        self.canvas_positions.insert(i - 1, p);
                    }
                    if let Some(p) = pos_b {
                        self.canvas_positions.insert(i, p);
                    }
                    self.multi_selected = self
                        .multi_selected
                        .iter()
                        .map(|&x| {
                            if x == i {
                                i - 1
                            } else if x == i - 1 {
                                i
                            } else {
                                x
                            }
                        })
                        .collect();
                    self.selected = Some(i - 1);
                    self.form_edit_buffers.retain(|k, _| {
                        let first_at = k.find('@').unwrap_or(k.len());
                        let rem = &k[first_at.saturating_add(1)..];
                        let end = rem.find('@').unwrap_or(rem.len());
                        rem[..end]
                            .parse::<usize>()
                            .map_or(true, |n| n != i && n != i - 1)
                    });
                    self.dirty = true;
                }
            }
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::ArrowDown))
        {
            if self.multi_selected.len() > 1 {
                let mut sorted: Vec<usize> = self.multi_selected.iter().cloned().collect();
                sorted.sort_unstable();
                let min_sel = sorted[0];
                let max_sel = *sorted.last().unwrap();
                if max_sel + 1 < self.steps.len() {
                    self.push_undo();
                    // Move the step just below the block to above the block
                    let below_pos = self.canvas_positions.remove(&(max_sel + 1));
                    let step = self.steps.remove(max_sel + 1);
                    let insert_at = min_sel; // block stays, insert before it
                    Self::canvas_shift_positions(&mut self.canvas_positions, max_sel + 1, -1);
                    Self::canvas_shift_positions(&mut self.canvas_positions, insert_at, 1);
                    self.steps.insert(insert_at, step);
                    if let Some(p) = below_pos {
                        self.canvas_positions.insert(insert_at, p);
                    }
                    self.multi_selected = self.multi_selected.iter().map(|&i| i + 1).collect();
                    self.selected = self.selected.map(|i| (i + 1).min(self.steps.len() - 1));
                    self.dirty = true;
                }
            } else if let Some(i) = self.selected {
                if i + 1 < self.steps.len() {
                    self.push_undo();
                    self.steps.swap(i, i + 1);
                    let pos_a = self.canvas_positions.remove(&i);
                    let pos_b = self.canvas_positions.remove(&(i + 1));
                    if let Some(p) = pos_a {
                        self.canvas_positions.insert(i + 1, p);
                    }
                    if let Some(p) = pos_b {
                        self.canvas_positions.insert(i, p);
                    }
                    self.multi_selected = self
                        .multi_selected
                        .iter()
                        .map(|&x| {
                            if x == i {
                                i + 1
                            } else if x == i + 1 {
                                i
                            } else {
                                x
                            }
                        })
                        .collect();
                    self.selected = Some(i + 1);
                    self.form_edit_buffers.retain(|k, _| {
                        let first_at = k.find('@').unwrap_or(k.len());
                        let rem = &k[first_at.saturating_add(1)..];
                        let end = rem.find('@').unwrap_or(rem.len());
                        rem[..end]
                            .parse::<usize>()
                            .map_or(true, |n| n != i && n != i + 1)
                    });
                    self.dirty = true;
                }
            }
        }
        // Bare ↑/↓ in Canvas view: move selection to prev/next node without reordering
        if self.view_mode == ViewMode::Canvas
            && !self.canvas_search_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp))
        {
            if let Some(sel) = self.selected {
                if sel > 0 {
                    let new_sel = sel - 1;
                    self.select_step(new_sel);
                    // Auto-scroll viewport to keep node visible
                    let cols = default_canvas_cols(self.steps.len());
                    let pos = self
                        .canvas_positions
                        .get(&new_sel)
                        .copied()
                        .unwrap_or_else(|| default_canvas_pos(new_sel, cols));
                    let z = self.canvas_zoom;
                    let vp = self.canvas_viewport_size;
                    self.canvas_pan = egui::vec2(
                        vp.x / 2.0 / z - pos.x - NODE_W / 2.0,
                        vp.y / 2.0 / z - pos.y - NODE_H / 2.0,
                    );
                }
            }
        }
        if self.view_mode == ViewMode::Canvas
            && !self.canvas_search_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown))
        {
            let n = self.steps.len();
            if let Some(sel) = self.selected {
                if sel + 1 < n {
                    let new_sel = sel + 1;
                    self.select_step(new_sel);
                    let cols = default_canvas_cols(n);
                    let pos = self
                        .canvas_positions
                        .get(&new_sel)
                        .copied()
                        .unwrap_or_else(|| default_canvas_pos(new_sel, cols));
                    let z = self.canvas_zoom;
                    let vp = self.canvas_viewport_size;
                    self.canvas_pan = egui::vec2(
                        vp.x / 2.0 / z - pos.x - NODE_W / 2.0,
                        vp.y / 2.0 / z - pos.y - NODE_H / 2.0,
                    );
                }
            }
        }
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::A,
                )
            })
        {
            self.add_menu_open = true;
            self.add_menu_just_opened = true;
            self.add_filter.clear();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N)) {
            if !self.dirty {
                self.name = "new_scenario".into();
                self.steps.clear();
                self.scenario_vars.clear();
                self.selected = None;
                self.path = None;
                self.dirty = false;
            } else {
                self.confirm_dialog = Some(ConfirmAction::NewFile);
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
            self.open_file();
        }
        // Cmd+, = Settings (standard macOS/app convention)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma)) {
            self.settings_open = true;
        }

        // ── Confirm dialog (modal) ────────────────────────────────────────────
        if let Some(action) = self.confirm_dialog.clone() {
            let delete_msg_buf;
            let desc = match &action {
                ConfirmAction::OpenFile => s.confirm_open_file,
                ConfirmAction::NewFile => s.confirm_new_file,
                ConfirmAction::DeleteStep(idx) => {
                    let child_count = self.steps.get(*idx).map(count_child_steps).unwrap_or(0);
                    if child_count > 0 {
                        delete_msg_buf = s.confirm_delete_step_with_children.replacen(
                            "{}",
                            &child_count.to_string(),
                            1,
                        );
                        &delete_msg_buf
                    } else {
                        s.confirm_delete_step
                    }
                }
                ConfirmAction::DeleteSteps(count) => {
                    delete_msg_buf = s.confirm_delete_steps.replacen("{}", &count.to_string(), 1);
                    &delete_msg_buf
                }
                ConfirmAction::Quit => s.confirm_quit,
            };
            let mut yes = false;
            let mut no = false;
            // Enter and Escape both cancel — Undo exists so accidental deletion risk is high.
            // The user must explicitly click the confirm button to proceed.
            egui::Modal::new(egui::Id::new("confirm_modal")).show(ctx, |ui| {
                ui.set_min_width(240.0);
                ui.strong(s.confirm_title);
                ui.add_space(4.0);
                ui.label(desc);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(s.btn_yes).clicked() {
                        yes = true;
                    }
                    let cancel_btn = ui.button(s.btn_cancel);
                    if cancel_btn.clicked() {
                        no = true;
                    }
                    // Auto-focus cancel button so Enter/Space triggers cancel, not confirm
                    if !yes {
                        cancel_btn.request_focus();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.weak(s.esc_cancel);
                    });
                });
            });
            if yes {
                self.confirm_dialog = None;
                match action {
                    ConfirmAction::OpenFile => self.do_open_file(),
                    ConfirmAction::NewFile => {
                        self.name = "new_scenario".into();
                        self.steps.clear();
                        self.scenario_vars.clear();
                        self.selected = None;
                        self.path = None;
                        self.dirty = false;
                    }
                    ConfirmAction::DeleteStep(idx) => {
                        if idx < self.steps.len() {
                            self.push_undo();
                            self.steps.remove(idx);
                            Self::canvas_shift_positions(&mut self.canvas_positions, idx, -1);
                            self.selected = None;
                            self.multi_selected.clear();
                            self.edit_buf.clear();
                            self.parse_error = None;
                            let suffix = format!("@{idx}");
                            self.form_edit_buffers.retain(|k, _| !k.ends_with(&suffix));
                            Self::form_edit_buffers_shift(&mut self.form_edit_buffers, idx, -1);
                            self.dirty = true;
                        }
                    }
                    ConfirmAction::DeleteSteps(_) => {
                        self.delete_selected_steps();
                    }
                    ConfirmAction::Quit => {
                        // Clear dirty flag before closing so the close_requested()
                        // handler on the next frame does not re-open the dialog.
                        self.dirty = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            } else if no || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.confirm_dialog = None;
            }
        }

        // ── Window title (dirty indicator) ───────────────────────────────────
        let title = if self.dirty {
            format!("RPA シナリオエディター ─ {}*", self.name)
        } else {
            format!("RPA シナリオエディター ─ {}", self.name)
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        // ── Live run progress polling ─────────────────────────────────────────
        let child_exited_status: Option<std::process::ExitStatus> =
            if let Some(ref mut child) = self.run_child {
                child.try_wait().ok().flatten()
            } else {
                None
            };
        if let Some(status) = child_exited_status {
            self.run_child = None;
            self.run_progress_file = None;
            let last_run_step = self.current_run_step;
            let offset = self.run_from_offset; // BUG-1: rpa indices are slice-relative
            self.current_run_step = None;
            if status.success() {
                // Mark all steps from offset..end as completed
                for i in offset..self.steps.len() {
                    self.canvas_completed_steps.insert(i);
                }
                self.log_ok("実行が完了しました");
            } else {
                let code = status.code().unwrap_or(-1);
                if let Some(slice_step) = last_run_step {
                    let orig_step = offset + slice_step;
                    self.canvas_error_steps
                        .insert(orig_step, format!("終了コード: {code}"));
                    for i in offset..orig_step {
                        self.canvas_completed_steps.insert(i);
                    }
                }
                self.log_err(format!("実行に失敗しました (終了コード: {code}) — ログパネルでシナリオのエラーを確認してください"));
            }
        }
        if self.last_progress_check.elapsed() > std::time::Duration::from_millis(100) {
            if let Some(ref f) = self.run_progress_file {
                if let Ok(s) = std::fs::read_to_string(f) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                        // BUG-1: rpa emits 0-based step index relative to the slice;
                        // add offset to get original canvas index.
                        let offset = self.run_from_offset;
                        self.current_run_step = v["step"].as_u64().map(|n| offset + n as usize);
                        if let Some(curr) = self.current_run_step {
                            for i in offset..curr {
                                self.canvas_completed_steps.insert(i);
                            }
                        }
                    }
                }
            }
            self.last_progress_check = std::time::Instant::now();
        }
        if self.run_child.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        // Drain rpa stderr lines into the log panel in real time (BUG-3: request
        // a repaint if any lines arrived so the UI updates even on exit frame).
        let stderr_lines: Vec<String> = self
            .stderr_rx
            .as_ref()
            .map(|rx| std::iter::from_fn(|| rx.try_recv().ok()).collect())
            .unwrap_or_default();
        if !stderr_lines.is_empty() {
            for line in stderr_lines {
                // SEC-4: redact lines that likely contain secret values
                let redacted = redact_secret_line(&line);
                self.log_err(format!("rpa: {redacted}"));
            }
            ctx.request_repaint();
        }
        if self.run_child.is_none() {
            self.stderr_rx = None;
        }

        // ── AI response polling ───────────────────────────────────────────────
        if let Some(ref rx) = self.ai_rx {
            if let Ok(response) = rx.try_recv() {
                let (yaml_blocks, unclosed) = extract_yaml_blocks(&response);
                if unclosed {
                    self.log.push(LogEntry {
                        message: "⚠ AI応答に閉じていない```yamlブロックがあります。YAMLを確認してください。".into(),
                        level: LogLevel::Error,
                    });
                }
                self.ai_messages.push(AiMessage {
                    role: "assistant".into(),
                    content: response,
                    yaml_blocks,
                });
                self.ai_loading = false;
                self.ai_rx = None;
                if !self.ai_panel_open {
                    self.ai_unread = true;
                }
                ctx.request_repaint();
            }
        }

        // ── ai_create step generation polling ────────────────────────────────
        let ai_step_done = if let Some((idx, ref rx)) = self.ai_step_rx {
            if let Ok(result) = rx.try_recv() {
                ctx.request_repaint();
                Some((idx, result))
            } else {
                ctx.request_repaint();
                None
            }
        } else {
            None
        };
        if let Some((idx, result)) = ai_step_done {
            self.ai_step_rx = None;
            match result {
                Err(e) => {
                    self.ai_step_error = Some((idx, e.to_string()));
                }
                Ok(text) => {
                    let (blocks, _) = extract_yaml_blocks(&text);
                    let yaml_src = if blocks.is_empty() {
                        text.clone()
                    } else {
                        blocks.join("\n---\n")
                    };
                    match serde_yml::from_str::<serde_yml::Value>(&yaml_src) {
                        Err(e) => {
                            self.ai_step_error = Some((idx, format!("YAML解析失敗: {e}")));
                        }
                        Ok(val) => {
                            let new_steps: Vec<serde_yml::Value> = match val {
                                serde_yml::Value::Sequence(seq) => seq,
                                other => vec![other],
                            };
                            if new_steps.is_empty() {
                                self.ai_step_error =
                                    Some((idx, "AIから有効なステップが返りませんでした。".into()));
                            } else {
                                // Verify the ai_create step is still at the original idx.
                                // The user may have inserted/deleted steps while the request
                                // was in-flight, which would shift or remove the slot.
                                let still_valid = idx < self.steps.len()
                                    && get_step_key(&self.steps[idx]) == "ai_create";
                                if !still_valid {
                                    self.ai_step_error = Some((
                                        idx,
                                        "生成完了前にステップが変更されました。再度お試しください。".into(),
                                    ));
                                } else {
                                    let count = new_steps.len();
                                    self.push_undo();
                                    self.steps.remove(idx);
                                    Self::canvas_shift_positions(
                                        &mut self.canvas_positions,
                                        idx,
                                        -1,
                                    );
                                    Self::form_edit_buffers_shift(
                                        &mut self.form_edit_buffers,
                                        idx,
                                        -1,
                                    );
                                    for (j, s) in new_steps.into_iter().enumerate() {
                                        self.steps.insert(idx + j, s);
                                        Self::canvas_shift_positions(
                                            &mut self.canvas_positions,
                                            idx + j,
                                            1,
                                        );
                                        Self::form_edit_buffers_shift(
                                            &mut self.form_edit_buffers,
                                            idx + j,
                                            1,
                                        );
                                    }
                                    self.selected = Some(idx);
                                    self.multi_selected.clear();
                                    self.edit_buf = self
                                        .steps
                                        .get(idx)
                                        .map(|s| serde_yml::to_string(s).unwrap_or_default())
                                        .unwrap_or_default();
                                    self.dirty = true;
                                    self.log_ok(format!("{count} ステップを生成しました"));
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Settings test-connection polling ──────────────────────────────────
        if let Some(ref rx) = self.settings_test_rx {
            if let Ok(result) = rx.try_recv() {
                self.settings_test_result = Some(result);
                self.settings_test_rx = None;
                ctx.request_repaint();
            }
        }

        // ── Menu bar ─────────────────────────────────────────────────────────
        #[allow(deprecated)]
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let sty = ui.style_mut();
                sty.spacing.button_padding = egui::vec2(2.0, 0.0);
                sty.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                sty.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                sty.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                sty.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.set_min_size(egui::vec2(
                    ui.available_width(),
                    ui.spacing().interact_size.y,
                ));

                let bar_id = ui.id();

                // FILE
                {
                    let mut bs = egui::menu::BarState::load(ui.ctx(), bar_id);
                    let r = ui.add(egui::Button::new(s.menu_file));
                    bs.bar_menu(&r, |ui| {
                        if ui.button(s.menu_new).clicked() {
                            ui.close();
                            if !self.dirty {
                                self.name = "new_scenario".into();
                                self.steps.clear();
                                self.scenario_vars.clear();
                                self.selected = None;
                                self.path = None;
                                self.dirty = false;
                            } else {
                                self.confirm_dialog = Some(ConfirmAction::NewFile);
                            }
                        }
                        if ui.button(s.menu_open).clicked() {
                            ui.close();
                            self.open_file();
                        }
                        ui.separator();
                        if ui.button(s.menu_save).clicked() {
                            ui.close();
                            self.save_file();
                        }
                        if ui.button(s.menu_save_as).clicked() {
                            ui.close();
                            self.save_file_as();
                        }
                        ui.separator();
                        if ui.button(s.menu_settings).clicked() {
                            ui.close();
                            self.settings_open = true;
                        }
                    });
                    bs.store(ui.ctx(), bar_id);
                }

                // EDIT
                {
                    let mut bs = egui::menu::BarState::load(ui.ctx(), bar_id);
                    let r = ui.add(egui::Button::new(s.menu_edit));
                    bs.bar_menu(&r, |ui| {
                        ui.add_enabled_ui(!self.undo_stack.is_empty(), |ui| {
                            if ui.button(s.menu_undo).clicked() {
                                ui.close();
                                self.undo();
                            }
                        });
                        ui.add_enabled_ui(!self.redo_stack.is_empty(), |ui| {
                            if ui.button(s.menu_redo).clicked() {
                                ui.close();
                                self.redo();
                            }
                        });
                        ui.separator();
                        if ui.button(s.menu_add_step).clicked() {
                            ui.close();
                            self.add_menu_open = true;
                            self.add_menu_just_opened = true;
                            self.add_filter.clear();
                        }
                        ui.separator();
                        let has_sel = !self.multi_selected.is_empty();
                        let has_clip = !self.step_clipboard.is_empty();
                        ui.add_enabled_ui(has_sel, |ui| {
                            if ui.button(s.menu_copy).clicked() {
                                ui.close();
                                self.copy_selected_steps();
                            }
                        });
                        ui.add_enabled_ui(has_sel, |ui| {
                            if ui.button(s.menu_cut).clicked() {
                                ui.close();
                                self.copy_selected_steps();
                                self.delete_selected_steps();
                            }
                        });
                        ui.add_enabled_ui(has_clip, |ui| {
                            if ui.button(s.menu_paste).clicked() {
                                ui.close();
                                self.paste_steps();
                            }
                        });
                        ui.add_enabled_ui(has_sel, |ui| {
                            if ui.button(s.menu_duplicate).clicked() {
                                ui.close();
                                self.copy_selected_steps();
                                self.paste_steps();
                            }
                        });
                        ui.separator();
                        ui.add_enabled_ui(has_sel, |ui| {
                            if ui.button(s.menu_delete_step).clicked() {
                                ui.close();
                                if self.multi_selected.len() > 1 {
                                    let count = self.multi_selected.len();
                                    self.confirm_dialog = Some(ConfirmAction::DeleteSteps(count));
                                } else if let Some(idx) = self.selected {
                                    self.confirm_dialog = Some(ConfirmAction::DeleteStep(idx));
                                }
                            }
                        });
                    });
                    bs.store(ui.ctx(), bar_id);
                }

                // VIEW
                {
                    let mut bs = egui::menu::BarState::load(ui.ctx(), bar_id);
                    let r = ui.add(egui::Button::new(s.menu_view));
                    bs.bar_menu(&r, |ui| {
                        if ui
                            .selectable_label(self.view_mode == ViewMode::List, s.menu_list)
                            .clicked()
                        {
                            self.view_mode = ViewMode::List;
                            ui.close();
                        }
                        if ui
                            .selectable_label(self.view_mode == ViewMode::Flow, s.menu_flow)
                            .clicked()
                        {
                            self.view_mode = ViewMode::Flow;
                            self.selected_child = None;
                            ui.close();
                        }
                        if ui
                            .selectable_label(self.view_mode == ViewMode::Canvas, s.menu_canvas)
                            .clicked()
                        {
                            self.view_mode = ViewMode::Canvas;
                            self.selected_child = None;
                            self.ensure_canvas_layout();
                            ui.close();
                        }
                        ui.separator();
                        if ui
                            .selectable_label(self.ai_panel_open, s.menu_ai_panel)
                            .clicked()
                        {
                            self.ai_panel_open = !self.ai_panel_open;
                            ui.close();
                        }
                    });
                    bs.store(ui.ctx(), bar_id);
                }

                // RUN
                {
                    let mut bs = egui::menu::BarState::load(ui.ctx(), bar_id);
                    let r = ui.add(egui::Button::new(s.menu_run_menu));
                    bs.bar_menu(&r, |ui| {
                        if self.run_child.is_some() {
                            if ui.button(s.menu_stop).clicked() {
                                ui.close();
                                self.stop_run();
                            }
                        } else {
                            if ui.button(s.menu_run).clicked() {
                                ui.close();
                                self.run_scenario();
                            }
                            ui.add_enabled_ui(!self.multi_selected.is_empty(), |ui| {
                                if ui.button(s.menu_run_selection).clicked() {
                                    ui.close();
                                    self.run_selection();
                                }
                            });
                        }
                    });
                    bs.store(ui.ctx(), bar_id);
                }

                // HELP
                {
                    let mut bs = egui::menu::BarState::load(ui.ctx(), bar_id);
                    let r = ui.add(egui::Button::new(s.menu_help));
                    bs.bar_menu(&r, |ui| {
                        if ui
                            .selectable_label(self.manual_open, s.menu_manual)
                            .clicked()
                        {
                            self.manual_open = !self.manual_open;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(s.menu_about).clicked() {
                            ui.close();
                            self.about_open = true;
                        }
                    });
                    bs.store(ui.ctx(), bar_id);
                }
            });
        });

        // ── Toolbar ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 開く").clicked() {
                    self.open_file();
                }
                if ui.button("💾 保存").on_hover_text("保存 (Cmd+S)").clicked() {
                    self.save_file();
                }
                if ui
                    .button("📁 名前を付けて保存")
                    .on_hover_text("名前を付けて保存 (Cmd+Shift+S)")
                    .clicked()
                {
                    self.save_file_as();
                }
                ui.separator();
                ui.add_enabled_ui(!self.undo_stack.is_empty(), |ui| {
                    if ui.button("↶").on_hover_text("アンドゥ (Cmd+Z)").clicked() {
                        self.undo();
                    }
                });
                ui.add_enabled_ui(!self.redo_stack.is_empty(), |ui| {
                    if ui
                        .button("↷")
                        .on_hover_text("リドゥ (Cmd+Shift+Z)")
                        .clicked()
                    {
                        self.redo();
                    }
                });
                ui.separator();
                ui.label(format!("{}:", s.scenario_name_label));
                if ui.text_edit_singleline(&mut self.name).changed() {
                    self.dirty = true;
                }
                ui.separator();
                if self.run_child.is_some() {
                    if ui
                        .button(format!("⏹ {}", s.btn_stop))
                        .on_hover_text(format!("{} (F5)", s.btn_stop))
                        .clicked()
                    {
                        self.stop_run();
                    }
                } else if ui
                    .button(format!("▶ {}", s.btn_run))
                    .on_hover_text(format!("{} (F5)", s.btn_run))
                    .clicked()
                {
                    self.run_scenario();
                }
                if ui
                    .button("📷 採取")
                    .on_hover_text("テンプレート採取ツールを起動 (Ctrl+Shift+C でキャプチャ)")
                    .clicked()
                {
                    open_snip();
                }
                ui.separator();
                if ui
                    .selectable_value(&mut self.view_mode, ViewMode::List, s.menu_list)
                    .clicked()
                {
                    // nothing extra
                }
                if ui
                    .selectable_value(&mut self.view_mode, ViewMode::Flow, s.menu_flow)
                    .clicked()
                {
                    self.selected_child = None;
                    self.scroll_to_selected = true;
                }
                if ui
                    .selectable_value(&mut self.view_mode, ViewMode::Canvas, s.menu_canvas)
                    .clicked()
                {
                    self.selected_child = None;
                    self.ensure_canvas_layout();
                }
                if self.view_mode == ViewMode::Canvas {
                    ui.separator();
                    if ui.button(s.canvas_reset).clicked() {
                        self.push_undo();
                        self.canvas_positions.clear();
                        self.ensure_canvas_layout();
                        self.canvas_layout_dirty = true;
                    }
                    if ui.button(s.canvas_fit).clicked() {
                        self.canvas_fit_view(self.canvas_viewport_size);
                    }
                    if ui
                        .selectable_label(self.settings.canvas_snap, s.canvas_snap)
                        .clicked()
                    {
                        self.settings.canvas_snap = !self.settings.canvas_snap;
                        save_settings(&self.settings);
                    }
                    if ui
                        .selectable_label(self.settings.minimap_show, s.minimap_label)
                        .on_hover_text(s.minimap_tooltip)
                        .clicked()
                    {
                        self.settings.minimap_show = !self.settings.minimap_show;
                        save_settings(&self.settings);
                    }
                    ui.label(format!("{:.0}%", self.canvas_zoom * 100.0));
                    let help_btn = ui.button("?").on_hover_text("キャンバス操作ヘルプ");
                    egui::Popup::from_toggle_button_response(&help_btn)
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            ui.set_min_width(280.0);
                            ui.strong("Canvas 操作ガイド");
                            ui.separator();
                            egui::Grid::new("canvas_help_grid")
                                .num_columns(2)
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Ctrl+スクロール / ピンチ");
                                    ui.label("ズーム (カーソル固定)");
                                    ui.end_row();
                                    ui.label("Cmd+0");
                                    ui.label("全体表示 (fit to view)");
                                    ui.end_row();
                                    ui.label("Cmd+1");
                                    ui.label("100% ズーム + 中央");
                                    ui.end_row();
                                    ui.label("スクロール");
                                    ui.label("上下パン");
                                    ui.end_row();
                                    ui.label("背景ドラッグ");
                                    ui.label("パン");
                                    ui.end_row();
                                    ui.label("Shift+背景ドラッグ");
                                    ui.label("ラッソ選択 (途中Shift押しでも可)");
                                    ui.end_row();
                                    ui.label("Cmd+A");
                                    ui.label("全選択 (Canvasビューのみ)");
                                    ui.end_row();
                                    ui.label("Delete / Backspace");
                                    ui.label("選択ステップを削除 (確認あり)");
                                    ui.end_row();
                                    ui.label("Cmd+C / X / V / D");
                                    ui.label("コピー / カット / 貼付 / 複製");
                                    ui.end_row();
                                    ui.label("クリック");
                                    ui.label("選択 / 背景クリックで解除");
                                    ui.end_row();
                                    ui.label("Cmd+クリック");
                                    ui.label("選択をトグル");
                                    ui.end_row();
                                    ui.label("Shift+クリック");
                                    ui.label("範囲選択");
                                    ui.end_row();
                                    ui.label("ダブルクリック");
                                    ui.label("List ビューで開く");
                                    ui.end_row();
                                    ui.label("右クリック (ノード)");
                                    ui.label("削除 / 複製 / 整列");
                                    ui.end_row();
                                    ui.label("右クリック (背景)");
                                    ui.label("全選択 / 整列");
                                    ui.end_row();
                                    ui.label("2+選択→右クリック");
                                    ui.label("左揃え / 上揃え");
                                    ui.end_row();
                                    ui.label("3+選択→右クリック");
                                    ui.label("水平/垂直等間隔");
                                    ui.end_row();
                                    ui.label("ミニマップクリック");
                                    ui.label("その位置へジャンプ");
                                    ui.end_row();
                                    ui.label("ミニマップドラッグ");
                                    ui.label("連続ナビゲーション");
                                    ui.end_row();
                                    ui.label("↑ / ↓");
                                    ui.label("前/次ノードを選択 + ビュースクロール");
                                    ui.end_row();
                                    ui.label("Cmd+↑ / Cmd+↓");
                                    ui.label("選択ステップを前後に移動");
                                    ui.end_row();
                                    ui.label("Cmd+G");
                                    ui.label("ノード検索バーを開く");
                                    ui.end_row();
                                    ui.label("Cmd+Shift+A");
                                    ui.label("ステップ追加メニュー");
                                    ui.end_row();
                                    ui.label("?");
                                    ui.label("このヘルプを表示");
                                    ui.end_row();
                                });
                        });
                }
                if self.view_mode == ViewMode::Flow {
                    ui.separator();
                    if ui
                        .button("⌂ リセット")
                        .on_hover_text(
                            "ズーム・パンをリセット (Ctrl+ドラッグでパン、Ctrl+スクロールでズーム)",
                        )
                        .clicked()
                    {
                        self.flow_zoom = 1.0;
                        self.flow_pan = egui::Vec2::ZERO;
                    }
                    ui.label(format!("{:.0}%", self.flow_zoom * 100.0));
                }

                // ── Theme toggle (right-aligned) ──────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    use crate::settings::Theme;
                    let (icon, tooltip) = match self.settings.theme {
                        Theme::Light => ("🌙", "ダークモードに切り替え"),
                        Theme::Dark => ("☀", "ライトモードに切り替え"),
                    };
                    if ui.button(icon).on_hover_text(tooltip).clicked() {
                        self.settings.theme = match self.settings.theme {
                            Theme::Light => Theme::Dark,
                            Theme::Dark => Theme::Light,
                        };
                        crate::apply_style(ui.ctx(), &self.settings.theme);
                        save_settings(&self.settings);
                    }
                });
            });
            // Status bar row: last log entry right-aligned
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(last) = self.log.last() {
                        let text = if last.message.chars().count() > 60 {
                            let end = last
                                .message
                                .char_indices()
                                .nth(57)
                                .map(|(i, _)| i)
                                .unwrap_or(last.message.len());
                            format!("{}…", &last.message[..end])
                        } else {
                            last.message.clone()
                        };
                        ui.colored_label(last.level.color(), text);
                    }
                });
            });
        });

        // ── Bottom: tabbed Variables + Log ────────────────────────────────
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .min_height(60.0)
            .default_height(160.0)
            .show(ctx, |ui| {
                let problem_count = self.validate_scenario().len();
                // Auto-switch to Problems tab when new errors appear
                if problem_count > self.prev_problem_count && self.bottom_tab != BottomTab::Problems
                {
                    self.bottom_tab = BottomTab::Problems;
                }
                self.prev_problem_count = problem_count;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Variables, s.panel_vars);
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Log, s.panel_log);
                    if self.bottom_tab == BottomTab::Log && ui.small_button(s.clear).clicked() {
                        self.log.clear();
                    }
                    let problems_label = if problem_count > 0 {
                        format!("⚠ {} ({})", s.tab_problems, problem_count)
                    } else {
                        s.tab_problems.to_owned()
                    };
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Problems, problems_label);
                    if let Some(ref p) = self.path {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.weak(p.display().to_string());
                        });
                    }
                });
                ui.separator();
                match self.bottom_tab {
                    BottomTab::Log => {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for entry in &self.log {
                                    ui.colored_label(
                                        entry.level.color(),
                                        egui::RichText::new(&entry.message).monospace().size(11.0),
                                    );
                                }
                            });
                    }
                    BottomTab::Variables => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui::Grid::new("vars_grid")
                                .striped(true)
                                .num_columns(2)
                                .min_col_width(120.0)
                                .show(ui, |ui| {
                                    ui.strong(s.vars_name_header);
                                    ui.strong(s.vars_initial_header);
                                    ui.end_row();
                                    let keys: Vec<serde_yml::Value> =
                                        self.scenario_vars.keys().cloned().collect();
                                    for key in &keys {
                                        let name_str = key.as_str().unwrap_or("").to_owned();
                                        let val_str = self
                                            .scenario_vars
                                            .get(key)
                                            .map(|v| {
                                                v.as_str().map(|s| s.to_owned()).unwrap_or_else(
                                                    || {
                                                        serde_yml::to_string(v)
                                                            .unwrap_or_default()
                                                            .trim()
                                                            .to_owned()
                                                    },
                                                )
                                            })
                                            .unwrap_or_default();
                                        let is_highlighted = self.var_highlight.as_deref()
                                            == Some(name_str.as_str());
                                        if ui.selectable_label(is_highlighted, &name_str).clicked()
                                        {
                                            if is_highlighted {
                                                self.var_highlight = None;
                                            } else {
                                                self.var_highlight = Some(name_str.clone());
                                            }
                                        }
                                        ui.label(&val_str);
                                        ui.end_row();
                                    }
                                    if self.scenario_vars.is_empty() {
                                        ui.weak("(変数なし)");
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                    BottomTab::Problems => {
                        let issues = self.validate_scenario();
                        let mut jump_to: Option<usize> = None;
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if issues.is_empty() {
                                ui.colored_label(LogLevel::Ok.color(), s.no_problems);
                            } else {
                                for issue in &issues {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(issue.level.color(), &issue.message);
                                        if ui
                                            .small_button(format!("→ ステップ {}", issue.step_idx))
                                            .clicked()
                                        {
                                            jump_to = Some(issue.step_idx);
                                        }
                                    });
                                }
                            }
                        });
                        if let Some(idx) = jump_to {
                            self.select_step(idx);
                        }
                    }
                }
            });

        // ── Left: step palette (permanent tree of available step types) ─────
        let mut palette_insert: Option<&'static str> = None;
        egui::SidePanel::left("step_palette")
            .resizable(true)
            .default_width(220.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(s.panel_nodes);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(s.btn_collapse_all)
                            .on_hover_text("全て閉じる")
                            .clicked()
                        {
                            self.palette_force_open = Some(false);
                        }
                        if ui
                            .small_button(s.btn_expand_all)
                            .on_hover_text("全て展開")
                            .clicked()
                        {
                            self.palette_force_open = Some(true);
                        }
                    });
                });
                ui.separator();
                // ── Search box ────────────────────────────────────────────
                let search_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.nodes_search)
                        .hint_text("🔍 検索…")
                        .desired_width(f32::INFINITY),
                );
                if search_resp.gained_focus() {
                    // Expanding all while searching makes results easier to find.
                    self.palette_force_open = Some(true);
                }
                let query = self.nodes_search.to_lowercase();
                let searching = !query.is_empty();
                if searching && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.nodes_search.clear();
                }
                ui.add_space(2.0);
                let force = if searching {
                    None
                } else {
                    self.palette_force_open.take()
                };
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if searching {
                        // Flat filtered list — no category headers
                        let matches: Vec<_> = STEP_TEMPLATES
                            .iter()
                            .filter(|t| {
                                t.display_name.to_lowercase().contains(&query)
                                    || t.name.to_lowercase().contains(&query)
                                    || t.category.to_lowercase().contains(&query)
                            })
                            .collect();
                        if matches.is_empty() {
                            ui.weak("一致するノードがありません");
                        }
                        for t in matches {
                            let label = egui::RichText::new(format!(
                                "{} {}",
                                step_icon(t.name),
                                t.display_name
                            ))
                            .size(11.0);
                            let drag_id = egui::Id::new(("palette_drag", t.name));
                            let resp = ui
                                .dnd_drag_source(drag_id, DragPayload::NewStep(t.yaml), |ui| {
                                    ui.selectable_label(false, label)
                                })
                                .inner;
                            if resp.double_clicked() {
                                palette_insert = Some(t.yaml);
                            }
                            resp.on_hover_text(format!(
                                "{}\n{}\n\n{}",
                                t.name, s.hint_double_click, t.yaml
                            ));
                        }
                    } else {
                        // Normal category tree
                        let mut cats: Vec<&str> = Vec::new();
                        for t in STEP_TEMPLATES {
                            if !cats.contains(&t.category) {
                                cats.push(t.category);
                            }
                        }
                        for cat in cats {
                            let col = category_color(cat);
                            let hdr = egui::RichText::new(cat).color(col).strong().size(11.0);
                            egui::CollapsingHeader::new(hdr)
                                .default_open(true)
                                .open(force)
                                .show(ui, |ui| {
                                    for t in STEP_TEMPLATES.iter().filter(|t| t.category == cat) {
                                        let label = egui::RichText::new(format!(
                                            "{} {}",
                                            step_icon(t.name),
                                            t.display_name
                                        ))
                                        .size(11.0);
                                        let drag_id = egui::Id::new(("palette_drag", t.name));
                                        let resp = ui
                                            .dnd_drag_source(
                                                drag_id,
                                                DragPayload::NewStep(t.yaml),
                                                |ui| ui.selectable_label(false, label),
                                            )
                                            .inner;
                                        if resp.double_clicked() {
                                            palette_insert = Some(t.yaml);
                                        }
                                        resp.on_hover_text(format!(
                                            "{}\n{}\n\n{}",
                                            t.name, s.hint_double_click, t.yaml
                                        ));
                                    }
                                });
                        }
                    }
                });
            });

        // ── Left: step list ───────────────────────────────────────────────
        egui::SidePanel::left("steps_panel")
            .min_width(240.0)
            .show(ctx, |ui| {
                ui.heading(s.panel_steps);
                ui.separator();

                let mut action: Option<StepAction> = None;
                let step_count = self.steps.len();

                // Collect row rects for insertion-point computation (populated inside the loop).
                let mut row_rects: Vec<egui::Rect> = Vec::with_capacity(step_count);
                // Deferred drop handling (resolved after the closure).
                let mut drop_payload: Option<DragPayload> = None;
                // Insertion index computed from hover position while dragging.
                let mut drop_insert_idx: usize = step_count;

                egui::ScrollArea::vertical()
                    .id_salt("step_list")
                    .show(ui, |ui| {
                        let (inner_resp, released_payload) =
                            ui.dnd_drop_zone::<DragPayload, _>(egui::Frame::default(), |ui| {
                                for i in 0..step_count {
                                    let key = get_step_key(&self.steps[i]);
                                    let cat = step_key_category(key);
                                    let color = category_color(cat);
                                    let summary = step_summary(&self.steps[i]);
                                    let is_multi = self.multi_selected.contains(&i);
                                    let is_primary = self.selected == Some(i);
                                    let is_running = self.current_run_step == Some(i);
                                    let is_disabled =
                                        crate::state::EditorApp::step_is_disabled(&self.steps[i]);
                                    let is_var_match = if let Some(ref vh) = self.var_highlight {
                                        let pattern = format!("{{{{ {vh} }}}}");
                                        serde_yml::to_string(&self.steps[i])
                                            .map(|y| y.contains(&pattern))
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    let drag_id = egui::Id::new(("step_drag", i));
                                    let label_text = if is_running {
                                        format!("▶ {}: {summary}", i + 1)
                                    } else if is_disabled {
                                        format!("⊘ {}: {summary}", i + 1)
                                    } else {
                                        format!("{}: {summary}", i + 1)
                                    };

                                    let drag_indices = if self.multi_selected.contains(&i)
                                        && self.multi_selected.len() > 1
                                    {
                                        let mut v: Vec<usize> =
                                            self.multi_selected.iter().copied().collect();
                                        v.sort_unstable();
                                        v
                                    } else {
                                        vec![i]
                                    };
                                    let row_resp = ui
                                        .dnd_drag_source(
                                            drag_id,
                                            DragPayload::ReorderStep(drag_indices),
                                            |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.set_min_height(
                                                        crate::tokens::STEP_ROW_HEIGHT,
                                                    ); // DESIGN.md §3.2
                                                    let bar_color = if is_running {
                                                        crate::tokens::WARNING
                                                    } else {
                                                        color
                                                    };
                                                    ui.colored_label(bar_color, "▌");
                                                    // Enable/disable toggle
                                                    let toggle_icon =
                                                        if is_disabled { "○" } else { "●" };
                                                    let toggle_color = if is_disabled {
                                                        egui::Color32::from_gray(90)
                                                    } else {
                                                        egui::Color32::from_gray(160)
                                                    };
                                                    if ui
                                                        .add(
                                                            egui::Label::new(
                                                                egui::RichText::new(toggle_icon)
                                                                    .color(toggle_color)
                                                                    .size(10.0),
                                                            )
                                                            .sense(egui::Sense::click()),
                                                        )
                                                        .on_hover_text(if is_disabled {
                                                            "クリックで有効化"
                                                        } else {
                                                            "クリックで無効化"
                                                        })
                                                        .clicked()
                                                    {
                                                        action = Some(StepAction::ToggleEnabled(i));
                                                    }
                                                    let label_richtext = if is_disabled {
                                                        egui::RichText::new(&label_text)
                                                            .color(egui::Color32::from_gray(100))
                                                            .strikethrough()
                                                    } else {
                                                        egui::RichText::new(&label_text)
                                                    };
                                                    let resp = ui.selectable_label(
                                                        is_primary,
                                                        label_richtext,
                                                    );
                                                    if is_multi && !is_primary && !is_running {
                                                        ui.painter().rect_filled(
                                                            resp.rect.expand2(egui::vec2(2.0, 1.0)),
                                                            2.0,
                                                            egui::Color32::from_rgba_premultiplied(
                                                                0, 28, 50,
                                                                60, // ACCENT #0078D4 @ 24% alpha
                                                            ),
                                                        );
                                                    }
                                                    if is_var_match
                                                        && !is_primary
                                                        && !is_multi
                                                        && !is_running
                                                    {
                                                        ui.painter().rect_filled(
                                                            resp.rect.expand2(egui::vec2(2.0, 1.0)),
                                                            2.0,
                                                            egui::Color32::from_rgba_premultiplied(
                                                                180, 120, 20, 50,
                                                            ),
                                                        );
                                                    }
                                                    if resp.clicked() {
                                                        let (ctrl, shift) = ui.input(|inp| {
                                                            (
                                                                inp.modifiers.command,
                                                                inp.modifiers.shift,
                                                            )
                                                        });
                                                        if ctrl {
                                                            action =
                                                                Some(StepAction::ToggleMulti(i));
                                                        } else if shift {
                                                            action =
                                                                Some(StepAction::ShiftSelect(i));
                                                        } else {
                                                            action = Some(StepAction::Select(i));
                                                        }
                                                    }
                                                    ui.add_enabled_ui(i > 0, |ui| {
                                                        if ui
                                                            .small_button("↑")
                                                            .on_hover_cursor(
                                                                egui::CursorIcon::PointingHand,
                                                            )
                                                            .clicked()
                                                        {
                                                            action = Some(StepAction::MoveUp(i));
                                                        }
                                                    });
                                                    ui.add_enabled_ui(i + 1 < step_count, |ui| {
                                                        if ui
                                                            .small_button("↓")
                                                            .on_hover_cursor(
                                                                egui::CursorIcon::PointingHand,
                                                            )
                                                            .clicked()
                                                        {
                                                            action = Some(StepAction::MoveDown(i));
                                                        }
                                                    });
                                                    if ui
                                                        .small_button("✕")
                                                        .on_hover_cursor(
                                                            egui::CursorIcon::PointingHand,
                                                        )
                                                        .clicked()
                                                    {
                                                        action = Some(StepAction::Delete(i));
                                                    }
                                                })
                                            },
                                        )
                                        .response;

                                    row_rects.push(row_resp.rect);
                                }
                            });

                        // Compute insertion index from hover position while a drag is active.
                        let is_dragging =
                            egui::DragAndDrop::has_payload_of_type::<DragPayload>(ui.ctx());
                        if is_dragging {
                            if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                                drop_insert_idx = step_count; // default: append
                                for (ri, rect) in row_rects.iter().enumerate() {
                                    if hover_pos.y < rect.center().y {
                                        drop_insert_idx = ri;
                                        break;
                                    }
                                }
                                // Draw insertion line
                                let line_y = if drop_insert_idx < row_rects.len() {
                                    row_rects[drop_insert_idx].top()
                                } else {
                                    row_rects
                                        .last()
                                        .map(|r| r.bottom())
                                        .unwrap_or(inner_resp.response.rect.top())
                                };
                                let x_min = inner_resp.response.rect.left();
                                let x_max = inner_resp.response.rect.right();
                                ui.painter().line_segment(
                                    [egui::pos2(x_min, line_y), egui::pos2(x_max, line_y)],
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255)),
                                );
                            }
                        }

                        if let Some(payload) = released_payload {
                            drop_payload = Some((*payload).clone());
                        }
                    });

                ui.separator();
                if ui.button(s.btn_add_step).clicked() {
                    self.add_menu_open = true;
                    self.add_menu_just_opened = true;
                    self.add_filter.clear();
                }
                if self.multi_selected.len() > 1 {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 140, 220),
                        format!("{} 件選択中", self.multi_selected.len()),
                    );
                }

                if let Some(act) = action {
                    match act {
                        StepAction::Select(i) => self.select_step(i),
                        StepAction::ToggleMulti(i) => {
                            if self.multi_selected.contains(&i) {
                                self.multi_selected.remove(&i);
                                if self.selected == Some(i) {
                                    self.selected = self.multi_selected.iter().next().cloned();
                                }
                            } else {
                                self.multi_selected.insert(i);
                                self.flush_edit();
                                self.selected = Some(i);
                                if let Some(step) = self.steps.get(i) {
                                    self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
                                }
                            }
                        }
                        StepAction::ShiftSelect(i) => {
                            let anchor = self.selected.unwrap_or(i);
                            let (lo, hi) = if anchor <= i {
                                (anchor, i)
                            } else {
                                (i, anchor)
                            };
                            self.multi_selected = (lo..=hi).collect();
                            self.selected = Some(i);
                            if let Some(step) = self.steps.get(i) {
                                self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
                            }
                        }
                        StepAction::MoveUp(i) => {
                            self.push_undo();
                            self.steps.swap(i - 1, i);
                            // Swap canvas positions
                            let pos_a = self.canvas_positions.get(&i).copied();
                            let pos_b = self.canvas_positions.get(&(i - 1)).copied();
                            if let Some(a) = pos_a {
                                self.canvas_positions.insert(i - 1, a);
                            }
                            if let Some(b) = pos_b {
                                self.canvas_positions.insert(i, b);
                            }
                            if self.selected == Some(i) {
                                self.selected = Some(i - 1);
                            }
                            self.multi_selected = self
                                .multi_selected
                                .iter()
                                .map(|&x| {
                                    if x == i {
                                        i - 1
                                    } else if x == i - 1 {
                                        i
                                    } else {
                                        x
                                    }
                                })
                                .collect();
                            self.form_edit_buffers.clear();
                        }
                        StepAction::MoveDown(i) => {
                            self.push_undo();
                            self.steps.swap(i, i + 1);
                            // Swap canvas positions
                            let pos_a = self.canvas_positions.get(&i).copied();
                            let pos_b = self.canvas_positions.get(&(i + 1)).copied();
                            if let Some(a) = pos_a {
                                self.canvas_positions.insert(i + 1, a);
                            }
                            if let Some(b) = pos_b {
                                self.canvas_positions.insert(i, b);
                            }
                            if self.selected == Some(i) {
                                self.selected = Some(i + 1);
                            }
                            self.multi_selected = self
                                .multi_selected
                                .iter()
                                .map(|&x| {
                                    if x == i {
                                        i + 1
                                    } else if x == i + 1 {
                                        i
                                    } else {
                                        x
                                    }
                                })
                                .collect();
                            self.form_edit_buffers.clear();
                        }
                        StepAction::Delete(i) => {
                            if self.multi_selected.len() > 1 {
                                let count = self.multi_selected.len();
                                self.confirm_dialog = Some(ConfirmAction::DeleteSteps(count));
                            } else {
                                self.confirm_dialog = Some(ConfirmAction::DeleteStep(i));
                            }
                        }
                        StepAction::ToggleEnabled(i) => {
                            if i < self.steps.len() {
                                self.push_undo();
                                Self::toggle_step_enabled(&mut self.steps[i]);
                                self.dirty = true;
                            }
                        }
                    }
                }

                // ── Handle drag-and-drop drops ───────────────────────────
                if let Some(payload) = drop_payload {
                    match payload {
                        DragPayload::NewStep(yaml) => {
                            if let Ok(v) = serde_yml::from_str::<serde_yml::Value>(yaml) {
                                self.push_undo();
                                self.steps.insert(drop_insert_idx, v);
                                Self::canvas_shift_positions(
                                    &mut self.canvas_positions,
                                    drop_insert_idx,
                                    1,
                                );
                                self.select_step(drop_insert_idx);
                                self.log_info("ノードをドロップしました");
                            }
                        }
                        DragPayload::ReorderStep(from_indices) => {
                            if from_indices.len() == 1 {
                                let from = from_indices[0];
                                if from != drop_insert_idx && from + 1 != drop_insert_idx {
                                    self.push_undo();
                                    let step = self.steps.remove(from);
                                    let to = if drop_insert_idx > from {
                                        drop_insert_idx - 1
                                    } else {
                                        drop_insert_idx
                                    };
                                    self.steps.insert(to, step);
                                    let src_pos = self.canvas_positions.get(&from).copied();
                                    Self::canvas_shift_positions(
                                        &mut self.canvas_positions,
                                        from,
                                        -1,
                                    );
                                    Self::canvas_shift_positions(&mut self.canvas_positions, to, 1);
                                    if let Some(pos) = src_pos {
                                        self.canvas_positions.insert(to, pos);
                                    }
                                    self.selected = Some(to);
                                    self.multi_selected.clear();
                                    self.multi_selected.insert(to);
                                    if let Some(s) = self.steps.get(to) {
                                        self.edit_buf = serde_yml::to_string(s).unwrap_or_default();
                                    }
                                    self.form_edit_buffers.clear();
                                    self.dirty = true;
                                    self.log_info("ステップを並び替えました");
                                }
                            } else {
                                // Multi-select reorder: extract selected steps, insert at target
                                let target = drop_insert_idx;
                                let all_in_selection: bool =
                                    from_indices.iter().all(|&x| x < self.steps.len());
                                if all_in_selection {
                                    self.push_undo();
                                    // Collect steps in order
                                    let mut extracted: Vec<serde_yml::Value> = from_indices
                                        .iter()
                                        .map(|&x| self.steps[x].clone())
                                        .collect();
                                    // Remove in reverse order to preserve indices
                                    let mut sorted_from = from_indices.clone();
                                    sorted_from.sort_unstable_by(|a, b| b.cmp(a));
                                    for idx in &sorted_from {
                                        self.steps.remove(*idx);
                                    }
                                    // Adjust insertion point after removals
                                    let removed_before =
                                        from_indices.iter().filter(|&&x| x < target).count();
                                    let insert_at =
                                        target.saturating_sub(removed_before).min(self.steps.len());
                                    // Insert extracted steps at target
                                    for (offset, step) in extracted.drain(..).enumerate() {
                                        self.steps.insert(insert_at + offset, step);
                                    }
                                    // Remap canvas positions to follow the moved steps.
                                    // new_order[new_idx] = old_idx that now lives at new_idx.
                                    {
                                        let n_total = self.steps.len();
                                        let moved_set: HashSet<usize> =
                                            from_indices.iter().cloned().collect();
                                        let remaining: Vec<usize> = (0..n_total)
                                            .filter(|i| !moved_set.contains(i))
                                            .collect();
                                        let split = insert_at.min(remaining.len());
                                        let new_order: Vec<usize> = remaining[..split]
                                            .iter()
                                            .chain(from_indices.iter())
                                            .chain(remaining[split..].iter())
                                            .cloned()
                                            .collect();
                                        let old_pos_map: HashMap<usize, egui::Pos2> =
                                            self.canvas_positions.drain().collect();
                                        for (new_i, old_i) in new_order.iter().enumerate() {
                                            if let Some(&p) = old_pos_map.get(old_i) {
                                                self.canvas_positions.insert(new_i, p);
                                            }
                                        }
                                    }
                                    // Update selection to follow first moved step
                                    let new_sel = insert_at;
                                    self.selected = Some(new_sel);
                                    self.multi_selected.clear();
                                    for offset in 0..from_indices.len() {
                                        self.multi_selected.insert(insert_at + offset);
                                    }
                                    if let Some(s) = self.steps.get(new_sel) {
                                        self.edit_buf = serde_yml::to_string(s).unwrap_or_default();
                                    }
                                    self.form_edit_buffers.clear();
                                    self.dirty = true;
                                    self.log_info("ステップを並び替えました");
                                }
                            }
                        }
                    }
                }
            });

        // ── AI side panel (must be before CentralPanel) ───────────────────
        self.show_ai_panel(ctx);

        // ── Center: flowchart or YAML editor / onboarding ────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.view_mode == ViewMode::Flow {
                self.show_flowchart(ui);
                return;
            }
            if self.view_mode == ViewMode::Canvas {
                let drop_screen_pos = ui.input(|i| i.pointer.hover_pos());
                let (canvas_outer, released) =
                    ui.dnd_drop_zone::<DragPayload, _>(egui::Frame::default(), |ui| {
                        self.show_canvas(ui);
                    });
                if let Some(payload) = released {
                    if let DragPayload::NewStep(yaml) = *payload {
                        if let Ok(v) = serde_yml::from_str::<serde_yml::Value>(yaml) {
                            self.push_undo();
                            let idx = self.steps.len();
                            self.steps.push(v);
                            let z = self.canvas_zoom;
                            let origin = canvas_outer.response.rect.min;
                            let canvas_pos = if let Some(sp) = drop_screen_pos {
                                let rel = sp - origin;
                                egui::pos2(
                                    rel.x / z - self.canvas_pan.x,
                                    rel.y / z - self.canvas_pan.y,
                                )
                            } else {
                                default_canvas_pos(idx, default_canvas_cols(idx + 1))
                            };
                            self.canvas_positions.insert(idx, canvas_pos);
                            self.select_step(idx);
                            self.log_info("ノードをドロップしました");
                        }
                    }
                }
                return;
            }

            if let Some(idx) = self.selected {
                // Inline parse error banner
                if let Some(ref err) = self.parse_error.clone() {
                    egui::Frame::new()
                        .fill(crate::tokens::ERROR.gamma_multiply(0.35))
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 160, 150), // tokens::ERROR の明るいトーン
                                format!("⚠ 構文エラー: {err}"),
                            );
                        });
                }

                let key = get_step_key(&self.steps[idx]);
                let cat = step_key_category(key);
                let color = category_color(cat);
                ui.horizontal(|ui| {
                    ui.colored_label(color, "▌");
                    ui.label(
                        egui::RichText::new(format!(
                            "ステップ {}  —  {}",
                            idx,
                            step_display_name(key)
                        ))
                        .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.selectable_value(&mut self.prop_view, PropView::Yaml, "YAML");
                        ui.selectable_value(&mut self.prop_view, PropView::Form, "フォーム");
                    });
                });
                ui.separator();

                if self.prop_view == PropView::Form {
                    self.show_property_form(ui, idx);
                } else {
                    let response =
                        egui::ScrollArea::vertical()
                            .id_salt("yaml_editor")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.edit_buf)
                                        .code_editor()
                                        .desired_rows(20)
                                        .desired_width(f32::INFINITY),
                                )
                            });
                    if response.inner.gained_focus() {
                        // Snapshot before the user starts typing so Cmd+Z can revert the edit
                        let snap = self.snapshot();
                        self.undo_stack.push_back(snap);
                        self.redo_stack.clear();
                    }
                    if response.inner.changed() {
                        self.flush_edit();
                    }
                }
            } else if self.steps.is_empty() {
                // Onboarding guide
                ui.add_space(50.0);
                ui.vertical_centered(|ui| {
                    ui.heading("robost");
                    ui.add_space(20.0);
                    ui.label(s.onboard_1);
                    ui.add_space(8.0);
                    ui.label(s.onboard_2);
                    ui.add_space(8.0);
                    ui.label(s.onboard_3);
                    ui.add_space(8.0);
                    ui.label(s.onboard_4);
                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.weak(s.onboard_open);
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(s.empty_select_step);
                });
            }
        });

        // ── Add step popup ────────────────────────────────────────────────
        if self.add_menu_open {
            let mut close = false;
            let mut insert: Option<&'static str> = None;

            egui::Window::new("ステップを追加")
                .collapsible(false)
                .resizable(true)
                .default_size([300.0, 500.0])
                .show(ctx, |ui| {
                    let filter_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.add_filter)
                            .hint_text("検索…")
                            .desired_width(f32::INFINITY),
                    );
                    if self.add_menu_just_opened {
                        filter_resp.request_focus();
                        self.add_menu_just_opened = false;
                        self.add_menu_selected_idx = 0;
                    }
                    ui.separator();

                    let prev_filter = self.add_filter.clone();
                    let filter = self.add_filter.to_lowercase();

                    // Collect visible templates for keyboard navigation
                    let visible_templates: Vec<&StepTemplate> = STEP_TEMPLATES
                        .iter()
                        .filter(|t| {
                            filter.is_empty()
                                || t.name.to_lowercase().contains(&filter)
                                || t.display_name.to_lowercase().contains(&filter)
                                || t.category.to_lowercase().contains(&filter)
                        })
                        .collect();
                    let n_visible = visible_templates.len();

                    // Reset keyboard selection when filter changes
                    let _ = prev_filter; // consumed above
                    if filter_resp.changed() {
                        self.add_menu_selected_idx = 0;
                    }

                    // Handle arrow-key and Enter navigation
                    if n_visible > 0 {
                        let (up, down, enter) = ui.input_mut(|i| {
                            (
                                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                            )
                        });
                        if down {
                            self.add_menu_selected_idx =
                                (self.add_menu_selected_idx + 1) % n_visible;
                        }
                        if up {
                            self.add_menu_selected_idx =
                                self.add_menu_selected_idx.saturating_sub(1);
                        }
                        self.add_menu_selected_idx =
                            self.add_menu_selected_idx.min(n_visible.saturating_sub(1));
                        if enter {
                            insert = Some(visible_templates[self.add_menu_selected_idx].yaml);
                            close = true;
                        }
                    } else if ui
                        .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
                    {
                        // no-op: nothing to insert
                    }

                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| {
                            if filter.is_empty() {
                                // Tree view: one CollapsingHeader per category
                                let mut cats: Vec<&str> = Vec::new();
                                for t in STEP_TEMPLATES {
                                    if !cats.contains(&t.category) {
                                        cats.push(t.category);
                                    }
                                }
                                let mut flat_idx: usize = 0;
                                for cat in cats {
                                    let col = category_color(cat);
                                    let hdr = egui::RichText::new(cat).color(col).strong();
                                    egui::CollapsingHeader::new(hdr).default_open(true).show(
                                        ui,
                                        |ui| {
                                            for t in
                                                STEP_TEMPLATES.iter().filter(|t| t.category == cat)
                                            {
                                                let is_kbd_sel =
                                                    flat_idx == self.add_menu_selected_idx;
                                                let label = egui::RichText::new(format!(
                                                    "  {} ({})",
                                                    t.display_name, t.name
                                                ))
                                                .size(12.0);
                                                if ui.selectable_label(is_kbd_sel, label).clicked()
                                                {
                                                    insert = Some(t.yaml);
                                                    close = true;
                                                }
                                                flat_idx += 1;
                                            }
                                        },
                                    );
                                }
                            } else {
                                // Flat filtered list with keyboard highlight
                                for (i, t) in visible_templates.iter().enumerate() {
                                    let is_kbd_sel = i == self.add_menu_selected_idx;
                                    let col = category_color(t.category);
                                    let label = egui::RichText::new(format!(
                                        "{} / {} ({})",
                                        t.category, t.display_name, t.name
                                    ))
                                    .size(12.0);
                                    let btn = egui::Button::new(label)
                                        .fill(if is_kbd_sel {
                                            egui::Color32::from_rgba_unmultiplied(
                                                col.r(),
                                                col.g(),
                                                col.b(),
                                                60,
                                            )
                                        } else {
                                            egui::Color32::from_rgba_unmultiplied(
                                                col.r(),
                                                col.g(),
                                                col.b(),
                                                18,
                                            )
                                        })
                                        .min_size(egui::vec2(250.0, 26.0));
                                    if ui.add(btn).clicked() {
                                        insert = Some(t.yaml);
                                        close = true;
                                    }
                                }
                            }
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(s.btn_cancel).clicked() {
                            close = true;
                        }
                        ui.weak("Esc / ↑↓ で選択 / Enter で挿入");
                    });
                });

            if let Some(yaml) = insert {
                match serde_yml::from_str::<serde_yml::Value>(yaml) {
                    Ok(v) => {
                        self.push_undo();
                        if let Some((parent_idx, branch_name)) = self.add_branch_target.take() {
                            // Insert into a branch sub-list
                            let new_ci = self
                                .get_branch_steps(parent_idx, &branch_name)
                                .map_or(0, |s| s.len());
                            self.add_branch_step(parent_idx, &branch_name, v.clone());
                            if let Some(updated) = self.get_branch_steps(parent_idx, &branch_name) {
                                if new_ci < updated.len() {
                                    self.child_edit_buf =
                                        serde_yml::to_string(&updated[new_ci]).unwrap_or_default();
                                }
                            }
                            self.selected_child = Some((branch_name, new_ci));
                            self.log_info("ブランチにステップを追加しました");
                        } else {
                            let idx = self.selected.map(|i| i + 1).unwrap_or(self.steps.len());
                            self.steps.insert(idx, v);
                            Self::canvas_shift_positions(&mut self.canvas_positions, idx, 1);
                            if let Some(pos) = self.canvas_pending_insert_pos.take() {
                                self.canvas_positions.insert(idx, pos);
                            }
                            self.select_step(idx);
                            self.log_info("ステップを追加しました");
                        }
                    }
                    Err(e) => self.log_err(format!("テンプレートエラー: {e} — ステップテンプレートの YAML 構文を確認してください")),
                }
            }
            if close {
                self.add_menu_open = false;
                self.add_branch_target = None;
            }
        }

        // ── Handle palette double-click insert ───────────────────────────
        if let Some(yaml) = palette_insert {
            if let Ok(v) = serde_yml::from_str::<serde_yml::Value>(yaml) {
                self.push_undo();
                let idx = self.selected.map(|i| i + 1).unwrap_or(self.steps.len());
                self.steps.insert(idx, v);
                Self::canvas_shift_positions(&mut self.canvas_positions, idx, 1);
                self.select_step(idx);
                self.log_info("ノードを追加しました");
            }
        }

        // ── Settings window ───────────────────────────────────────────────
        self.show_settings_window(ctx);

        // ── Manual window ─────────────────────────────────────────────────
        self.show_manual_window(ctx);

        // ── About dialog ──────────────────────────────────────────────────
        if self.about_open {
            let s = S::for_lang(&self.settings.lang);
            let mut open = true;
            egui::Window::new(s.about_title)
                .collapsible(false)
                .resizable(false)
                .default_width(320.0)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.heading("robost");
                        ui.label(s.about_rpa_tool);
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);
                        egui::Grid::new("about_grid")
                            .num_columns(2)
                            .spacing([16.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(s.about_version);
                                ui.label(env!("CARGO_PKG_VERSION"));
                                ui.end_row();
                                ui.label(s.about_license);
                                ui.label("MIT OR Apache-2.0");
                                ui.end_row();
                            });
                        ui.add_space(8.0);
                    });
                });
            self.about_open = open;
        }

        // ── Toast notifications overlay ───────────────────────────────────
        let now = std::time::Instant::now();
        self.toasts.retain(|t| t.expires > now);
        if !self.toasts.is_empty() {
            ctx.request_repaint();
            let screen = ctx.input(|i| i.viewport_rect());
            let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("toasts"));
            let painter = ctx.layer_painter(layer);
            let mut y = screen.min.y + 48.0;
            let x_right = screen.max.x - 12.0;
            let font_id = egui::FontId::proportional(13.0);
            let padding = egui::Vec2::new(10.0, 6.0);
            let toast_width = 280.0_f32;
            let toast_height = 30.0_f32;
            for toast in &self.toasts {
                let bg = match toast.level {
                    LogLevel::Ok => egui::Color32::from_rgba_premultiplied(30, 100, 50, 220),
                    LogLevel::Error => egui::Color32::from_rgba_premultiplied(120, 30, 30, 220),
                    LogLevel::Info => egui::Color32::from_rgba_premultiplied(40, 60, 100, 220),
                };
                let text_color = egui::Color32::WHITE;
                let rect = egui::Rect::from_min_size(
                    egui::pos2(x_right - toast_width, y),
                    egui::vec2(toast_width, toast_height),
                );
                painter.rect_filled(rect, 6.0, bg);
                painter.text(
                    rect.min + padding,
                    egui::Align2::LEFT_TOP,
                    &toast.message,
                    font_id.clone(),
                    text_color,
                );
                y += toast_height + 4.0;
            }
        }

        if self.canvas_layout_dirty {
            self.save_canvas_layout();
            self.canvas_layout_dirty = false;
        }
    }
}

fn open_snip() {
    let bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("robost-snip")))
        .unwrap_or_else(|| std::path::PathBuf::from("robost-snip"));
    if let Err(e) = std::process::Command::new(&bin).spawn() {
        tracing::error!("robost-snip の起動に失敗しました: {e}");
    }
}

/// SEC-4: replace any line that looks like it contains a secret value with a
/// redacted placeholder before writing to the visible log panel.
fn redact_secret_line(line: &str) -> std::borrow::Cow<'_, str> {
    let lower = line.to_ascii_lowercase();
    if lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("api_key")
        || lower.contains("apikey")
    {
        "[redacted — line may contain secret value]".into()
    } else {
        line.into()
    }
}
