use anyhow::Result;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

// ---- color palette (ajisai-inspired) --------------------------------------

const COL_IMG:   egui::Color32 = egui::Color32::from_rgb(70,  130, 200);
const COL_FLOW:  egui::Color32 = egui::Color32::from_rgb(200, 140,  50);
const COL_INPUT: egui::Color32 = egui::Color32::from_rgb(100, 200, 120);
const COL_DLG:   egui::Color32 = egui::Color32::from_rgb(180, 100, 200);
const COL_VAR:   egui::Color32 = egui::Color32::from_rgb(220, 200,  80);
const COL_WAIT:  egui::Color32 = egui::Color32::from_rgb(140, 140, 140);
const COL_SCR:   egui::Color32 = egui::Color32::from_rgb(220, 100, 100);
const COL_CLIP:  egui::Color32 = egui::Color32::from_rgb(100, 200, 220);
const COL_LIB:   egui::Color32 = egui::Color32::from_rgb(180, 180, 180);

fn category_color(category: &str) -> egui::Color32 {
    match category {
        "画像操作"       => COL_IMG,
        "制御フロー"     => COL_FLOW,
        "入力操作"       => COL_INPUT,
        "ダイアログ"     => COL_DLG,
        "変数"           => COL_VAR,
        "待機"           => COL_WAIT,
        "スクリプト"     => COL_SCR,
        "クリップボード" => COL_CLIP,
        "ライブラリ"     => COL_LIB,
        _                => egui::Color32::GRAY,
    }
}

fn step_key_category(key: &str) -> &str {
    match key {
        "wait_image" | "click_image" | "find_image" | "match_rect"
            => "画像操作",
        "if" | "foreach" | "repeat" | "while" | "do_while" | "try_catch"
        | "group" | "break" | "continue" | "exit" | "sub_scenario" | "call_scenario"
            => "制御フロー",
        "type" | "press"
            => "入力操作",
        "dialog_wait" | "dialog_input" | "dialog_select"
            => "ダイアログ",
        "set" | "copy_var" | "get_datetime" | "get_username" | "calc"
        | "increment" | "to_fullwidth" | "to_halfwidth"
            => "変数",
        "wait_ms" | "wait_window"
            => "待機",
        "shell" | "script"
            => "スクリプト",
        "clipboard_set" | "clipboard_get"
            => "クリップボード",
        "library"
            => "ライブラリ",
        _   => "",
    }
}

// ---- view mode / flowchart types ------------------------------------------

#[derive(PartialEq, Clone, Copy, Default)]
enum ViewMode { #[default] List, Flow }

struct FlowNode {
    step_idx:    usize,
    depth:       usize,
    label:       String,
    color:       egui::Color32,
    expand_key:  Option<usize>,
    is_expanded: bool,
    is_header:   bool,
}

// ---- log ------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum LogLevel { Info, Ok, Error }

impl LogLevel {
    fn color(self) -> egui::Color32 {
        match self {
            LogLevel::Info  => egui::Color32::LIGHT_GRAY,
            LogLevel::Ok    => egui::Color32::from_rgb(100, 220, 100),
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
        }
    }
}

struct LogEntry {
    message: String,
    level:   LogLevel,
}

// ---- step templates -------------------------------------------------------

struct StepTemplate {
    category:     &'static str,
    display_name: &'static str,
    name:         &'static str,
    yaml:         &'static str,
}

const STEP_TEMPLATES: &[StepTemplate] = &[
    StepTemplate { category: "制御フロー", display_name: "条件分岐",        name: "if",           yaml: "if:\n  cond: \"{{ var }}\"\nthen:\n  - wait_ms: 100\nelse:\n  - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "繰り返し",         name: "foreach",      yaml: "foreach:\n  var: __rows__\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "N回繰り返し",      name: "repeat",       yaml: "repeat:\n  count: 3\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "例外処理",         name: "try_catch",    yaml: "try_catch:\n  try:\n    - wait_ms: 100\n  catch:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "サブシナリオ",     name: "sub_scenario", yaml: "sub_scenario:\n  path: sub.yaml\n" },
    StepTemplate { category: "制御フロー", display_name: "シナリオ終了",     name: "exit",         yaml: "exit: ~\n" },
    StepTemplate { category: "制御フロー", display_name: "ループ脱出",       name: "break",        yaml: "break: ~\n" },
    StepTemplate { category: "制御フロー", display_name: "次の繰り返しへ",   name: "continue",     yaml: "continue: ~\n" },
    StepTemplate { category: "画像操作",   display_name: "画像待機",         name: "wait_image",   yaml: "wait_image:\n  template: button.png\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作",   display_name: "画像クリック",     name: "click_image",  yaml: "click_image:\n  template: button.png\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作",   display_name: "画像検索",         name: "find_image",   yaml: "find_image:\n  template: button.png\n  save_as: pos\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作",   display_name: "矩形マッチング",   name: "match_rect",   yaml: "match_rect:\n  template: button.png\n  rect: { x: 0, y: 0, width: 100, height: 50 }\n  timeout_ms: 5000\n" },
    StepTemplate { category: "入力操作",   display_name: "文字入力",         name: "type",         yaml: "type: \"hello\"\n" },
    StepTemplate { category: "入力操作",   display_name: "キー押下",         name: "press",        yaml: "press: Enter\n" },
    StepTemplate { category: "待機",       display_name: "指定時間待機",     name: "wait_ms",      yaml: "wait_ms: 500\n" },
    StepTemplate { category: "待機",       display_name: "ウィンドウ待機",   name: "wait_window",  yaml: "wait_window:\n  title_contains: \"MyApp\"\n  state: exists\n  timeout_ms: 10000\n" },
    StepTemplate { category: "変数",       display_name: "変数設定",         name: "set",          yaml: "set:\n  name: my_var\n  value: \"value\"\n" },
    StepTemplate { category: "変数",       display_name: "変数コピー",       name: "copy_var",     yaml: "copy_var:\n  from: src_var\n  to: dst_var\n" },
    StepTemplate { category: "変数",       display_name: "日時取得",         name: "get_datetime", yaml: "get_datetime:\n  format: \"%Y%m%d\"\n  save_as: today\n" },
    StepTemplate { category: "変数",       display_name: "計算",             name: "calc",         yaml: "calc:\n  expr: \"a + b\"\n  save_as: result\n" },
    StepTemplate { category: "変数",       display_name: "カウントアップ",   name: "increment",    yaml: "increment:\n  name: counter\n" },
    StepTemplate { category: "変数",       display_name: "全角変換",         name: "to_fullwidth", yaml: "to_fullwidth:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "変数",       display_name: "半角変換",         name: "to_halfwidth", yaml: "to_halfwidth:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "クリップボード", display_name: "クリップボード設定", name: "clipboard_set", yaml: "clipboard_set:\n  value: \"text\"\n" },
    StepTemplate { category: "クリップボード", display_name: "クリップボード取得", name: "clipboard_get", yaml: "clipboard_get:\n  save_as: clip\n" },
    StepTemplate { category: "ダイアログ", display_name: "待機ボックス",     name: "dialog_wait",   yaml: "dialog_wait:\n  message: \"確認してください\"\n" },
    StepTemplate { category: "ダイアログ", display_name: "入力ボックス",     name: "dialog_input",  yaml: "dialog_input:\n  message: \"値を入力してください\"\n  save_as: user_input\n" },
    StepTemplate { category: "ダイアログ", display_name: "選択ボックス",     name: "dialog_select", yaml: "dialog_select:\n  message: \"選択してください\"\n  options: [\"A\", \"B\"]\n  save_as: choice\n" },
    StepTemplate { category: "スクリプト", display_name: "コマンド実行",     name: "shell",         yaml: "shell:\n  cmd: \"echo\"\n  args: [\"hello\"]\n  save_as: output\n" },
    StepTemplate { category: "スクリプト", display_name: "スクリプト実行",   name: "script",        yaml: "script:\n  script: |\n    let x = 1 + 1;\n    x\n  save_as: result\n" },
    StepTemplate { category: "ライブラリ", display_name: "ライブラリ呼び出し", name: "library",     yaml: "library:\n  name: plugin.function\n  inputs:\n    arg1: value\n  save_as: out\n" },
];

// ---- flowchart helpers ----------------------------------------------------

fn get_inner_steps(step: &serde_yml::Value) -> Vec<(&'static str, Vec<serde_yml::Value>)> {
    let m = match step.as_mapping() { Some(m) => m, None => return vec![] };
    let key = m.iter().next().and_then(|(k, _)| k.as_str()).unwrap_or("");
    match key {
        "if" => {
            let mut out = vec![];
            if let Some(seq) = m.get("then").and_then(|v| v.as_sequence()) {
                out.push(("then", seq.clone()));
            }
            if let Some(seq) = m.get("else").and_then(|v| v.as_sequence()) {
                out.push(("else", seq.clone()));
            }
            out
        }
        "foreach" | "repeat" | "while" | "do_while" => {
            if let Some(seq) = m.get(key)
                .and_then(|v| v.as_mapping())
                .and_then(|im| im.get("do"))
                .and_then(|v| v.as_sequence())
            {
                vec![("do", seq.clone())]
            } else {
                vec![]
            }
        }
        "try_catch" => {
            let inner = m.get("try_catch").and_then(|v| v.as_mapping());
            let mut out = vec![];
            if let Some(im) = inner {
                if let Some(seq) = im.get("try").and_then(|v| v.as_sequence()) {
                    out.push(("try", seq.clone()));
                }
                if let Some(seq) = im.get("catch").and_then(|v| v.as_sequence()) {
                    out.push(("catch", seq.clone()));
                }
            }
            out
        }
        "group" => {
            if let Some(seq) = m.get("group")
                .and_then(|v| v.as_mapping())
                .and_then(|im| im.get("steps"))
                .and_then(|v| v.as_sequence())
            {
                vec![("steps", seq.clone())]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

fn collect_nodes(
    steps: &[serde_yml::Value],
    depth: usize,
    expanded: &HashSet<usize>,
    nodes: &mut Vec<FlowNode>,
) {
    for (i, step) in steps.iter().enumerate() {
        let key = get_step_key(step);
        let color = category_color(step_key_category(key));
        let summary = step_summary(step);
        let is_compound = matches!(key,
            "if" | "foreach" | "repeat" | "while" | "do_while" | "try_catch" | "group"
        );
        let step_idx = i;
        let (expand_key, is_expanded) = if depth == 0 && is_compound {
            (Some(i), expanded.contains(&i))
        } else {
            (None, false)
        };
        let label = if depth == 0 { format!("{i}  {summary}") } else { summary };

        nodes.push(FlowNode { step_idx, depth, label, color, expand_key, is_expanded, is_header: false });

        if depth == 0 && is_compound && is_expanded {
            for (branch_name, branch_steps) in get_inner_steps(step) {
                nodes.push(FlowNode {
                    step_idx: i,
                    depth: depth + 1,
                    label: format!("─ {branch_name}:"),
                    color: egui::Color32::from_gray(100),
                    expand_key: None,
                    is_expanded: false,
                    is_header: true,
                });
                for inner in &branch_steps {
                    let inner_color = category_color(step_key_category(get_step_key(inner)));
                    nodes.push(FlowNode {
                        step_idx: i,
                        depth: depth + 2,
                        label: step_summary(inner),
                        color: inner_color,
                        expand_key: None,
                        is_expanded: false,
                        is_header: false,
                    });
                }
            }
        }
    }
}

// ---- helpers --------------------------------------------------------------

fn step_display_name(key: &str) -> &str {
    STEP_TEMPLATES.iter().find(|t| t.name == key).map(|t| t.display_name).unwrap_or(key)
}

fn get_step_key(v: &serde_yml::Value) -> &str {
    v.as_mapping()
        .and_then(|m| m.iter().next())
        .and_then(|(k, _)| k.as_str())
        .unwrap_or("?")
}

fn step_summary(v: &serde_yml::Value) -> String {
    let map = match v.as_mapping() {
        Some(m) => m,
        None => return "(空)".into(),
    };
    if let Some((k, val)) = map.iter().next() {
        let key = k.as_str().unwrap_or("?");
        let display = step_display_name(key);
        let val_str = match val {
            serde_yml::Value::String(s) => s.clone(),
            serde_yml::Value::Number(n) => n.to_string(),
            serde_yml::Value::Bool(b)   => b.to_string(),
            serde_yml::Value::Mapping(m) => {
                if let Some((sk, sv)) = m.iter().next() {
                    format!("{}: {}", sk.as_str().unwrap_or("?"), sv.as_str().unwrap_or("…"))
                } else {
                    "{}".into()
                }
            }
            _ => "…".into(),
        };
        format!("{display}: {val_str}")
    } else {
        "(空)".into()
    }
}

fn parse_scenario_steps(text: &str) -> Result<(String, Vec<serde_yml::Value>)> {
    let doc: serde_yml::Value = serde_yml::from_str(text)?;
    let name = doc.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_owned();
    let steps = doc.get("steps").and_then(|v| v.as_sequence()).cloned().unwrap_or_default();
    Ok((name, steps))
}

fn build_scenario_yaml(name: &str, steps: &[serde_yml::Value]) -> Result<String> {
    let mut map = serde_yml::Mapping::new();
    map.insert(serde_yml::Value::String("name".into()),  serde_yml::Value::String(name.into()));
    map.insert(serde_yml::Value::String("steps".into()), serde_yml::Value::Sequence(steps.to_vec()));
    Ok(serde_yml::to_string(&serde_yml::Value::Mapping(map))?)
}

// ---- app ------------------------------------------------------------------

struct EditorApp {
    path:           Option<PathBuf>,
    name:           String,
    steps:          Vec<serde_yml::Value>,
    selected:       Option<usize>,
    edit_buf:       String,
    parse_error:    Option<String>,
    add_menu_open:  bool,
    add_filter:     String,
    log:            Vec<LogEntry>,
    view_mode:      ViewMode,
    expanded_steps: HashSet<usize>,
}

impl Default for EditorApp {
    fn default() -> Self {
        Self {
            path:           None,
            name:           "new_scenario".into(),
            steps:          Vec::new(),
            selected:       None,
            edit_buf:       String::new(),
            parse_error:    None,
            add_menu_open:  false,
            add_filter:     String::new(),
            log:            Vec::new(),
            view_mode:      ViewMode::List,
            expanded_steps: HashSet::new(),
        }
    }
}

impl EditorApp {
    fn push_log(&mut self, message: impl Into<String>, level: LogLevel) {
        self.log.push(LogEntry { message: message.into(), level });
        if self.log.len() > 500 {
            self.log.drain(0..100);
        }
    }
    fn log_ok(&mut self, msg: impl Into<String>)  { self.push_log(msg, LogLevel::Ok); }
    fn log_err(&mut self, msg: impl Into<String>) { self.push_log(msg, LogLevel::Error); }
    fn log_info(&mut self, msg: impl Into<String>){ self.push_log(msg, LogLevel::Info); }

    fn open_file(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("YAML", &["yaml", "yml"]).pick_file() {
            match std::fs::read_to_string(&p) {
                Ok(text) => match parse_scenario_steps(&text) {
                    Ok((name, steps)) => {
                        self.name = name;
                        self.steps = steps;
                        self.selected = None;
                        self.edit_buf.clear();
                        self.path = Some(p.clone());
                        self.log_ok(format!("開きました: {}", p.display()));
                    }
                    Err(e) => self.log_err(format!("構文エラー: {e}")),
                },
                Err(e) => self.log_err(format!("読み込みエラー: {e}")),
            }
        }
    }

    fn save_file(&mut self) {
        self.flush_edit();
        let path = if let Some(ref p) = self.path {
            p.clone()
        } else if let Some(p) = rfd::FileDialog::new().add_filter("YAML", &["yaml", "yml"]).save_file() {
            self.path = Some(p.clone());
            p
        } else {
            return;
        };
        match build_scenario_yaml(&self.name, &self.steps) {
            Ok(text) => match std::fs::write(&path, &text) {
                Ok(()) => self.log_ok(format!("保存しました: {}", path.display())),
                Err(e) => self.log_err(format!("書き込みエラー: {e}")),
            },
            Err(e) => self.log_err(format!("シリアライズエラー: {e}")),
        }
    }

    fn flush_edit(&mut self) {
        if let Some(idx) = self.selected {
            match serde_yml::from_str::<serde_yml::Value>(&self.edit_buf) {
                Ok(v) => {
                    if idx < self.steps.len() { self.steps[idx] = v; }
                    self.parse_error = None;
                }
                Err(e) => {
                    self.parse_error = Some(e.to_string());
                }
            }
        }
    }

    fn select_step(&mut self, idx: usize) {
        self.flush_edit();
        self.selected = Some(idx);
        if let Some(step) = self.steps.get(idx) {
            self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
            self.parse_error = None;
        }
    }

    fn run_scenario(&mut self) {
        self.flush_edit();
        if self.path.is_none() {
            self.save_file();
        }
        let Some(ref path) = self.path else {
            self.log_err("実行するにはまず保存してください");
            return;
        };
        match std::process::Command::new("rpa").args(["run", &path.to_string_lossy()]).spawn() {
            Ok(_)  => self.log_ok("rpa を起動しました"),
            Err(e) => self.log_err(format!("起動に失敗しました: {e}")),
        }
    }

    fn build_flow_nodes(&self) -> Vec<FlowNode> {
        let mut nodes = Vec::new();
        collect_nodes(&self.steps, 0, &self.expanded_steps, &mut nodes);
        nodes
    }

    fn show_flowchart(&mut self, ui: &mut egui::Ui) {
        const NODE_W:  f32 = 300.0;
        const NODE_H:  f32 = 42.0;
        const HEAD_H:  f32 = 22.0;
        const INDENT:  f32 = 24.0;
        const GAP_Y:   f32 = 7.0;
        const PAD_X:   f32 = 20.0;
        const PAD_TOP: f32 = 16.0;
        const ICON_W:  f32 = 22.0;

        let nodes = self.build_flow_nodes();
        let selected = self.selected;

        if nodes.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("ステップがありません。左パネルで追加してください。");
            });
            return;
        }

        // Pre-compute (x, y, height) for each node
        let mut y_cur = 0.0_f32;
        let positions: Vec<(f32, f32, f32)> = nodes.iter().map(|n| {
            let x = n.depth as f32 * INDENT;
            let h = if n.is_header { HEAD_H } else { NODE_H };
            let pos = (x, y_cur, h);
            y_cur += h + GAP_Y;
            pos
        }).collect();

        let total_w = (PAD_X * 2.0 + NODE_W + nodes.iter().map(|n| n.depth as f32 * INDENT).fold(0.0_f32, f32::max))
            .max(ui.available_width());
        let total_h = PAD_TOP * 2.0 + y_cur;

        let mut toggle_expand: Option<usize> = None;
        let mut click_select:  Option<usize> = None;

        egui::ScrollArea::both()
            .id_salt("flow_canvas")
            .show(ui, |ui| {
                let (resp, painter) = ui.allocate_painter(
                    egui::vec2(total_w, total_h),
                    egui::Sense::click(),
                );

                let origin   = resp.rect.min + egui::vec2(PAD_X, PAD_TOP);
                let click_pos = if resp.clicked() { resp.interact_pointer_pos() } else { None };

                for (ni, (node, &(nx, ny, nh))) in nodes.iter().zip(positions.iter()).enumerate() {
                    let node_rect = egui::Rect::from_min_size(
                        origin + egui::vec2(nx, ny),
                        egui::vec2(NODE_W, nh),
                    );

                    // ── connector from previous node ──────────────────────────
                    if ni > 0 {
                        let &(prev_x, prev_y, prev_h) = &positions[ni - 1];
                        let from = origin + egui::vec2(prev_x + 14.0, prev_y + prev_h);
                        let to   = origin + egui::vec2(nx   + 14.0, ny);
                        let stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(65));

                        if (from.x - to.x).abs() < 0.5 {
                            painter.line_segment([from, to], stroke);
                        } else {
                            let mid_y = from.y + GAP_Y / 2.0;
                            painter.line_segment([from, egui::pos2(from.x, mid_y)], stroke);
                            painter.line_segment([egui::pos2(from.x, mid_y), egui::pos2(to.x, mid_y)], stroke);
                            painter.line_segment([egui::pos2(to.x, mid_y), to], stroke);
                        }

                        if !node.is_header {
                            painter.arrow(
                                egui::pos2(to.x, to.y - 5.0),
                                egui::vec2(0.0, 5.0),
                                egui::Stroke::new(1.5, egui::Color32::from_gray(65)),
                            );
                        }
                    }

                    // ── branch header (no box) ─────────────────────────────────
                    if node.is_header {
                        painter.text(
                            node_rect.left_center() + egui::vec2(8.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            &node.label,
                            egui::FontId::proportional(11.5),
                            egui::Color32::from_gray(128),
                        );
                        continue;
                    }

                    // ── node box ───────────────────────────────────────────────
                    let is_selected = selected == Some(node.step_idx);
                    let bg = if is_selected { egui::Color32::from_rgb(28, 52, 88) }
                             else { egui::Color32::from_gray(40) };
                    painter.rect_filled(node_rect, 5.0, bg);
                    painter.rect_stroke(node_rect, 5.0,
                        egui::Stroke::new(1.0, if is_selected {
                            egui::Color32::from_rgb(70, 125, 200)
                        } else {
                            egui::Color32::from_gray(58)
                        }),
                        egui::StrokeKind::Inside,
                    );

                    // left color stripe
                    painter.rect_filled(
                        egui::Rect::from_min_size(node_rect.min, egui::vec2(4.0, NODE_H)),
                        0.0,
                        node.color,
                    );

                    // label
                    painter.text(
                        node_rect.left_center() + egui::vec2(12.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &node.label,
                        egui::FontId::proportional(13.0),
                        egui::Color32::LIGHT_GRAY,
                    );

                    // ── expand / collapse icon ─────────────────────────────────
                    if let Some(expand_key) = node.expand_key {
                        let icon_pos = node_rect.right_center()
                            + egui::vec2(-(ICON_W / 2.0 + 5.0), 0.0);
                        painter.text(
                            icon_pos,
                            egui::Align2::CENTER_CENTER,
                            if node.is_expanded { "▼" } else { "▶" },
                            egui::FontId::proportional(11.0),
                            egui::Color32::from_gray(170),
                        );

                        if let Some(cp) = click_pos {
                            if egui::Rect::from_center_size(icon_pos, egui::vec2(ICON_W, ICON_W)).contains(cp) {
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
            });

        if let Some(key) = toggle_expand {
            if self.expanded_steps.contains(&key) { self.expanded_steps.remove(&key); }
            else { self.expanded_steps.insert(key); }
        } else if let Some(idx) = click_select {
            self.select_step(idx);
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Toolbar ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 開く").clicked()  { self.open_file(); }
                if ui.button("💾 保存").clicked()  { self.save_file(); }
                ui.separator();
                ui.label("シナリオ名:");
                ui.text_edit_singleline(&mut self.name);
                ui.separator();
                if ui.button("▶ 実行").clicked()  { self.run_scenario(); }
                ui.separator();
                ui.selectable_value(&mut self.view_mode, ViewMode::List, "リスト");
                ui.selectable_value(&mut self.view_mode, ViewMode::Flow, "フロー");

                // Last log entry right-aligned (ajisai-style status)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(last) = self.log.last() {
                        let text = if last.message.len() > 60 {
                            format!("{}…", &last.message[..57])
                        } else {
                            last.message.clone()
                        };
                        ui.colored_label(last.level.color(), text);
                    }
                });
            });
        });

        // ── Log panel (bottom, resizable) ─────────────────────────────────
        egui::TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .min_height(60.0)
            .default_height(130.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("ログ");
                    if ui.small_button("クリア").clicked() { self.log.clear(); }
                    if let Some(ref p) = self.path {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.weak(p.display().to_string());
                        });
                    }
                });
                ui.separator();
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
            });

        // ── Left: step list ───────────────────────────────────────────────
        egui::SidePanel::left("steps_panel").min_width(210.0).show(ctx, |ui| {
            ui.heading("ステップ一覧");
            ui.separator();

            let mut action: Option<StepAction> = None;
            let step_count = self.steps.len();

            egui::ScrollArea::vertical().id_salt("step_list").show(ui, |ui| {
                for i in 0..step_count {
                    let key   = get_step_key(&self.steps[i]);
                    let cat   = step_key_category(key);
                    let color = category_color(cat);
                    let summary  = step_summary(&self.steps[i]);
                    let selected = self.selected == Some(i);

                    ui.horizontal(|ui| {
                        // Category color indicator (ajisai header-bar concept)
                        ui.colored_label(color, "▌");
                        if ui.selectable_label(selected, format!("{i}: {summary}")).clicked() {
                            action = Some(StepAction::Select(i));
                        }
                        if ui.small_button("↑").clicked() && i > 0 {
                            action = Some(StepAction::MoveUp(i));
                        }
                        if ui.small_button("↓").clicked() && i + 1 < step_count {
                            action = Some(StepAction::MoveDown(i));
                        }
                        if ui.small_button("×").clicked() {
                            action = Some(StepAction::Delete(i));
                        }
                    });
                }
            });

            ui.separator();
            if ui.button("+ ステップ追加").clicked() {
                self.add_menu_open = true;
                self.add_filter.clear();
            }

            if let Some(act) = action {
                match act {
                    StepAction::Select(i) => self.select_step(i),
                    StepAction::MoveUp(i) => {
                        self.steps.swap(i - 1, i);
                        if self.selected == Some(i) { self.selected = Some(i - 1); }
                    }
                    StepAction::MoveDown(i) => {
                        self.steps.swap(i, i + 1);
                        if self.selected == Some(i) { self.selected = Some(i + 1); }
                    }
                    StepAction::Delete(i) => {
                        self.steps.remove(i);
                        if self.selected == Some(i) {
                            self.selected = None;
                            self.edit_buf.clear();
                        } else if let Some(s) = self.selected {
                            if s > i { self.selected = Some(s - 1); }
                        }
                    }
                }
            }
        });

        // ── Center: flowchart or YAML editor / onboarding ────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.view_mode == ViewMode::Flow {
                self.show_flowchart(ui);
                return;
            }

            if let Some(idx) = self.selected {
                // Inline parse error banner
                if let Some(ref err) = self.parse_error {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(80, 20, 20))
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 130, 130),
                                format!("⚠ 構文エラー: {err}"),
                            );
                        });
                }

                let key   = get_step_key(&self.steps[idx]);
                let cat   = step_key_category(key);
                let color = category_color(cat);
                ui.horizontal(|ui| {
                    ui.colored_label(color, "▌");
                    ui.label(egui::RichText::new(
                        format!("ステップ {}  —  {}", idx, step_display_name(key))
                    ).strong());
                });
                ui.separator();

                let response = egui::ScrollArea::vertical()
                    .id_salt("yaml_editor")
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.edit_buf)
                                .code_editor()
                                .desired_rows(20)
                                .desired_width(f32::INFINITY),
                        )
                    });
                if response.inner.changed() {
                    self.flush_edit();
                }
            } else if self.steps.is_empty() {
                // Onboarding guide
                ui.add_space(50.0);
                ui.vertical_centered(|ui| {
                    ui.heading("シナリオを作成しましょう");
                    ui.add_space(20.0);
                    ui.label("① 上部の「シナリオ名」を入力してください");
                    ui.add_space(8.0);
                    ui.label("② 左パネルの「+ ステップ追加」でステップを選んでください");
                    ui.add_space(8.0);
                    ui.label("③ ステップを選択すると YAML で内容を編集できます");
                    ui.add_space(8.0);
                    ui.label("④「保存」してから「実行」でシナリオを起動できます");
                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.weak("既存のシナリオを開くには上部の「📂 開く」をご利用ください");
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("左のパネルからステップを選択してください");
                });
            }
        });

        // ── Add step popup ────────────────────────────────────────────────
        if self.add_menu_open {
            let mut close  = false;
            let mut insert: Option<&'static str> = None;

            egui::Window::new("ステップを追加")
                .collapsible(false)
                .resizable(true)
                .default_size([280.0, 420.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("絞り込み:");
                        ui.text_edit_singleline(&mut self.add_filter);
                    });
                    ui.separator();

                    egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                        let filter = self.add_filter.to_lowercase();
                        let mut last_cat = "";

                        for t in STEP_TEMPLATES {
                            if !filter.is_empty()
                                && !t.name.to_lowercase().contains(&filter)
                                && !t.display_name.to_lowercase().contains(&filter)
                                && !t.category.to_lowercase().contains(&filter)
                            {
                                continue;
                            }
                            if t.category != last_cat {
                                ui.add_space(4.0);
                                // Colored category header (ajisai palette section header)
                                let col = category_color(t.category);
                                ui.colored_label(col, egui::RichText::new(t.category).strong().size(12.0));
                                ui.separator();
                                last_cat = t.category;
                            }

                            let col = category_color(t.category);
                            let label_text = format!("  {} ({})", t.display_name, t.name);
                            let btn = egui::Button::new(
                                egui::RichText::new(label_text).size(12.0)
                            )
                            .min_size(egui::vec2(250.0, 26.0));

                            // Subtle tint matching category color
                            let btn = btn.fill(
                                egui::Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 18)
                            );

                            if ui.add(btn).clicked() {
                                insert = Some(t.yaml);
                                close = true;
                            }
                        }
                    });

                    ui.separator();
                    if ui.button("キャンセル").clicked() { close = true; }
                });

            if let Some(yaml) = insert {
                match serde_yml::from_str::<serde_yml::Value>(yaml) {
                    Ok(v) => {
                        let idx = self.selected.map(|i| i + 1).unwrap_or(self.steps.len());
                        self.steps.insert(idx, v);
                        self.select_step(idx);
                        self.log_info("ステップを追加しました");
                    }
                    Err(e) => self.log_err(format!("テンプレートエラー: {e}")),
                }
            }
            if close { self.add_menu_open = false; }
        }
    }
}

enum StepAction { Select(usize), MoveUp(usize), MoveDown(usize), Delete(usize) }

// ---- main -----------------------------------------------------------------

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RPA シナリオエディター")
            .with_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "rpa-editor",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(EditorApp::default()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}
