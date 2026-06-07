// ---- property panel, AI panel, settings, manual windows --------------------

use eframe::egui;
use std::collections::HashMap;

use crate::tokens;

use crate::ai_integration::{build_system_prompt, call_ai_api, test_ai_connection};
use crate::flow_helpers::{get_step_key, step_display_name, step_summary};
use crate::i18n::{Lang, S};
use crate::settings::{save_settings, AiProvider};
use crate::state::EditorApp;
use crate::step_templates::STEP_TEMPLATES;
use crate::types::{category_color, AiMessage, LogEntry, LogLevel, PropView};

impl EditorApp {
    pub(crate) fn show_property_form(&mut self, ui: &mut egui::Ui, idx: usize) {
        const FILE_KEYS: &[&str] = &[
            "template", "file", "path", "src", "dest", "local", "remote", "sub",
        ];
        const SECRET_KEYS: &[&str] = &["password", "pass", "secret", "token"];
        const MULTILINE_KEYS: &[&str] = &["script", "sql", "cmd", "body", "message", "query"];
        const PRESS_KEYS: &[&str] = &[
            "Enter",
            "Tab",
            "Escape",
            "Space",
            "Backspace",
            "Delete",
            "Up",
            "Down",
            "Left",
            "Right",
            "Home",
            "End",
            "PageUp",
            "PageDown",
            "F1",
            "F2",
            "F3",
            "F4",
            "F5",
            "F6",
            "F7",
            "F8",
            "F9",
            "F10",
            "F11",
            "F12",
            "Insert",
        ];
        const WINDOW_STATES: &[&str] = &["exists", "visible", "focused", "closed"];
        const WIN_CTRL_ACTIONS: &[&str] = &["focus", "maximize", "minimize", "close"];
        const SCROLL_DIRS: &[&str] = &["down", "up", "left", "right"];
        const UIA_STATES: &[&str] = &["exists", "enabled", "visible"];
        const ALERT_ACTIONS: &[&str] = &["accept", "dismiss", "get_text"];

        enum ChildAction {
            Select(String, usize),
            MoveUp(String, usize),
            MoveDown(String, usize),
            Delete(String, usize),
            Add(String),
        }

        let step = self.steps[idx].clone();
        let outer_key = get_step_key(&step);
        let outer_map = match step.as_mapping() {
            Some(m) => m.clone(),
            None => {
                ui.label("ステップ形式が不明です。YAML タブで編集してください。");
                return;
            }
        };
        let inner_val = outer_map.get(outer_key).cloned();

        // Collect sibling branch sequences (if: then/else at top level)
        let sibling_branches: Vec<(String, Vec<serde_yml::Value>)> = outer_map
            .iter()
            .filter_map(|(k, v)| {
                let ks = k.as_str()?;
                if ks == outer_key {
                    return None;
                }
                let seq = v.as_sequence()?;
                if seq.is_empty() || seq.iter().all(|s| s.is_mapping()) {
                    Some((ks.to_owned(), seq.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Pending actions resolved after closures
        let mut pending_file_key: Option<String> = None;
        let mut snip_clicked = false;
        let mut child_action: Option<ChildAction> = None;

        // ── ai_create: special prompt + generate button ─────────────────────
        if outer_key == "ai_create" {
            ui.label(
                egui::RichText::new("AI に指示を書くと、自動でステップを生成して置き換えます。")
                    .weak(),
            );
            ui.add_space(6.0);
            let buf_key = format!("ai_prompt@{idx}");
            let prompt_text = self
                .form_edit_buffers
                .entry(buf_key.clone())
                .or_insert_with(|| {
                    outer_map
                        .get("prompt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned()
                })
                .clone();
            let mut prompt_buf = prompt_text.clone();
            let te = egui::TextEdit::multiline(&mut prompt_buf)
                .hint_text("例: 「ログインボタンを押す」「パスワード欄に入力してEnterを押す」")
                .desired_rows(6)
                .desired_width(f32::INFINITY);
            if ui.add(te).changed() {
                self.form_edit_buffers.insert(buf_key, prompt_buf.clone());
                let escaped = prompt_buf
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r");
                self.edit_buf = format!("ai_create:\n  prompt: \"{escaped}\"\n");
                self.flush_edit();
                // Clear stale error so user gets immediate visual feedback that edit registered
                self.ai_step_error = None;
            }
            ui.add_space(8.0);
            let is_loading = self.ai_step_rx.as_ref().map(|(i, _)| *i) == Some(idx);
            let any_loading = self.ai_step_rx.is_some();
            if is_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("生成中...");
                });
            } else {
                let btn = egui::Button::new("⚡ AI で生成");
                if ui.add_enabled(!any_loading, btn).clicked() {
                    let current_prompt = self
                        .form_edit_buffers
                        .get(&format!("ai_prompt@{idx}"))
                        .cloned()
                        .unwrap_or_default();
                    if current_prompt.trim().is_empty() {
                        self.ai_step_error = Some((idx, "指示を入力してください。".into()));
                    } else if self.settings.api_key.trim().is_empty() {
                        self.ai_step_error = Some((
                            idx,
                            "APIキーが設定されていません。「設定」から入力してください。".into(),
                        ));
                    } else {
                        let (tx, rx) = std::sync::mpsc::channel::<anyhow::Result<String>>();
                        let settings = self.settings.clone();
                        let prompt_to_send = current_prompt.clone();
                        let system = build_system_prompt();
                        std::thread::spawn(move || {
                            let result = call_ai_api(&settings, &[], &prompt_to_send, &system);
                            let _ = tx.send(result);
                        });
                        self.ai_step_rx = Some((idx, rx));
                        self.ai_step_error = None;
                    }
                }
            }
            if let Some((err_idx, ref msg)) = self.ai_step_error.clone() {
                if err_idx == idx {
                    ui.add_space(4.0);
                    ui.colored_label(tokens::ERROR, format!("⚠ {msg}"));
                    if msg.contains("APIキー") && ui.small_button("設定を開く").clicked() {
                        self.settings_open = true;
                    }
                }
            }
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("prop_form_scroll")
            .show(ui, |ui| {
                // ── press: ComboBox ──────────────────────────────────────────
                if outer_key == "press" {
                    let current = match &inner_val {
                        Some(serde_yml::Value::String(s)) => s.clone(),
                        _ => String::new(),
                    };
                    let mut new_key = current.clone();
                    ui.horizontal(|ui| {
                        ui.monospace("press:");
                        egui::ComboBox::from_id_salt("press_combo")
                            .selected_text(&current)
                            .width(130.0)
                            .show_ui(ui, |ui| {
                                for k in PRESS_KEYS {
                                    ui.selectable_value(&mut new_key, k.to_string(), *k);
                                }
                            });
                        let mut free_buf = current.clone();
                        if ui.add(egui::TextEdit::singleline(&mut free_buf).desired_width(80.0)).changed() {
                            new_key = free_buf;
                        }
                    });
                    if new_key != current {
                        let mut rebuilt = outer_map.clone();
                        rebuilt.insert(
                            serde_yml::Value::String(outer_key.to_owned()),
                            serde_yml::Value::String(new_key),
                        );
                        self.push_undo();
                        self.commit_step(idx, rebuilt);
                    }
                    return;
                }

                match inner_val {
                    Some(serde_yml::Value::Mapping(m)) => {
                        let pairs: Vec<(String, serde_yml::Value)> = m
                            .iter()
                            .filter_map(|(k, v)| k.as_str().map(|s| (s.to_owned(), v.clone())))
                            .collect();

                        let scalar_pairs: Vec<&(String, serde_yml::Value)> = pairs
                            .iter()
                            .filter(|(_, v)| !matches!(v, serde_yml::Value::Sequence(_)))
                            .collect();
                        let inner_branches: Vec<(String, Vec<serde_yml::Value>)> = pairs
                            .iter()
                            .filter_map(|(k, v)| {
                                if let serde_yml::Value::Sequence(seq) = v {
                                    if seq.is_empty() || seq.iter().all(|s| s.is_mapping()) {
                                        return Some((k.clone(), seq.clone()));
                                    }
                                }
                                None
                            })
                            .collect();
                        // Sequences of scalars (e.g. keys: [ctrl, c]) — rendered as tag pills.
                        let scalar_seq_pairs: Vec<(String, Vec<serde_yml::Value>)> = pairs
                            .iter()
                            .filter_map(|(k, v)| {
                                if let serde_yml::Value::Sequence(seq) = v {
                                    if !seq.is_empty() && seq.iter().any(|s| !s.is_mapping()) {
                                        return Some((k.clone(), seq.clone()));
                                    }
                                }
                                None
                            })
                            .collect();

                        // Pre-populate persistent numeric-field edit buffers from stored state or model.
                        let mut field_bufs: HashMap<String, String> = scalar_pairs
                            .iter()
                            .filter_map(|(fk, fv)| {
                                if matches!(fv, serde_yml::Value::Number(_)) {
                                    let key = format!("{fk}@{idx}");
                                    let val = self.form_edit_buffers.get(&key).cloned()
                                        .unwrap_or_else(|| match fv {
                                            serde_yml::Value::Number(n) => n.to_string(),
                                            _ => unreachable!(),
                                        });
                                    Some((fk.clone(), val))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let mut new_m = m.clone();
                        let mut any_changed = false;
                        let mut num_errors: Vec<(String, String)> = Vec::new();
                        // Collect existing variable names for save_as autocomplete
                        let existing_var_names: Vec<String> = self.scenario_vars
                            .keys()
                            .filter_map(|k| k.as_str().map(|s| s.to_owned()))
                            .collect();

                        egui::Grid::new("prop_grid")
                            .num_columns(2)
                            .min_col_width(90.0)
                            .spacing([8.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (fk, fv) in &scalar_pairs {
                                    ui.label(egui::RichText::new(fk.as_str()).monospace().size(12.0));
                                    ui.horizontal(|ui| {
                                        let is_file = FILE_KEYS.contains(&fk.as_str());
                                        let is_secret = SECRET_KEYS.contains(&fk.as_str());
                                        let is_multi = MULTILINE_KEYS.contains(&fk.as_str());
                                        let is_state = fk.as_str() == "state" && outer_key == "wait_window";
                                        let combo_opts: Option<(&[&str], &str)> =
                                            if fk.as_str() == "state" && outer_key == "wait_window" {
                                                Some((WINDOW_STATES, "state_combo"))
                                            } else if fk.as_str() == "action" && outer_key == "window_control" {
                                                Some((WIN_CTRL_ACTIONS, "win_ctrl_action_combo"))
                                            } else if fk.as_str() == "direction" && outer_key == "mouse_scroll" {
                                                Some((SCROLL_DIRS, "scroll_dir_combo"))
                                            } else if fk.as_str() == "state" && outer_key == "uia_wait" {
                                                Some((UIA_STATES, "uia_state_combo"))
                                            } else if fk.as_str() == "action" && outer_key == "web_alert" {
                                                Some((ALERT_ACTIONS, "alert_action_combo"))
                                            } else {
                                                None
                                            };
                                        match fv {
                                            serde_yml::Value::String(s) => {
                                                if let Some((opts, salt)) = combo_opts {
                                                    let mut sel = s.clone();
                                                    egui::ComboBox::from_id_salt(salt)
                                                        .selected_text(&sel)
                                                        .width(110.0)
                                                        .show_ui(ui, |ui| {
                                                            for opt in opts {
                                                                ui.selectable_value(&mut sel, opt.to_string(), *opt);
                                                            }
                                                        });
                                                    if sel != *s {
                                                        new_m.insert(
                                                            serde_yml::Value::String(fk.clone()),
                                                            serde_yml::Value::String(sel),
                                                        );
                                                        any_changed = true;
                                                    }
                                                } else if is_state {
                                                    // legacy fallback (should be covered above)
                                                    let mut sel = s.clone();
                                                    egui::ComboBox::from_id_salt("state_combo")
                                                        .selected_text(&sel)
                                                        .width(110.0)
                                                        .show_ui(ui, |ui| {
                                                            for st in WINDOW_STATES {
                                                                ui.selectable_value(&mut sel, st.to_string(), *st);
                                                            }
                                                        });
                                                    if sel != *s {
                                                        new_m.insert(
                                                            serde_yml::Value::String(fk.clone()),
                                                            serde_yml::Value::String(sel),
                                                        );
                                                        any_changed = true;
                                                    }
                                                } else if is_multi || s.contains('\n') {
                                                    let mut buf = s.clone();
                                                    if ui.add(
                                                        egui::TextEdit::multiline(&mut buf)
                                                            .code_editor()
                                                            .desired_width(240.0)
                                                            .desired_rows(4),
                                                    ).changed() {
                                                        new_m.insert(
                                                            serde_yml::Value::String(fk.clone()),
                                                            serde_yml::Value::String(buf),
                                                        );
                                                        any_changed = true;
                                                    }
                                                } else if fk.as_str() == "save_as" {
                                                    // save_as: text field + existing variable picker
                                                    let mut buf = s.clone();
                                                    if ui.add(
                                                        egui::TextEdit::singleline(&mut buf)
                                                            .desired_width(160.0)
                                                            .hint_text("変数名"),
                                                    ).changed() {
                                                        new_m.insert(
                                                            serde_yml::Value::String(fk.clone()),
                                                            serde_yml::Value::String(buf.clone()),
                                                        );
                                                        any_changed = true;
                                                    }
                                                    if !existing_var_names.is_empty() {
                                                        let mut sel = buf.clone();
                                                        egui::ComboBox::from_id_salt(
                                                            format!("save_as_pick_{idx}"),
                                                        )
                                                        .selected_text("▾")
                                                        .width(36.0)
                                                        .show_ui(ui, |ui| {
                                                            ui.set_min_width(120.0);
                                                            for name in &existing_var_names {
                                                                ui.selectable_value(
                                                                    &mut sel,
                                                                    name.clone(),
                                                                    name.as_str(),
                                                                );
                                                            }
                                                        });
                                                        if sel != buf {
                                                            new_m.insert(
                                                                serde_yml::Value::String(fk.clone()),
                                                                serde_yml::Value::String(sel),
                                                            );
                                                            any_changed = true;
                                                        }
                                                    }
                                                } else {
                                                    let mut buf = s.clone();
                                                    let w = if is_file { 180.0 } else { 250.0 };
                                                    if ui.add(
                                                        egui::TextEdit::singleline(&mut buf)
                                                            .password(is_secret)
                                                            .desired_width(w),
                                                    ).changed() {
                                                        new_m.insert(
                                                            serde_yml::Value::String(fk.clone()),
                                                            serde_yml::Value::String(buf),
                                                        );
                                                        any_changed = true;
                                                    }
                                                    if fk == "template" {
                                                        use egui_phosphor::regular as ph;
                                                        if ui.small_button(ph::CAMERA)
                                                            .on_hover_text("Snip ツールを起動 — 対象UIを表示して Ctrl+Shift+C でキャプチャ後、📂 で PNG を選択")
                                                            .clicked()
                                                        {
                                                            snip_clicked = true;
                                                        }
                                                    }
                                                    if is_file && ui.small_button("📂").clicked() {
                                                        pending_file_key = Some(fk.clone());
                                                    }
                                                }
                                            }
                                            serde_yml::Value::Number(n) => {
                                                let current_model = n.to_string();
                                                let buf = field_bufs
                                                    .entry(fk.clone())
                                                    .or_insert_with(|| current_model.clone());
                                                let buf_valid = serde_yml::from_str::<serde_yml::Value>(buf.as_str()).is_ok();
                                                let text_color = if buf_valid {
                                                    ui.visuals().text_color()
                                                } else {
                                                    tokens::ERROR
                                                };
                                                let resp = ui.add(
                                                    egui::TextEdit::singleline(buf)
                                                        .desired_width(90.0)
                                                        .text_color(text_color),
                                                );
                                                if !buf_valid && resp.has_focus() {
                                                    ui.colored_label(tokens::ERROR, "⚠");
                                                }
                                                if resp.has_focus() {
                                                    if resp.changed() {
                                                        match serde_yml::from_str::<serde_yml::Value>(buf.as_str()) {
                                                            Ok(v) => {
                                                                new_m.insert(serde_yml::Value::String(fk.clone()), v);
                                                                any_changed = true;
                                                            }
                                                            Err(_) => {
                                                                num_errors.push((fk.clone(), format!("「{buf}」は数値ではありません")));
                                                            }
                                                        }
                                                    }
                                                } else if resp.lost_focus() {
                                                    if serde_yml::from_str::<serde_yml::Value>(buf.as_str()).is_err() {
                                                        *buf = current_model;
                                                    }
                                                } else {
                                                    // Not focused: keep buffer in sync with model
                                                    *buf = current_model;
                                                }
                                            }
                                            serde_yml::Value::Bool(b) => {
                                                let mut val = *b;
                                                if ui.checkbox(&mut val, "").changed() {
                                                    new_m.insert(
                                                        serde_yml::Value::String(fk.clone()),
                                                        serde_yml::Value::Bool(val),
                                                    );
                                                    any_changed = true;
                                                }
                                            }
                                            _ => {
                                                let s = serde_yml::to_string(fv).unwrap_or_default();
                                                let preview = s.trim();
                                                let short: String = preview.chars().take(48).collect();
                                                let short = short.as_str();
                                                ui.weak(egui::RichText::new(short).monospace().size(10.0))
                                                    .on_hover_text("複雑な値は YAML タブで編集してください");
                                                if ui.small_button("YAML タブで編集").clicked() {
                                                    self.prop_view = PropView::Yaml;
                                                }
                                            }
                                        }
                                    });
                                    ui.end_row();
                                }
                            });

                        for (fk, err) in &num_errors {
                            ui.colored_label(tokens::ERROR, format!("⚠ {fk}: {err}"));
                        }

                        // Sync persistent numeric buffers back to struct storage.
                        for (fk, buf) in &field_bufs {
                            self.form_edit_buffers.insert(format!("{fk}@{idx}"), buf.clone());
                        }

                        // ── Scalar-sequence tag-pill editor ──────────────────
                        for (fk, seq) in &scalar_seq_pairs {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(fk.as_str()).monospace().size(12.0),
                            );
                            let mut new_seq = seq.clone();
                            let mut seq_changed = false;
                            let mut delete_idx: Option<usize> = None;

                            ui.horizontal_wrapped(|ui| {
                                for (item_idx, item) in seq.iter().enumerate() {
                                    let label = match item {
                                        serde_yml::Value::String(s) => s.clone(),
                                        serde_yml::Value::Number(n) => n.to_string(),
                                        serde_yml::Value::Bool(b) => b.to_string(),
                                        _ => "?".to_owned(),
                                    };
                                    let pill = egui::Button::new(
                                        egui::RichText::new(format!("{label} ×")).size(11.0),
                                    )
                                    .fill(egui::Color32::from_gray(55))
                                    .min_size(egui::vec2(0.0, 20.0));
                                    if ui.add(pill).clicked() {
                                        delete_idx = Some(item_idx);
                                    }
                                    ui.add_space(2.0);
                                }
                            });

                            let add_key = format!("{fk}@{idx}@add");
                            let mut add_buf = self
                                .form_edit_buffers
                                .get(&add_key)
                                .cloned()
                                .unwrap_or_default();
                            let mut do_add = false;
                            ui.horizontal(|ui| {
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut add_buf)
                                        .desired_width(120.0)
                                        .hint_text("追加…"),
                                );
                                let enter = resp.has_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                if (enter || ui.small_button("+").clicked()) && !add_buf.is_empty()
                                {
                                    do_add = true;
                                }
                            });

                            if let Some(di) = delete_idx {
                                new_seq.remove(di);
                                seq_changed = true;
                            }
                            if do_add {
                                new_seq.push(serde_yml::Value::String(add_buf.clone()));
                                add_buf.clear();
                                seq_changed = true;
                            }
                            self.form_edit_buffers.insert(add_key, add_buf);

                            if seq_changed {
                                new_m.insert(
                                    serde_yml::Value::String(fk.clone()),
                                    serde_yml::Value::Sequence(new_seq),
                                );
                                any_changed = true;
                            }
                        }

                        if any_changed {
                            let mut rebuilt = outer_map.clone();
                            rebuilt.insert(
                                serde_yml::Value::String(outer_key.to_owned()),
                                serde_yml::Value::Mapping(new_m),
                            );
                            self.push_undo();
                            self.commit_step(idx, rebuilt);
                            self.parse_error = None;
                        }

                        // ── Branch sub-lists ─────────────────────────────────
                        let all_branches: Vec<(String, Vec<serde_yml::Value>)> = {
                            let mut v = inner_branches;
                            v.extend(sibling_branches.iter().cloned());
                            v
                        };

                        if !all_branches.is_empty() {
                            ui.add_space(8.0);
                            ui.separator();

                            for (branch_name, branch_steps) in &all_branches {
                                let hdr = egui::RichText::new(format!(
                                    "{}  ({} ステップ)", branch_name, branch_steps.len()
                                )).strong().size(12.0);
                                egui::CollapsingHeader::new(hdr)
                                    .default_open(true)
                                    .id_salt(format!("branch_{branch_name}"))
                                    .show(ui, |ui| {
                                        for (ci, child) in branch_steps.iter().enumerate() {
                                            let ck = get_step_key(child);
                                            let col = category_color(crate::types::step_key_category(ck));
                                            let summary = step_summary(child);
                                            let is_sel = self.selected_child
                                                == Some((branch_name.clone(), ci));
                                            ui.horizontal(|ui| {
                                                ui.colored_label(col, "▌");
                                                if ui.selectable_label(
                                                    is_sel,
                                                    format!("{ci}: {summary}"),
                                                ).clicked() {
                                                    child_action = Some(ChildAction::Select(branch_name.clone(), ci));
                                                }
                                                let branch_len = branch_steps.len();
                                                ui.add_enabled_ui(ci > 0, |ui| {
                                                    if ui.small_button("↑").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                        child_action = Some(ChildAction::MoveUp(branch_name.clone(), ci));
                                                    }
                                                });
                                                ui.add_enabled_ui(ci + 1 < branch_len, |ui| {
                                                    if ui.small_button("↓").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                        child_action = Some(ChildAction::MoveDown(branch_name.clone(), ci));
                                                    }
                                                });
                                                if ui.small_button("×").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                    child_action = Some(ChildAction::Delete(branch_name.clone(), ci));
                                                }
                                            });
                                        }
                                        if ui.small_button("+ ステップ追加…").clicked() {
                                            child_action = Some(ChildAction::Add(branch_name.clone()));
                                        }
                                    });
                            }

                            // Property form (or YAML fallback) for selected child step
                            if let Some((ref branch, child_idx)) = self.selected_child.clone() {
                                if let Some(branch_steps) = self.get_branch_steps(idx, branch) {
                                    if child_idx < branch_steps.len() {
                                        ui.separator();
                                        let ck = get_step_key(&branch_steps[child_idx]);
                                        let col = category_color(crate::types::step_key_category(ck));
                                        let mut deselect_child = false;
                                        ui.horizontal_wrapped(|ui| {
                                            ui.colored_label(egui::Color32::from_rgb(120, 120, 120), format!("ステップ {}", idx + 1));
                                            ui.colored_label(egui::Color32::from_rgb(120, 120, 120), "(");
                                            ui.colored_label(col, outer_key);
                                            ui.colored_label(egui::Color32::from_rgb(120, 120, 120), ")");
                                            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), "▶");
                                            ui.colored_label(tokens::ACCENT, branch.as_str());
                                            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), "▶");
                                            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), format!("[{}]", child_idx + 1));
                                            ui.colored_label(col, step_display_name(ck));
                                            if ui.small_button("×").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                deselect_child = true;
                                            }
                                        });
                                        // Show inline property form for leaf steps;
                                        // compound steps fall back to the YAML editor.
                                        let branch_clone = branch.clone();
                                        self.show_child_property_form(ui, idx, &branch_clone, child_idx);
                                        if deselect_child {
                                            self.selected_child = None;
                                            self.child_edit_buf.clear();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(scalar) => {
                        let mut buf = match &scalar {
                            serde_yml::Value::String(s) => s.clone(),
                            serde_yml::Value::Number(n) => n.to_string(),
                            serde_yml::Value::Bool(b) => b.to_string(),
                            serde_yml::Value::Null => String::new(),
                            _ => serde_yml::to_string(&scalar).unwrap_or_default(),
                        };
                        ui.horizontal(|ui| {
                            ui.monospace(outer_key);
                            ui.label(":");
                            if ui.text_edit_singleline(&mut buf).changed() {
                                let new_val = serde_yml::from_str::<serde_yml::Value>(&buf)
                                    .unwrap_or(serde_yml::Value::String(buf.clone()));
                                let mut rebuilt = outer_map.clone();
                                rebuilt.insert(
                                    serde_yml::Value::String(outer_key.to_owned()),
                                    new_val,
                                );
                                self.push_undo();
                                self.commit_step(idx, rebuilt);
                            }
                        });
                    }
                    None => {
                        ui.label("ステップ形式が不明です。YAML タブで編集してください。");
                    }
                }
            });

        // ── Deferred snip launch ─────────────────────────────────────────────
        if snip_clicked {
            self.log_info("📸 対象UIを表示して Ctrl+Shift+C でキャプチャ。採取後 📂 でPNGを選択してください。".to_string());
            crate::app::open_snip();
        }

        // ── Deferred file dialog ─────────────────────────────────────────────
        if let Some(fkey) = pending_file_key {
            if let Some(p) = rfd::FileDialog::new().pick_file() {
                let path_str = if let Some(ref scenario_path) = self.path {
                    if let Some(dir) = scenario_path.parent() {
                        p.strip_prefix(dir)
                            .map(|r| r.to_string_lossy().into_owned())
                            .unwrap_or_else(|_| p.to_string_lossy().into_owned())
                    } else {
                        p.to_string_lossy().into_owned()
                    }
                } else {
                    p.to_string_lossy().into_owned()
                };
                if let Some(step) = self.steps.get(idx) {
                    let step = step.clone();
                    let ok = get_step_key(&step);
                    if let Some(serde_yml::Value::Mapping(mut inner)) =
                        step.as_mapping().and_then(|m| m.get(ok)).cloned()
                    {
                        inner.insert(
                            serde_yml::Value::String(fkey),
                            serde_yml::Value::String(path_str),
                        );
                        let mut outer = step.as_mapping().unwrap().clone();
                        outer.insert(
                            serde_yml::Value::String(ok.to_owned()),
                            serde_yml::Value::Mapping(inner),
                        );
                        self.push_undo();
                        self.commit_step(idx, outer);
                    }
                }
            }
        }

        // ── Child actions ─────────────────────────────────────────────────────
        if let Some(act) = child_action {
            match act {
                ChildAction::Select(branch, ci) => {
                    if let Some(steps) = self.get_branch_steps(idx, &branch) {
                        if ci < steps.len() {
                            // REF-6: child_edit_buf is populated lazily in
                            // show_child_yaml_editor only when the YAML editor
                            // fallback is actually used (compound/unknown steps).
                            self.child_edit_buf.clear();
                            self.selected_child = Some((branch, ci));
                        }
                    }
                }
                ChildAction::MoveUp(branch, ci) => {
                    self.push_undo();
                    self.swap_branch_steps(idx, &branch, ci - 1, ci);
                    if self.selected_child == Some((branch.clone(), ci)) {
                        self.selected_child = Some((branch, ci - 1));
                    }
                }
                ChildAction::MoveDown(branch, ci) => {
                    self.push_undo();
                    self.swap_branch_steps(idx, &branch, ci, ci + 1);
                    if self.selected_child == Some((branch.clone(), ci)) {
                        self.selected_child = Some((branch, ci + 1));
                    }
                }
                ChildAction::Delete(branch, ci) => {
                    self.push_undo();
                    self.remove_branch_step(idx, &branch, ci);
                    if self.selected_child == Some((branch.clone(), ci)) {
                        self.selected_child = None;
                        self.child_edit_buf.clear();
                    } else if let Some((ref b, ref mut sel_ci)) = self.selected_child {
                        if *b == branch && *sel_ci > ci {
                            *sel_ci -= 1;
                        }
                    }
                }
                ChildAction::Add(branch) => {
                    // Open the step picker targeting this branch
                    self.add_branch_target = Some((idx, branch));
                    self.add_menu_open = true;
                    self.add_menu_just_opened = true;
                    self.add_filter.clear();
                }
            }
        }
    }

    /// Inline property form for a child step inside a branch.
    /// Shows a field grid for leaf steps; falls back to the YAML textarea for
    /// compound steps (if/foreach/try_catch/…) which have their own branches.
    pub(crate) fn show_child_property_form(
        &mut self,
        ui: &mut egui::Ui,
        parent_idx: usize,
        branch: &str,
        child_idx: usize,
    ) {
        const FILE_KEYS: &[&str] = &[
            "template", "file", "path", "src", "dest", "local", "remote", "sub",
        ];
        const COMPOUND_KEYS: &[&str] = &[
            "if",
            "foreach",
            "repeat",
            "while",
            "do_while",
            "try_catch",
            "group",
            "switch",
        ];

        let child_steps = match self.get_branch_steps(parent_idx, branch) {
            Some(s) => s,
            None => return,
        };
        let child = match child_steps.get(child_idx) {
            Some(c) => c.clone(),
            None => return,
        };

        let outer_map = match child.as_mapping() {
            Some(m) => m.clone(),
            None => {
                // Non-mapping value — show YAML fallback
                self.show_child_yaml_editor(ui, parent_idx, branch, child_idx);
                return;
            }
        };
        let outer_key = match outer_map.iter().next().and_then(|(k, _)| k.as_str()) {
            Some(k) => k,
            None => {
                self.show_child_yaml_editor(ui, parent_idx, branch, child_idx);
                return;
            }
        };

        // Compound steps keep the YAML editor (their branches need separate handling)
        if COMPOUND_KEYS.contains(&outer_key) {
            self.show_child_yaml_editor(ui, parent_idx, branch, child_idx);
            return;
        }

        let inner_val = outer_map.get(outer_key).cloned();
        let mut new_outer = outer_map.clone();
        let mut changed = false;
        let mut pending_file_key: Option<String> = None;
        let mut snip_clicked = false;

        match inner_val {
            Some(serde_yml::Value::Mapping(inner_map)) => {
                let pairs: Vec<(String, serde_yml::Value)> = inner_map
                    .iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_owned(), v.clone())))
                    .collect();
                let mut new_inner = inner_map.clone();

                egui::Grid::new(egui::Id::new(("child_form", parent_idx, child_idx)))
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (fk, fv) in &pairs {
                            let is_file = FILE_KEYS.contains(&fk.as_str());
                            ui.monospace(fk);
                            ui.horizontal(|ui| match fv {
                                serde_yml::Value::String(sv) => {
                                    let mut buf = sv.clone();
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut buf).desired_width(200.0),
                                    );
                                    if fk == "template" {
                                        use egui_phosphor::regular as ph;
                                        if ui.small_button(ph::CAMERA)
                                            .on_hover_text("Snip ツールを起動 — 対象UIを表示して Ctrl+Shift+C でキャプチャ後、📂 で PNG を選択")
                                            .clicked()
                                        {
                                            snip_clicked = true;
                                        }
                                    }
                                    if is_file && ui.small_button("📂").clicked() {
                                        pending_file_key = Some(fk.clone());
                                    }
                                    if resp.changed() {
                                        new_inner.insert(
                                            serde_yml::Value::String(fk.clone()),
                                            serde_yml::Value::String(buf),
                                        );
                                        changed = true;
                                    }
                                }
                                serde_yml::Value::Number(n) => {
                                    let mut buf = n.to_string();
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut buf)
                                                .desired_width(100.0),
                                        )
                                        .changed()
                                    {
                                        if let Ok(nv) =
                                            serde_yml::from_str::<serde_yml::Value>(&buf)
                                        {
                                            new_inner
                                                .insert(serde_yml::Value::String(fk.clone()), nv);
                                            changed = true;
                                        }
                                    }
                                }
                                serde_yml::Value::Bool(b) => {
                                    let mut val = *b;
                                    if ui.checkbox(&mut val, "").changed() {
                                        new_inner.insert(
                                            serde_yml::Value::String(fk.clone()),
                                            serde_yml::Value::Bool(val),
                                        );
                                        changed = true;
                                    }
                                }
                                _ => {
                                    let preview: String = serde_yml::to_string(fv)
                                        .unwrap_or_default()
                                        .trim()
                                        .chars()
                                        .take(48)
                                        .collect();
                                    ui.weak(egui::RichText::new(preview).monospace().size(10.0))
                                        .on_hover_text("複雑な値は YAML タブで編集してください");
                                }
                            });
                            ui.end_row();
                        }
                    });

                // Deferred snip launch (must run outside the grid closure)
                if snip_clicked {
                    self.log_info("📸 対象UIを表示して Ctrl+Shift+C でキャプチャ。採取後 📂 でPNGを選択してください。".to_string());
                    crate::app::open_snip();
                }

                // Deferred file picker (must run outside the grid closure)
                if let Some(fk) = pending_file_key {
                    if let Some(p) = rfd::FileDialog::new().pick_file() {
                        let path_str = if let Some(ref scenario_path) = self.path {
                            if let Some(dir) = scenario_path.parent() {
                                p.strip_prefix(dir)
                                    .map(|r| r.to_string_lossy().into_owned())
                                    .unwrap_or_else(|_| p.to_string_lossy().into_owned())
                            } else {
                                p.to_string_lossy().into_owned()
                            }
                        } else {
                            p.to_string_lossy().into_owned()
                        };
                        new_inner.insert(
                            serde_yml::Value::String(fk),
                            serde_yml::Value::String(path_str),
                        );
                        changed = true;
                    }
                }

                if changed {
                    new_outer.insert(
                        serde_yml::Value::String(outer_key.to_owned()),
                        serde_yml::Value::Mapping(new_inner),
                    );
                    let branch_owned = branch.to_owned();
                    self.push_undo();
                    self.set_branch_step(
                        parent_idx,
                        &branch_owned,
                        child_idx,
                        serde_yml::Value::Mapping(new_outer),
                    );
                }
            }

            Some(serde_yml::Value::String(sv)) => {
                // e.g.  type: "hello world"
                let mut buf = sv.clone();
                if ui
                    .add(egui::TextEdit::singleline(&mut buf).desired_width(280.0))
                    .changed()
                {
                    new_outer.insert(
                        serde_yml::Value::String(outer_key.to_owned()),
                        serde_yml::Value::String(buf),
                    );
                    let branch_owned = branch.to_owned();
                    self.push_undo();
                    self.set_branch_step(
                        parent_idx,
                        &branch_owned,
                        child_idx,
                        serde_yml::Value::Mapping(new_outer),
                    );
                }
            }

            Some(serde_yml::Value::Number(n)) => {
                // e.g.  wait_ms: 1000
                let mut buf = n.to_string();
                if ui
                    .add(egui::TextEdit::singleline(&mut buf).desired_width(120.0))
                    .changed()
                {
                    if let Ok(nv) = serde_yml::from_str::<serde_yml::Value>(&buf) {
                        new_outer.insert(serde_yml::Value::String(outer_key.to_owned()), nv);
                        let branch_owned = branch.to_owned();
                        self.push_undo();
                        self.set_branch_step(
                            parent_idx,
                            &branch_owned,
                            child_idx,
                            serde_yml::Value::Mapping(new_outer),
                        );
                    }
                }
            }

            _ => {
                // Unknown structure — YAML fallback
                self.show_child_yaml_editor(ui, parent_idx, branch, child_idx);
            }
        }
    }

    /// Raw YAML textarea for child steps (fallback for compound/unknown steps).
    fn show_child_yaml_editor(
        &mut self,
        ui: &mut egui::Ui,
        parent_idx: usize,
        branch: &str,
        child_idx: usize,
    ) {
        // REF-6: lazily sync buffer with current child state so form edits that
        // preceded this YAML view are reflected correctly.
        if let Some(steps) = self.get_branch_steps(parent_idx, branch) {
            if let Some(child) = steps.get(child_idx) {
                let current_yaml = serde_yml::to_string(child).unwrap_or_default();
                if self.child_edit_buf.trim() != current_yaml.trim() {
                    self.child_edit_buf = current_yaml;
                    self.child_parse_error = None;
                }
            }
        }
        let resp = ui.add(
            egui::TextEdit::multiline(&mut self.child_edit_buf)
                .code_editor()
                .desired_rows(5)
                .desired_width(f32::INFINITY),
        );
        if resp.changed() {
            match serde_yml::from_str::<serde_yml::Value>(&self.child_edit_buf) {
                Ok(new_child) => {
                    self.child_parse_error = None;
                    let branch_owned = branch.to_owned();
                    self.push_undo();
                    self.set_branch_step(parent_idx, &branch_owned, child_idx, new_child);
                }
                Err(e) => {
                    self.child_parse_error = Some(e.to_string());
                }
            }
        }
        if let Some(ref err) = self.child_parse_error.clone() {
            ui.colored_label(tokens::ERROR, format!("YAML エラー: {err}"));
        }
    }

    pub(crate) fn insert_yaml_snippet(&mut self, yaml: &str) {
        match serde_yml::from_str::<serde_yml::Value>(yaml) {
            Ok(val) => {
                self.push_undo();
                let at = self.selected.map(|i| i + 1).unwrap_or(self.steps.len());
                let count = match &val {
                    serde_yml::Value::Sequence(seq) => seq.len(),
                    _ => 1,
                };
                match val {
                    serde_yml::Value::Sequence(seq) => {
                        for (j, s) in seq.into_iter().enumerate() {
                            self.steps.insert(at + j, s);
                            Self::canvas_shift_positions(&mut self.canvas_positions, at + j, 1);
                            Self::form_edit_buffers_shift(&mut self.form_edit_buffers, at + j, 1);
                        }
                    }
                    other => {
                        self.steps.insert(at, other);
                        Self::canvas_shift_positions(&mut self.canvas_positions, at, 1);
                        Self::form_edit_buffers_shift(&mut self.form_edit_buffers, at, 1);
                    }
                }
                self.dirty = true;
                self.ensure_canvas_layout();
                self.log.push(LogEntry {
                    message: match self.settings.lang {
                        Lang::Ja => format!("✅ {}ステップを{}番目の後に挿入しました", count, at),
                        Lang::En => format!("✅ Inserted {count} step(s) after position {at}"),
                        Lang::Zh => format!("✅ 已在第 {at} 个之后插入 {count} 个步骤"),
                    },
                    level: LogLevel::Ok,
                });
            }
            Err(e) => {
                self.log.push(LogEntry {
                    message: match self.settings.lang {
                        Lang::Ja => format!("⚠ YAML解析エラー (挿入失敗): {e}"),
                        Lang::En => format!("⚠ YAML parse error; insert failed: {e}"),
                        Lang::Zh => format!("⚠ YAML 解析错误，插入失败: {e}"),
                    },
                    level: LogLevel::Error,
                });
            }
        }
    }

    pub(crate) fn send_ai_request(&mut self) {
        if self.ai_loading || self.ai_input.trim().is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        self.ai_rx = Some(rx);
        self.ai_loading = true;
        let settings = self.settings.clone();
        let history = self.ai_messages.clone();
        let input = self.ai_input.clone();
        let input_for_thread = input.clone();
        let system = build_system_prompt();
        std::thread::spawn(move || {
            let result = call_ai_api(&settings, &history, &input_for_thread, &system);
            let _ = tx.send(result.unwrap_or_else(|e| format!("⚠ API error: {e}")));
        });
        self.ai_messages.push(AiMessage {
            role: "user".into(),
            content: input,
            yaml_blocks: vec![],
        });
        self.ai_input.clear();
    }

    pub(crate) fn show_ai_panel(&mut self, ctx: &egui::Context) {
        // Floating button at bottom-right corner
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("ai_fab"))
            .fixed_pos(egui::pos2(screen.max.x - 60.0, screen.max.y - 60.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let fill = if self.ai_unread {
                    tokens::WARNING
                } else {
                    tokens::ACCENT
                };
                let label = egui::RichText::new("💬")
                    .color(egui::Color32::WHITE)
                    .size(22.0);
                if ui
                    .add(
                        egui::Button::new(label)
                            .min_size(egui::vec2(48.0, 48.0))
                            .corner_radius(egui::CornerRadius::same(24))
                            .fill(fill),
                    )
                    .on_hover_text(match self.settings.lang {
                        Lang::Ja => "AI アシスタント",
                        Lang::En => "AI assistant",
                        Lang::Zh => "AI 助手",
                    })
                    .clicked()
                {
                    self.ai_panel_open = !self.ai_panel_open;
                    if self.ai_panel_open {
                        self.ai_unread = false;
                    }
                }
            });

        if !self.ai_panel_open {
            return;
        }
        self.ai_unread = false;

        let default_pos = egui::pos2(screen.max.x - 390.0, screen.max.y - 360.0);
        let panel_size = egui::vec2(360.0, 320.0);
        let ai_title = match self.settings.lang {
            Lang::Ja => "AI アシスタント",
            Lang::En => "AI assistant",
            Lang::Zh => "AI 助手",
        };
        egui::Window::new(ai_title)
            .default_pos(default_pos)
            .fixed_size(panel_size)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_width(panel_size.x);
                ui.horizontal(|ui| {
                    ui.strong(ai_title);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("×")
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.ai_panel_open = false;
                        }
                        let clear_label = match self.settings.lang {
                            Lang::Ja => "クリア",
                            Lang::En => "Clear",
                            Lang::Zh => "清除",
                        };
                        if ui
                            .small_button(clear_label)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.ai_messages.clear();
                            self.md_cache = egui_commonmark::CommonMarkCache::default();
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("ai_scroll")
                    .max_height(ui.available_height() - 80.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let messages = self.ai_messages.clone();
                        for (i, msg) in messages.iter().enumerate() {
                            if msg.role == "user" {
                                ui.horizontal(|ui| {
                                    ui.strong(match self.settings.lang {
                                        Lang::Ja => "あなた:",
                                        Lang::En => "You:",
                                        Lang::Zh => "你:",
                                    });
                                    ui.label(&msg.content);
                                });
                            } else {
                                ui.strong("AI:");
                                egui_commonmark::CommonMarkViewer::new()
                                    .show(ui, &mut self.md_cache, &msg.content);
                                for (bi, yaml) in msg.yaml_blocks.iter().enumerate() {
                                    let yaml_clone = yaml.clone();
                                    let preview_text = match serde_yml::from_str::<serde_yml::Value>(&yaml_clone) {
                                        Ok(serde_yml::Value::Sequence(ref seq)) => {
                                            let names: Vec<&str> = seq.iter().map(get_step_key).take(3).collect();
                                            let suffix = if seq.len() > 3 {
                                                match self.settings.lang {
                                                    Lang::Ja => format!(" … ({}件)", seq.len()),
                                                    Lang::En => format!(" ... ({} steps)", seq.len()),
                                                    Lang::Zh => format!(" … ({} 个)", seq.len()),
                                                }
                                            } else {
                                                String::new()
                                            };
                                            match self.settings.lang {
                                                Lang::Ja => format!("{}件: {}{}", seq.len(), names.join(", "), suffix),
                                                Lang::En => format!("{} step(s): {}{}", seq.len(), names.join(", "), suffix),
                                                Lang::Zh => format!("{} 个: {}{}", seq.len(), names.join(", "), suffix),
                                            }
                                        }
                                        Ok(ref v) => match self.settings.lang {
                                            Lang::Ja => format!("1件: {}", get_step_key(v)),
                                            Lang::En => format!("1 step: {}", get_step_key(v)),
                                            Lang::Zh => format!("1 个: {}", get_step_key(v)),
                                        },
                                        Err(_) => match self.settings.lang {
                                            Lang::Ja => "(プレビュー不可)".to_owned(),
                                            Lang::En => "(preview unavailable)".to_owned(),
                                            Lang::Zh => "(无法预览)".to_owned(),
                                        },
                                    };
                                    if ui
                                        .button(match self.settings.lang {
                                            Lang::Ja => format!("📋 挿入 #{}", bi + 1),
                                            Lang::En => format!("📋 Insert #{}", bi + 1),
                                            Lang::Zh => format!("📋 插入 #{}", bi + 1),
                                        })
                                        .on_hover_text(preview_text)
                                        .clicked()
                                    {
                                        self.insert_yaml_snippet(&yaml_clone);
                                    }
                                }
                                let _ = i; // suppress unused variable warning
                            }
                            ui.separator();
                        }
                        if self.ai_loading {
                            ui.spinner();
                            ui.label(match self.settings.lang {
                                Lang::Ja => "処理中…",
                                Lang::En => "Working...",
                                Lang::Zh => "处理中…",
                            });
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    let input_w = (ui.available_width() - 64.0).max(180.0);
                    let resp = ui.add_sized(
                        [input_w, 44.0],
                        egui::TextEdit::multiline(&mut self.ai_input)
                            .desired_rows(2)
                            .hint_text(match self.settings.lang {
                                Lang::Ja => {
                                    "例: \"Excel のループを作って\"\n\"画像クリックのステップを追加して\""
                                }
                                Lang::En => {
                                    "e.g. \"Create an Excel loop\"\n\"Add an image-click step\""
                                }
                                Lang::Zh => "例: \"创建 Excel 循环\"\n\"添加图片点击步骤\"",
                            }),
                    );
                    let send_label = match self.settings.lang {
                        Lang::Ja => "送信",
                        Lang::En => "Send",
                        Lang::Zh => "发送",
                    };
                    let send = ui.add_enabled(
                        !self.ai_loading,
                        egui::Button::new(send_label).min_size(egui::vec2(50.0, 44.0)),
                    );
                    if send.clicked()
                        || (resp.has_focus()
                            && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                            && ctx.input(|i| i.modifiers.ctrl))
                    {
                        self.send_ai_request();
                    }
                });
                ui.weak(match self.settings.lang {
                    Lang::Ja => "Ctrl+Enter で送信",
                    Lang::En => "Ctrl+Enter to send",
                    Lang::Zh => "Ctrl+Enter 发送",
                });
            });
    }

    pub(crate) fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let s = S::for_lang(&self.settings.lang);
        let mut open = self.settings_open;
        egui::Window::new(s.settings_title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_size([380.0, 240.0])
            .show(ctx, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(format!("{}:", s.settings_provider));
                        egui::ComboBox::from_id_salt("provider_combo")
                            .selected_text(match self.settings.provider {
                                AiProvider::Anthropic => "Anthropic (Claude)",
                                AiProvider::OpenAI => "OpenAI",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.settings.provider,
                                    AiProvider::Anthropic,
                                    "Anthropic (Claude)",
                                );
                                ui.selectable_value(
                                    &mut self.settings.provider,
                                    AiProvider::OpenAI,
                                    "OpenAI",
                                );
                            });
                        ui.end_row();

                        ui.label(format!("{}:", s.settings_api_key));
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.api_key)
                                    .password(true)
                                    .desired_width(240.0),
                            )
                            .changed()
                        {
                            save_settings(&self.settings);
                        }
                        ui.end_row();

                        ui.label(format!("{}:", s.settings_model));
                        // Preset list for quick selection
                        let presets: &[&str] = match self.settings.provider {
                            AiProvider::Anthropic => &[
                                "claude-haiku-4-5-20251001",
                                "claude-sonnet-4-6",
                                "claude-opus-4-8",
                            ],
                            AiProvider::OpenAI => &["gpt-4o-mini", "gpt-4o", "gpt-4o-latest"],
                        };
                        ui.vertical(|ui| {
                            // Free-form text field — accepts any model ID including future ones
                            ui.text_edit_singleline(&mut self.settings.model);
                            // Preset buttons as quick-fill
                            ui.horizontal_wrapped(|ui| {
                                ui.weak("プリセット:");
                                for &p in presets {
                                    if ui.small_button(p).clicked() {
                                        self.settings.model = p.to_string();
                                    }
                                }
                            });
                        });
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(s.settings_save).clicked() {
                        save_settings(&self.settings);
                        self.settings_open = false;
                    }
                    let testing = self.settings_test_rx.is_some();
                    if ui
                        .add_enabled(
                            !testing && !self.settings.api_key.is_empty(),
                            egui::Button::new(s.settings_test),
                        )
                        .clicked()
                    {
                        self.settings_test_result = None;
                        let (tx, rx) = std::sync::mpsc::channel::<(bool, String)>();
                        self.settings_test_rx = Some(rx);
                        let settings = self.settings.clone();
                        std::thread::spawn(move || {
                            let result = test_ai_connection(&settings);
                            let _ = tx.send(result);
                        });
                    }
                    if testing {
                        ui.spinner();
                        ui.weak("テスト中…");
                    } else if let Some((ok, ref msg)) = self.settings_test_result {
                        if ok {
                            ui.colored_label(tokens::SUCCESS, format!("✓ {msg}"));
                        } else {
                            ui.colored_label(tokens::ERROR, format!("× {msg}"));
                        }
                    }
                });
            });
        // Flush settings when the window is closed by clicking the X button.
        if self.settings_open && !open {
            save_settings(&self.settings);
        }
        self.settings_open = open;
    }

    pub(crate) fn show_manual_window(&mut self, ctx: &egui::Context) {
        if !self.manual_open {
            return;
        }
        let mut open = self.manual_open;
        let mut insert_yaml: Option<&'static str> = None;

        egui::Window::new("マニュアル")
            .open(&mut open)
            .resizable(true)
            .default_size([720.0, 640.0])
            .show(ctx, |ui| {
                ui.heading("robost エディタ マニュアル");
                ui.weak("シナリオ作成、テンプレート採取、実行確認、ステップ仕様をまとめた操作ガイドです。");
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_salt("manual_body")
                    .show(ui, |ui| {
                        ui.collapsing("基本フロー", |ui| {
                            ui.label("1. 左のステップパレットからステップをダブルクリック、またはキャンバスへドラッグして追加します。");
                            ui.label("2. ステップを選択し、右のインスペクタまたはリスト編集画面でパラメータを入力します。");
                            ui.label("3. 画像操作ステップでは Snip で PNG テンプレートを採取し、template/path 欄へ設定します。");
                            ui.label("4. 下部の Problems タブで未設定項目や構文エラーを確認します。");
                            ui.label("5. F5 または実行ボタンでシナリオを実行します。ログとキャンバス上の完了/失敗表示で結果を確認します。");
                        });

                        ui.collapsing("画面構成", |ui| {
                            ui.label("Activity Bar: 左端のアイコン列です。シナリオ内ステップ、追加ノード、テンプレート画像を切り替えます。");
                            ui.label("Sidebar: ステップ一覧、ノードパレット、テンプレートギャラリーを表示します。");
                            ui.label("Editor surface: List / Flow / Canvas の作業面です。Canvas はノードを自由配置できます。");
                            ui.label("Inspector: Canvas / Flow で選択中ステップを編集する右側パネルです。フォームと YAML を切り替えられます。");
                            ui.label("Bottom panel: 変数、ログ、問題一覧を表示します。Problems は新しい問題が出ると自動で開きます。");
                        });

                        ui.collapsing("キャンバス操作", |ui| {
                            ui.label("ノードをドラッグすると配置を変更できます。複数選択中は右クリックメニューから整列や等間隔配置を実行できます。");
                            ui.label("Shift+ドラッグで範囲選択、Cmd+A で全選択、Esc で選択解除します。");
                            ui.label("Cmd+F でノード検索、Cmd+0 で全体表示、Cmd+1 で 100% 表示に戻します。");
                            ui.label("ノード下部の接続点からドラッグすると、ステップ順を移動できます。");
                        });

                        ui.collapsing("テンプレート採取", |ui| {
                            ui.label("画像操作ステップの template 欄ではカメラボタンから Snip ツールを起動できます。");
                            ui.label("採取した PNG はシナリオファイルと同じディレクトリ、または Documents/robost_templates からテンプレートギャラリーに表示されます。");
                            ui.label("テンプレートギャラリーの画像をダブルクリックすると、選択中の画像操作ステップへパスを設定します。");
                        });

                        ui.collapsing("実行と検証", |ui| {
                            ui.label("F5 は実行/停止の切り替えです。Cmd+R は停止中のみ実行します。");
                            ui.label("複数ステップ選択中は選択範囲だけを実行できます。Canvas の右クリックでは選択ステップ以降の実行もできます。");
                            ui.label("ログは下部 Log タブに出ます。実行中のステップ、完了、失敗はキャンバスにも反映されます。");
                            ui.label("Problems タブは YAML 構文、必須フィールド、テンプレート設定漏れなどの確認用です。");
                        });

                        ui.collapsing("YAML 編集", |ui| {
                            ui.label("フォームで扱いにくい複雑な値は YAML タブで直接編集できます。");
                            ui.label("構文エラーがある場合はインスペクタ上部に表示され、正しい YAML になるまでシナリオへ反映されません。");
                            ui.label("シナリオ変数は YAML の {{ name }} 形式で参照できます。Variables タブで変数をクリックすると参照ステップを強調表示します。");
                        });

                        ui.separator();
                        ui.heading("ステップリファレンス");
                        ui.horizontal(|ui| {
                            ui.label("検索:");
                            ui.text_edit_singleline(&mut self.manual_search);
                            if ui.small_button("クリア").clicked() {
                                self.manual_search.clear();
                            }
                        });

                        // Category chips — toggles the category filter independently of the text search.
                        ui.horizontal_wrapped(|ui| {
                            let mut seen = std::collections::HashSet::new();
                            for t in STEP_TEMPLATES {
                                if seen.insert(t.category) {
                                    let col = category_color(t.category);
                                    let is_active = self.manual_category_filter == Some(t.category);
                                    let fill = if is_active {
                                        egui::Color32::from_rgba_unmultiplied(
                                            col.r(),
                                            col.g(),
                                            col.b(),
                                            100,
                                        )
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(
                                            col.r(),
                                            col.g(),
                                            col.b(),
                                            35,
                                        )
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(t.category).size(10.0),
                                            )
                                            .fill(fill)
                                            .min_size(egui::vec2(0.0, 18.0)),
                                        )
                                        .clicked()
                                    {
                                        self.manual_category_filter =
                                            if is_active { None } else { Some(t.category) };
                                    }
                                }
                            }
                        });
                        ui.separator();

                        let kw_filter = self.manual_search.to_lowercase();
                        let mut last_cat = "";
                        for t in STEP_TEMPLATES {
                            let match_filter = (kw_filter.is_empty()
                                || t.name.to_lowercase().contains(&kw_filter)
                                || t.display_name.to_lowercase().contains(&kw_filter)
                                || t.yaml.to_lowercase().contains(&kw_filter))
                                && (self.manual_category_filter.is_none()
                                    || self.manual_category_filter == Some(t.category));
                            if !match_filter {
                                continue;
                            }

                            if t.category != last_cat {
                                ui.add_space(4.0);
                                let col = category_color(t.category);
                                ui.colored_label(
                                    col,
                                    egui::RichText::new(t.category).strong().size(12.0),
                                );
                                ui.separator();
                                last_cat = t.category;
                            }

                            ui.horizontal(|ui| {
                                let col = category_color(t.category);
                                ui.colored_label(col, "▌");
                                ui.label(egui::RichText::new(t.display_name).strong())
                                    .on_hover_text(format!("{}\n\n{}", t.name, t.yaml));
                                ui.weak(format!("({})", t.name));
                                if ui.small_button("挿入").clicked() {
                                    insert_yaml = Some(t.yaml);
                                }
                            });
                            ui.indent(("manual_yaml", t.name), |ui| {
                                ui.monospace(t.yaml.trim());
                            });
                            ui.add_space(4.0);
                        }
                    });
            });

        self.manual_open = open;
        if let Some(yaml) = insert_yaml {
            self.insert_yaml_snippet(yaml);
        }
    }
}
