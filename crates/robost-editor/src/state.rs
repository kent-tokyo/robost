// ---- EditorApp state --------------------------------------------------------

use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::flow_helpers::{
    build_scenario_yaml, collect_nodes, parse_scenario_steps, NODE_H, NODE_W,
};
use crate::settings::{load_settings, AppSettings};
use crate::types::{
    AiMessage, BottomTab, CanvasComment, ConfirmAction, EditorState, FlowNode, LogEntry, LogLevel,
    PropView, Toast, ValidationIssue, ViewMode,
};

pub(crate) struct EditorApp {
    pub(crate) path: Option<PathBuf>,
    pub(crate) name: String,
    pub(crate) steps: Vec<serde_yml::Value>,
    pub(crate) selected: Option<usize>,
    pub(crate) edit_buf: String,
    pub(crate) parse_error: Option<String>,
    pub(crate) add_menu_open: bool,
    pub(crate) add_menu_just_opened: bool,
    pub(crate) add_filter: String,
    pub(crate) add_menu_selected_idx: usize,
    pub(crate) log: Vec<LogEntry>,
    pub(crate) toasts: Vec<Toast>,
    pub(crate) view_mode: ViewMode,
    pub(crate) expanded_steps: HashSet<usize>,
    pub(crate) undo_stack: VecDeque<EditorState>,
    pub(crate) redo_stack: VecDeque<EditorState>,
    pub(crate) flow_zoom: f32,
    pub(crate) flow_pan: egui::Vec2,
    pub(crate) run_progress_file: Option<PathBuf>,
    pub(crate) current_run_step: Option<usize>,
    pub(crate) last_progress_check: std::time::Instant,
    pub(crate) dirty: bool,
    pub(crate) run_child: Option<std::process::Child>,
    /// Receives stderr lines from the rpa child process in real time.
    pub(crate) stderr_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// Temp YAML file held for cleanup after the run ends (BUG-2).
    pub(crate) run_tmp_file: Option<std::path::PathBuf>,
    /// Step offset used when `run_from_step` runs a slice — maps rpa progress
    /// indices (0-relative to the slice) back to original canvas indices (BUG-1).
    pub(crate) run_from_offset: usize,
    pub(crate) prop_view: PropView,
    pub(crate) selected_child: Option<(String, usize)>,
    pub(crate) child_edit_buf: String,
    pub(crate) child_parse_error: Option<String>,
    /// When set, branch "+" opens the step picker targeting this branch.
    pub(crate) add_branch_target: Option<(usize, String)>,
    pub(crate) confirm_dialog: Option<ConfirmAction>,
    /// Persistent raw-string buffers for numeric property-form fields.
    /// Key: "fieldname@step_idx", Value: current edit string.
    pub(crate) form_edit_buffers: HashMap<String, String>,
    pub(crate) settings: AppSettings,
    pub(crate) settings_open: bool,
    pub(crate) about_open: bool,
    pub(crate) ai_panel_open: bool,
    pub(crate) ai_messages: Vec<AiMessage>,
    pub(crate) ai_input: String,
    pub(crate) ai_loading: bool,
    pub(crate) ai_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub(crate) ai_unread: bool,
    pub(crate) md_cache: egui_commonmark::CommonMarkCache,
    pub(crate) manual_open: bool,
    pub(crate) manual_search: String,
    /// When true, flowchart will pan to center the selected node on next frame.
    pub(crate) scroll_to_selected: bool,
    pub(crate) settings_test_result: Option<(bool, String)>,
    pub(crate) settings_test_rx: Option<std::sync::mpsc::Receiver<(bool, String)>>,
    pub(crate) scenario_vars: serde_yml::Mapping,
    pub(crate) bottom_tab: BottomTab,
    /// Steps currently highlighted in multi-select (always contains `selected` when set).
    pub(crate) multi_selected: HashSet<usize>,
    /// Internal clipboard for copy/cut/paste of steps.
    pub(crate) step_clipboard: Vec<serde_yml::Value>,
    /// When Some, forces all node-palette categories open (true) or closed (false) for one frame.
    pub(crate) palette_force_open: Option<bool>,
    pub(crate) canvas_positions: HashMap<usize, egui::Pos2>,
    pub(crate) canvas_zoom: f32,
    pub(crate) canvas_pan: egui::Vec2,
    pub(crate) canvas_dragging: Option<(usize, egui::Vec2)>,
    pub(crate) undo_pushed_for_current_drag: bool,
    pub(crate) canvas_layout_dirty: bool,
    pub(crate) canvas_lasso: Option<(egui::Pos2, egui::Pos2)>,
    pub(crate) minimap_dragging: bool,
    /// Anchor node for Shift+click range selection. Persists through background clicks
    /// so clearing selection does not break the next range-select.
    pub(crate) canvas_selection_anchor: Option<usize>,
    pub(crate) canvas_viewport_size: egui::Vec2,
    /// When Some, step list rows referencing this variable name get an amber tint.
    pub(crate) var_highlight: Option<String>,
    /// Active category filter in the manual window (independent of text search).
    pub(crate) manual_category_filter: Option<&'static str>,
    /// Tracks when the undo-limit warning toast was last shown (throttles the toast).
    pub(crate) undo_limit_warned_at: Option<std::time::Instant>,
    /// Background channel for ai_create step generation. Holds (step_idx, receiver).
    pub(crate) ai_step_rx: Option<(usize, std::sync::mpsc::Receiver<anyhow::Result<String>>)>,
    /// Per-step error from the most recent ai_create generation attempt.
    pub(crate) ai_step_error: Option<(usize, String)>,
    /// Steps that failed during canvas execution, mapped to their error messages.
    pub(crate) canvas_error_steps: HashMap<usize, String>,
    /// Steps that completed successfully in the most recent run.
    pub(crate) canvas_completed_steps: HashSet<usize>,
    /// Active edge-drag: (source step index, current pointer screen position).
    pub(crate) canvas_edge_drag: Option<(usize, egui::Pos2)>,
    /// When lasso was started with Shift held, new selection extends existing multi_selected.
    pub(crate) canvas_lasso_additive: bool,
    /// Canvas-space position set by background right-click "Add Step Here"; consumed on insert.
    pub(crate) canvas_pending_insert_pos: Option<egui::Pos2>,
    /// Current canvas node search query.
    pub(crate) canvas_search: String,
    /// Whether the canvas search bar is visible.
    pub(crate) canvas_search_open: bool,
    /// Whether the canvas keyboard shortcut help overlay is visible.
    pub(crate) canvas_help_open: bool,
    /// Sticky-note comments on the canvas (persisted in .layout.json, not scenario YAML).
    pub(crate) canvas_comments: Vec<CanvasComment>,
    /// Index into canvas_comments of the comment being dragged, and the drag offset.
    pub(crate) canvas_comment_drag: Option<(usize, egui::Vec2)>,
    /// Index into canvas_comments of the comment being edited (double-clicked).
    pub(crate) canvas_editing_comment: Option<usize>,
    /// Counter for generating unique comment IDs.
    pub(crate) canvas_comment_next_id: u64,
    /// Problem count from the previous frame — used to detect when new errors appear.
    pub(crate) prev_problem_count: usize,
}

impl Default for EditorApp {
    fn default() -> Self {
        Self {
            path: None,
            name: "new_scenario".into(),
            steps: Vec::new(),
            selected: None,
            edit_buf: String::new(),
            parse_error: None,
            add_menu_open: false,
            add_menu_just_opened: false,
            add_filter: String::new(),
            add_menu_selected_idx: 0,
            log: Vec::new(),
            toasts: Vec::new(),
            view_mode: ViewMode::List,
            expanded_steps: HashSet::new(),
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            flow_zoom: 1.0,
            flow_pan: egui::Vec2::ZERO,
            run_progress_file: None,
            current_run_step: None,
            last_progress_check: std::time::Instant::now(),
            dirty: false,
            run_child: None,
            stderr_rx: None,
            run_tmp_file: None,
            run_from_offset: 0,
            prop_view: PropView::Form,
            selected_child: None,
            child_edit_buf: String::new(),
            child_parse_error: None,
            add_branch_target: None,
            confirm_dialog: None,
            form_edit_buffers: HashMap::new(),
            settings: load_settings(),
            settings_open: false,
            about_open: false,
            ai_panel_open: false,
            ai_messages: Vec::new(),
            ai_input: String::new(),
            ai_loading: false,
            ai_rx: None,
            ai_unread: false,
            md_cache: egui_commonmark::CommonMarkCache::default(),
            manual_open: false,
            manual_search: String::new(),
            scroll_to_selected: false,
            settings_test_result: None,
            settings_test_rx: None,
            scenario_vars: serde_yml::Mapping::new(),
            bottom_tab: BottomTab::default(),
            multi_selected: HashSet::new(),
            step_clipboard: Vec::new(),
            palette_force_open: None,
            canvas_positions: HashMap::new(),
            canvas_zoom: 1.0,
            canvas_pan: egui::Vec2::ZERO,
            canvas_dragging: None,
            undo_pushed_for_current_drag: false,
            canvas_layout_dirty: false,
            canvas_lasso: None,
            minimap_dragging: false,
            canvas_selection_anchor: None,
            canvas_viewport_size: egui::Vec2::new(800.0, 600.0),
            var_highlight: None,
            manual_category_filter: None,
            undo_limit_warned_at: None,
            ai_step_rx: None,
            ai_step_error: None,
            canvas_error_steps: HashMap::new(),
            canvas_completed_steps: HashSet::new(),
            canvas_edge_drag: None,
            canvas_lasso_additive: false,
            canvas_pending_insert_pos: None,
            canvas_search: String::new(),
            canvas_search_open: false,
            canvas_help_open: false,
            canvas_comments: Vec::new(),
            canvas_comment_drag: None,
            canvas_editing_comment: None,
            canvas_comment_next_id: 1,
            prev_problem_count: 0,
        }
    }
}

impl Drop for EditorApp {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.run_child {
            let _ = child.kill();
        }
    }
}

impl EditorApp {
    pub(crate) fn push_log(&mut self, message: impl Into<String>, level: LogLevel) {
        self.log.push(LogEntry {
            message: message.into(),
            level,
        });
        if self.log.len() > 500 {
            self.log.drain(0..100);
        }
    }
    pub(crate) fn push_toast(&mut self, message: String, level: LogLevel) {
        let expires = std::time::Instant::now() + std::time::Duration::from_secs(4);
        self.toasts.push(Toast {
            message,
            level,
            expires,
        });
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }
    pub(crate) fn log_ok(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        self.push_toast(s.clone(), LogLevel::Ok);
        self.push_log(s, LogLevel::Ok);
    }
    pub(crate) fn log_err(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        self.push_toast(s.clone(), LogLevel::Error);
        self.push_log(s, LogLevel::Error);
    }
    pub(crate) fn log_info(&mut self, msg: impl Into<String>) {
        self.push_log(msg, LogLevel::Info);
    }

    pub(crate) fn load_file_path(&mut self, p: &std::path::Path) {
        // SEC-6: reject unreasonably large files to prevent YAML alias amplification
        const MAX_YAML_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
        if let Ok(meta) = std::fs::metadata(p) {
            if meta.len() > MAX_YAML_BYTES {
                self.log_err(format!(
                    "ファイルが大きすぎます ({:.1} MB > 10 MB): {}",
                    meta.len() as f64 / 1_048_576.0,
                    p.display()
                ));
                return;
            }
        }
        match std::fs::read_to_string(p) {
            Ok(text) => match parse_scenario_steps(&text) {
                Ok((name, vars, steps)) => {
                    self.name = name;
                    self.scenario_vars = vars;
                    self.steps = steps;
                    self.selected = None;
                    self.edit_buf.clear();
                    self.path = Some(p.to_path_buf());
                    self.dirty = false;
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                    self.form_edit_buffers.clear();
                    self.canvas_positions.clear();
                    self.load_canvas_layout(p);
                }
                Err(e) => self.log_err(format!(
                    "構文エラー: {e} — YAML のインデントやコロンを確認してください"
                )),
            },
            Err(e) => self.log_err(format!(
                "読み込みエラー: {e} — ファイルが存在し読み取り可能か確認してください"
            )),
        }
    }

    pub(crate) fn commit_step(&mut self, idx: usize, mapping: serde_yml::Mapping) {
        self.steps[idx] = serde_yml::Value::Mapping(mapping);
        self.edit_buf = serde_yml::to_string(&self.steps[idx]).unwrap_or_default();
        self.dirty = true;
    }

    pub(crate) fn open_file(&mut self) {
        if self.dirty {
            self.confirm_dialog = Some(ConfirmAction::OpenFile);
            return;
        }
        self.do_open_file();
    }

    pub(crate) fn do_open_file(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file()
        {
            // SEC-6: size guard matches load_file_path
            const MAX_YAML_BYTES: u64 = 10 * 1024 * 1024;
            if let Ok(meta) = std::fs::metadata(&p) {
                if meta.len() > MAX_YAML_BYTES {
                    self.log_err(format!(
                        "ファイルが大きすぎます ({:.1} MB > 10 MB): {}",
                        meta.len() as f64 / 1_048_576.0,
                        p.display()
                    ));
                    return;
                }
            }
            match std::fs::read_to_string(&p) {
                Ok(text) => match parse_scenario_steps(&text) {
                    Ok((name, vars, steps)) => {
                        self.name = name;
                        self.scenario_vars = vars;
                        self.steps = steps;
                        self.selected = None;
                        self.edit_buf.clear();
                        self.path = Some(p.clone());
                        self.dirty = false;
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.form_edit_buffers.clear();
                        self.log_ok(format!("開きました: {}", p.display()));
                        self.canvas_positions.clear();
                        self.load_canvas_layout(&p);
                    }
                    Err(e) => self.log_err(format!("構文エラー: {e}")),
                },
                Err(e) => self.log_err(format!("読み込みエラー: {e}")),
            }
        }
    }

    pub(crate) fn write_scenario_to_path(&mut self, path: PathBuf) {
        match build_scenario_yaml(&self.name, &self.scenario_vars, &self.steps) {
            Ok(text) => match std::fs::write(&path, &text) {
                Ok(()) => {
                    self.path = Some(path.clone());
                    self.dirty = false;
                    self.log_ok(format!("保存しました: {}", path.display()));
                    self.save_canvas_layout();
                }
                Err(e) => self.log_err(format!(
                    "書き込みエラー: {e} — 保存先フォルダの書き込み権限を確認してください"
                )),
            },
            Err(e) => self.log_err(format!(
                "シリアライズエラー: {e} — ステップの値に不正な文字が含まれている可能性があります"
            )),
        }
    }

    pub(crate) fn save_file_as(&mut self) {
        self.flush_edit();
        if self.parse_error.is_some() {
            self.log_err("構文エラーがあるため保存できません。YAML を修正してください");
            return;
        }
        let Some(p) = rfd::FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .save_file()
        else {
            return;
        };
        self.write_scenario_to_path(p);
    }

    pub(crate) fn save_file(&mut self) {
        self.flush_edit();
        if self.parse_error.is_some() {
            self.log_err("構文エラーがあるため保存できません。YAML を修正してください");
            return;
        }
        let path = if let Some(ref p) = self.path {
            p.clone()
        } else if let Some(p) = rfd::FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .save_file()
        {
            p
        } else {
            return;
        };
        self.write_scenario_to_path(path);
    }

    pub(crate) fn flush_edit(&mut self) {
        if let Some(idx) = self.selected {
            match serde_yml::from_str::<serde_yml::Value>(&self.edit_buf) {
                Ok(v) => {
                    if idx < self.steps.len() && self.steps[idx] != v {
                        self.steps[idx] = v;
                        self.dirty = true;
                    }
                    self.parse_error = None;
                }
                Err(e) => {
                    self.parse_error = Some(e.to_string());
                }
            }
        }
    }

    pub(crate) fn select_step(&mut self, idx: usize) {
        // Only push undo when there is a pending edit that would actually change state.
        // Unconditional push_undo() here polluted the undo stack with every click.
        let has_pending_edit = self
            .selected
            .map(|sel| {
                sel < self.steps.len()
                    && self.parse_error.is_none()
                    && serde_yml::from_str::<serde_yml::Value>(&self.edit_buf)
                        .map(|v| self.steps[sel] != v)
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        if has_pending_edit {
            self.push_undo();
        }
        self.flush_edit();
        self.selected = Some(idx);
        self.multi_selected.clear();
        self.multi_selected.insert(idx);
        self.selected_child = None;
        self.child_edit_buf.clear();
        if let Some(step) = self.steps.get(idx) {
            self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
            self.parse_error = None;
        }
    }

    pub(crate) fn copy_selected_steps(&mut self) {
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_unstable();
        self.step_clipboard = indices
            .into_iter()
            .filter_map(|i| self.steps.get(i).cloned())
            .collect();
    }

    pub(crate) fn paste_steps(&mut self) {
        if self.step_clipboard.is_empty() {
            return;
        }
        self.push_undo();
        let at = self
            .multi_selected
            .iter()
            .max()
            .map(|i| i + 1)
            .unwrap_or(self.steps.len());
        let z = self.canvas_zoom;
        let vp = self.canvas_viewport_size;
        let pan = self.canvas_pan;
        for (j, step) in self.step_clipboard.iter().enumerate() {
            self.steps.insert(at + j, step.clone());
            Self::canvas_shift_positions(&mut self.canvas_positions, at + j, 1);
            Self::form_edit_buffers_shift(&mut self.form_edit_buffers, at + j, 1);
            let paste_pos = egui::pos2(
                vp.x / 2.0 / z - pan.x - NODE_W / 2.0 + j as f32 * 40.0,
                vp.y / 2.0 / z - pan.y - NODE_H / 2.0 + j as f32 * 40.0,
            );
            self.canvas_positions.insert(at + j, paste_pos);
        }
        // Select the newly pasted range.
        let end = at + self.step_clipboard.len() - 1;
        self.selected = Some(end);
        self.multi_selected = (at..=end).collect();
        if let Some(step) = self.steps.get(end) {
            self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
        }
        self.dirty = true;
    }

    pub(crate) fn delete_selected_steps(&mut self) {
        if self.multi_selected.is_empty() {
            return;
        }
        self.push_undo();
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_unstable_by(|a, b| b.cmp(a)); // delete from highest to keep indices valid
        for i in &indices {
            if *i < self.steps.len() {
                self.steps.remove(*i);
                Self::canvas_shift_positions(&mut self.canvas_positions, *i, -1);
                // BUG-4: keep form_edit_buffers in sync after bulk delete
                Self::form_edit_buffers_shift(&mut self.form_edit_buffers, *i, -1);
            }
        }
        self.multi_selected.clear();
        let new_sel = indices.last().and_then(|i| {
            let i = i.saturating_sub(1);
            if i < self.steps.len() {
                Some(i)
            } else {
                self.steps.len().checked_sub(1)
            }
        });
        self.selected = new_sel;
        if let Some(idx) = new_sel {
            self.multi_selected.insert(idx);
            self.edit_buf = self
                .steps
                .get(idx)
                .map(|s| serde_yml::to_string(s).unwrap_or_default())
                .unwrap_or_default();
        } else {
            self.edit_buf.clear();
        }
        self.form_edit_buffers.clear();
        self.dirty = true;
    }

    pub(crate) fn stop_run(&mut self) {
        if let Some(ref mut child) = self.run_child {
            let _ = child.kill();
            let _ = child.wait();
            self.log_info("実行を停止しました");
        }
        self.run_child = None;
        self.run_progress_file = None;
        self.current_run_step = None;
        // Clean up temp YAML files (BUG-2 / SEC-1)
        if let Some(ref p) = self.run_tmp_file {
            let _ = std::fs::remove_file(p);
        }
        self.run_tmp_file = None;
        self.run_from_offset = 0;
    }

    pub(crate) fn run_selection(&mut self) {
        if self.multi_selected.is_empty() {
            self.log_err("実行するステップが選択されていません — Ctrl+クリックで複数選択後に Ctrl+Shift+F5 を押してください");
            return;
        }
        self.flush_edit();
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_unstable();
        let selected_steps: Vec<serde_yml::Value> = indices
            .into_iter()
            .filter_map(|i| self.steps.get(i).cloned())
            .collect();
        let yaml =
            match build_scenario_yaml(&self.name, &serde_yml::Mapping::new(), &selected_steps) {
                Ok(y) => y,
                Err(e) => {
                    self.log_err(format!(
                        "YAML生成失敗: {e} — ステップの値を確認してください"
                    ));
                    return;
                }
            };
        let tmp_path = match write_tmp_yaml(&yaml) {
            Ok(p) => p,
            Err(e) => {
                self.log_err(format!("一時ファイル書き込み失敗: {e}"));
                return;
            }
        };
        let progress_file =
            std::env::temp_dir().join(format!("robost_progress_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&progress_file);
        self.run_progress_file = Some(progress_file.clone());
        self.run_from_offset = 0;
        self.current_run_step = None;
        self.canvas_error_steps.clear();
        self.canvas_completed_steps.clear();
        match spawn_rpa(&tmp_path, &progress_file) {
            Ok((child, rx)) => {
                self.run_tmp_file = Some(tmp_path);
                self.run_child = Some(child);
                self.stderr_rx = Some(rx);
                self.log_ok(format!(
                    "{}ステップを実行中 (選択範囲)",
                    self.multi_selected.len()
                ));
            }
            Err(e) => {
                self.run_progress_file = None;
                let _ = std::fs::remove_file(&tmp_path);
                self.log_err(format!("起動に失敗しました: {e}"));
            }
        }
    }

    pub(crate) fn run_scenario(&mut self) {
        self.stop_run();
        self.flush_edit();
        // Use a temp file for unsaved scenarios so users can test without being
        // Use a temp file for unsaved scenarios so first run never forces a save dialog.
        let (run_path, is_tmp) = if let Some(ref p) = self.path {
            (p.clone(), false)
        } else {
            let yaml = match build_scenario_yaml(&self.name, &self.scenario_vars, &self.steps) {
                Ok(y) => y,
                Err(e) => {
                    self.log_err(format!(
                        "YAML生成失敗: {e} — ステップの値を確認してください"
                    ));
                    return;
                }
            };
            match write_tmp_yaml(&yaml) {
                Ok(p) => (p, true),
                Err(e) => {
                    self.log_err(format!("一時ファイル書き込み失敗: {e}"));
                    return;
                }
            }
        };
        let progress_file =
            std::env::temp_dir().join(format!("robost_progress_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&progress_file);
        self.run_progress_file = Some(progress_file.clone());
        self.run_from_offset = 0;
        self.current_run_step = None;
        self.canvas_error_steps.clear();
        self.canvas_completed_steps.clear();
        match spawn_rpa(&run_path, &progress_file) {
            Ok((child, rx)) => {
                if is_tmp {
                    self.run_tmp_file = Some(run_path);
                }
                self.run_child = Some(child);
                self.stderr_rx = Some(rx);
                self.log_ok("rpa を起動しました");
            }
            Err(e) => {
                self.run_progress_file = None;
                if is_tmp {
                    let _ = std::fs::remove_file(&run_path);
                }
                let hint = if e.kind() == std::io::ErrorKind::NotFound {
                    "\n  ヒント: rpa バイナリがエディタと同じディレクトリにありません。\
                     cargo install robost-cli でインストールしてください。"
                } else {
                    ""
                };
                self.log_err(format!("起動に失敗しました: {e}{hint}"));
            }
        }
    }

    /// Run only steps[start_idx..] (BUG-1: sets `run_from_offset` so canvas
    /// progress highlighting maps rpa indices back to original step indices).
    pub(crate) fn run_from_step(&mut self, start_idx: usize) {
        self.stop_run();
        self.flush_edit();
        let steps_from: Vec<serde_yml::Value> = self.steps.get(start_idx..).unwrap_or(&[]).to_vec();
        if steps_from.is_empty() {
            self.log_err(format!(
                "ステップ {} 以降に実行するステップがありません",
                start_idx + 1
            ));
            return;
        }
        let yaml = match build_scenario_yaml(&self.name, &serde_yml::Mapping::new(), &steps_from) {
            Ok(y) => y,
            Err(e) => {
                self.log_err(format!("YAML生成失敗: {e}"));
                return;
            }
        };
        let tmp_path = match write_tmp_yaml(&yaml) {
            Ok(p) => p,
            Err(e) => {
                self.log_err(format!("一時ファイル書き込み失敗: {e}"));
                return;
            }
        };
        let progress_file =
            std::env::temp_dir().join(format!("robost_progress_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&progress_file);
        self.run_progress_file = Some(progress_file.clone());
        self.run_from_offset = start_idx; // BUG-1: rpa reports 0-based indices into the slice
        self.current_run_step = None;
        self.canvas_error_steps.clear();
        self.canvas_completed_steps.clear();
        match spawn_rpa(&tmp_path, &progress_file) {
            Ok((child, rx)) => {
                self.run_tmp_file = Some(tmp_path);
                self.run_child = Some(child);
                self.stderr_rx = Some(rx);
                self.log_ok(format!(
                    "ステップ {} 〜 {} を実行中",
                    start_idx + 1,
                    self.steps.len()
                ));
            }
            Err(e) => {
                self.run_progress_file = None;
                let _ = std::fs::remove_file(&tmp_path);
                self.log_err(format!("起動に失敗しました: {e}"));
            }
        }
    }

    pub(crate) fn build_flow_nodes(&self) -> Vec<FlowNode> {
        let mut nodes = Vec::new();
        collect_nodes(&self.steps, 0, &self.expanded_steps, &mut nodes);
        nodes
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Spawn the `rpa` binary with stderr piped.
/// Resolves the binary next to the current executable (never falls back to PATH
/// to avoid PATH-hijacking — SEC-5). Returns the child handle and a receiver
/// that delivers stderr lines as they arrive (REF-1).
fn spawn_rpa(
    yaml_path: &std::path::Path,
    progress_path: &std::path::Path,
) -> std::io::Result<(std::process::Child, std::sync::mpsc::Receiver<String>)> {
    let rpa_bin = std::env::current_exe()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?
        .parent()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "cannot determine rpa directory",
            )
        })?
        .join("rpa");

    let mut child = std::process::Command::new(&rpa_bin)
        .args([
            "run",
            &yaml_path.to_string_lossy(),
            "--progress",
            &progress_path.to_string_lossy(),
        ])
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let (tx, rx) = std::sync::mpsc::channel::<String>();
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            use std::io::BufRead;
            for line in std::io::BufReader::new(stderr)
                .lines()
                .map_while(Result::ok)
            {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
    }
    Ok((child, rx))
}

/// Write `yaml` to a secure temp file (mode 0o600, random name — SEC-1 / SEC-3).
/// Returns the path; caller must delete after use.
fn write_tmp_yaml(yaml: &str) -> std::io::Result<std::path::PathBuf> {
    use std::io::Write;
    let mut f = tempfile::Builder::new()
        .prefix("robost_")
        .suffix(".yaml")
        .tempfile()?;
    f.write_all(yaml.as_bytes())?;
    let (_, path) = f.keep().map_err(|e| e.error)?;
    Ok(path)
}

impl EditorApp {
    pub(crate) fn snapshot(&self) -> EditorState {
        EditorState {
            name: self.name.clone(),
            steps: self.steps.clone(),
            scenario_vars: self.scenario_vars.clone(),
            selected: self.selected,
            selected_child: self.selected_child.clone(),
            canvas_positions: self
                .canvas_positions
                .iter()
                .map(|(&k, &v)| (k, [v.x, v.y]))
                .collect(),
            canvas_zoom: self.canvas_zoom,
            canvas_pan: [self.canvas_pan.x, self.canvas_pan.y],
            multi_selected: self.multi_selected.iter().cloned().collect(),
            expanded_steps: self.expanded_steps.iter().cloned().collect(),
        }
    }

    pub(crate) fn restore(&mut self, state: EditorState) {
        self.name = state.name;
        self.steps = state.steps;
        self.scenario_vars = state.scenario_vars;
        self.selected = state.selected;
        self.selected_child = state.selected_child;
        self.canvas_positions = state
            .canvas_positions
            .into_iter()
            .map(|(k, [x, y])| (k, egui::pos2(x, y)))
            .collect();
        self.canvas_zoom = state.canvas_zoom;
        self.canvas_pan = egui::vec2(state.canvas_pan[0], state.canvas_pan[1]);
        self.multi_selected = state.multi_selected.into_iter().collect();
        self.expanded_steps = state.expanded_steps.into_iter().collect();
        self.child_edit_buf.clear();
        self.edit_buf = self
            .selected
            .and_then(|i| self.steps.get(i))
            .map(|s| serde_yml::to_string(s).unwrap_or_default())
            .unwrap_or_default();
        self.parse_error = None;
        self.form_edit_buffers.clear();
    }

    pub(crate) fn push_undo(&mut self) {
        self.push_undo_impl(false);
    }

    pub(crate) fn push_undo_forced(&mut self) {
        self.push_undo_impl(true);
    }

    pub(crate) fn push_undo_impl(&mut self, force: bool) {
        let snap = self.snapshot();
        let changed = force
            || self
                .undo_stack
                .back()
                .map(|s| s.steps != snap.steps || s.name != snap.name)
                .unwrap_or(true);
        if changed {
            self.undo_stack.push_back(snap);
            if self.undo_stack.len() > 50 {
                self.undo_stack.pop_front();
                let should_warn = self
                    .undo_limit_warned_at
                    .map(|t| t.elapsed() > std::time::Duration::from_secs(5))
                    .unwrap_or(true);
                if should_warn {
                    self.undo_limit_warned_at = Some(std::time::Instant::now());
                    self.push_toast(
                        "undo 履歴の上限 (50件) に達しました".to_owned(),
                        LogLevel::Info,
                    );
                }
            }
            self.redo_stack.clear();
        }
    }

    pub(crate) fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop_back() {
            let current = self.snapshot();
            self.redo_stack.push_back(current);
            self.restore(prev);
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop_back() {
            let current = self.snapshot();
            self.undo_stack.push_back(current);
            self.restore(next);
        }
    }

    pub(crate) fn get_branch_steps(
        &self,
        parent_idx: usize,
        branch_name: &str,
    ) -> Option<Vec<serde_yml::Value>> {
        let step = self.steps.get(parent_idx)?;
        let outer_map = step.as_mapping()?;
        let outer_key = outer_map.iter().next()?.0.as_str()?;
        if let Some(serde_yml::Value::Sequence(seq)) = outer_map.get(branch_name) {
            return Some(seq.clone());
        }
        if let Some(inner_map) = outer_map.get(outer_key).and_then(|v| v.as_mapping()) {
            if let Some(serde_yml::Value::Sequence(seq)) = inner_map.get(branch_name) {
                return Some(seq.clone());
            }
        }
        None
    }

    pub(crate) fn mutate_branch<F>(&mut self, parent_idx: usize, branch_name: &str, f: F)
    where
        F: FnOnce(&mut Vec<serde_yml::Value>),
    {
        let Some(step) = self.steps.get(parent_idx) else {
            return;
        };
        let Some(outer_map) = step.as_mapping() else {
            return;
        };
        let Some(outer_key) = outer_map.iter().next().and_then(|(k, _)| k.as_str()) else {
            return;
        };
        let outer_key = outer_key.to_owned();
        let mut new_outer = outer_map.clone();

        if let Some(serde_yml::Value::Sequence(seq)) = new_outer.get(branch_name).cloned() {
            let mut seq = seq;
            f(&mut seq);
            new_outer.insert(
                serde_yml::Value::String(branch_name.to_owned()),
                serde_yml::Value::Sequence(seq),
            );
            self.commit_step(parent_idx, new_outer);
            return;
        }

        if let Some(serde_yml::Value::Mapping(inner_map)) = new_outer.get(&outer_key).cloned() {
            let mut inner = inner_map;
            if let Some(serde_yml::Value::Sequence(seq)) = inner.get(branch_name).cloned() {
                let mut seq = seq;
                f(&mut seq);
                inner.insert(
                    serde_yml::Value::String(branch_name.to_owned()),
                    serde_yml::Value::Sequence(seq),
                );
                new_outer.insert(
                    serde_yml::Value::String(outer_key),
                    serde_yml::Value::Mapping(inner),
                );
                self.commit_step(parent_idx, new_outer);
            }
        }
    }

    pub(crate) fn set_branch_step(
        &mut self,
        parent_idx: usize,
        branch_name: &str,
        child_idx: usize,
        new_val: serde_yml::Value,
    ) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            if child_idx < seq.len() {
                seq[child_idx] = new_val;
            }
        });
    }

    pub(crate) fn swap_branch_steps(
        &mut self,
        parent_idx: usize,
        branch_name: &str,
        a: usize,
        b: usize,
    ) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            if a < seq.len() && b < seq.len() {
                seq.swap(a, b);
            }
        });
    }

    pub(crate) fn remove_branch_step(
        &mut self,
        parent_idx: usize,
        branch_name: &str,
        child_idx: usize,
    ) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            if child_idx < seq.len() {
                seq.remove(child_idx);
            }
        });
    }

    pub(crate) fn add_branch_step(
        &mut self,
        parent_idx: usize,
        branch_name: &str,
        new_step: serde_yml::Value,
    ) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            seq.push(new_step);
        });
    }

    /// Returns `true` if the step has `enabled: false`.
    pub(crate) fn step_is_disabled(step: &serde_yml::Value) -> bool {
        step.as_mapping()
            .and_then(|m| m.get("enabled"))
            .and_then(|v| v.as_bool())
            .map(|b| !b)
            .unwrap_or(false)
    }

    /// Toggle `enabled: false` on a step value. Does not push undo — callers must.
    pub(crate) fn toggle_step_enabled(step: &mut serde_yml::Value) {
        let Some(map) = step.as_mapping_mut() else {
            return;
        };
        let key = serde_yml::Value::String("enabled".to_owned());
        let is_disabled = map
            .get(&key)
            .and_then(|v| v.as_bool())
            .map(|b| !b)
            .unwrap_or(false);
        if is_disabled {
            map.remove(&key);
        } else {
            map.insert(key, serde_yml::Value::Bool(false));
        }
    }

    pub(crate) fn collect_var_refs_from_value<F: FnMut(&str)>(val: &serde_yml::Value, cb: &mut F) {
        match val {
            serde_yml::Value::String(s) => {
                let mut rest = s.as_str();
                while let Some(start) = rest.find("{{") {
                    rest = &rest[start + 2..];
                    if let Some(end) = rest.find("}}") {
                        let name = rest[..end].trim();
                        if !name.is_empty() {
                            cb(name);
                        }
                        rest = &rest[end + 2..];
                    } else {
                        break;
                    }
                }
            }
            serde_yml::Value::Sequence(seq) => {
                for v in seq {
                    Self::collect_var_refs_from_value(v, cb);
                }
            }
            serde_yml::Value::Mapping(map) => {
                for (k, v) in map {
                    Self::collect_var_refs_from_value(k, cb);
                    Self::collect_var_refs_from_value(v, cb);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn validate_scenario(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut defined: HashSet<String> = HashSet::new();
        // Pre-seed built-in loop variables
        defined.insert("item".to_owned());
        defined.insert("_index".to_owned());
        for (k, _) in &self.scenario_vars {
            if let Some(s) = k.as_str() {
                defined.insert(s.to_owned());
            }
        }

        const BRANCH_KEYS: &[&str] = &["do", "then", "else", "catch", "branches", "finally"];

        for (step_idx, step) in self.steps.iter().enumerate() {
            let Some(map) = step.as_mapping() else {
                continue;
            };

            // Register save_as before scanning this step's refs
            if let Some(serde_yml::Value::String(sv)) = map.get("save_as") {
                defined.insert(sv.clone());
            }

            // For foreach: register custom loop var from "as" field
            if let Some(inner) = map.get("foreach").and_then(|v| v.as_mapping()) {
                if let Some(serde_yml::Value::String(as_var)) = inner.get("as") {
                    defined.insert(as_var.clone());
                }
            }

            // Scan all keys except branch sub-keys and save_as
            for (k, v) in map {
                let k_str = k.as_str().unwrap_or("");
                if k_str == "save_as" || BRANCH_KEYS.contains(&k_str) {
                    continue;
                }
                let mut refs = Vec::new();
                Self::collect_var_refs_from_value(v, &mut |name| {
                    if !defined.contains(name) {
                        refs.push(name.to_owned());
                    }
                });
                refs.sort_unstable();
                refs.dedup();
                for name in refs {
                    issues.push(ValidationIssue {
                        step_idx,
                        message: format!("未定義の変数 '{{{{ {name} }}}}' を参照しています"),
                        level: LogLevel::Error,
                    });
                }
            }
        }
        issues
    }
}
