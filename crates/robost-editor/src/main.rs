use anyhow::Result;
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

// ---- i18n -------------------------------------------------------------------

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
enum Lang {
    Ja,
    #[default]
    En,
    Zh,
}

#[allow(dead_code)]
struct S {
    // menus
    menu_file: &'static str,
    menu_new: &'static str,
    menu_open: &'static str,
    menu_save: &'static str,
    menu_save_as: &'static str,
    menu_edit: &'static str,
    menu_undo: &'static str,
    menu_redo: &'static str,
    menu_add_step: &'static str,
    menu_copy: &'static str,
    menu_cut: &'static str,
    menu_paste: &'static str,
    menu_duplicate: &'static str,
    menu_delete_step: &'static str,
    menu_view: &'static str,
    menu_list: &'static str,
    menu_flow: &'static str,
    menu_canvas: &'static str,
    canvas_reset: &'static str,
    canvas_fit: &'static str,
    canvas_snap: &'static str,
    menu_ai_panel: &'static str,
    menu_manual: &'static str,
    menu_run_menu: &'static str,
    menu_run: &'static str,
    menu_stop: &'static str,
    menu_help: &'static str,
    menu_settings: &'static str,
    menu_about: &'static str,
    // panels
    panel_nodes: &'static str,
    panel_steps: &'static str,
    panel_vars: &'static str,
    panel_log: &'static str,
    tab_problems: &'static str,
    btn_add_step: &'static str,
    btn_expand_all: &'static str,
    btn_collapse_all: &'static str,
    btn_run: &'static str,
    btn_stop: &'static str,
    btn_save: &'static str,
    btn_open: &'static str,
    // settings dialog
    settings_title: &'static str,
    settings_lang: &'static str,
    settings_provider: &'static str,
    settings_api_key: &'static str,
    settings_model: &'static str,
    settings_save: &'static str,
    settings_test: &'static str,
    // about dialog
    about_title: &'static str,
    about_rpa_tool: &'static str,
    about_version: &'static str,
    about_license: &'static str,
    // misc
    scenario_name_label: &'static str,
    hint_double_click: &'static str,
    onboard_1: &'static str,
    onboard_2: &'static str,
    onboard_3: &'static str,
    onboard_4: &'static str,
    onboard_open: &'static str,
    clear: &'static str,
    // empty state messages
    empty_no_steps: &'static str,
    empty_select_step: &'static str,
    empty_canvas_no_file: &'static str,
}

impl S {
    fn for_lang(lang: &Lang) -> Self {
        match lang {
            Lang::Ja => Self {
                menu_file: "ファイル",
                menu_new: "新規         Cmd+N",
                menu_open: "開く…       Cmd+O",
                menu_save: "保存         Cmd+S",
                menu_save_as: "名前を付けて保存  Cmd+Shift+S",
                menu_edit: "編集",
                menu_undo: "アンドゥ    Cmd+Z",
                menu_redo: "リドゥ  Cmd+Shift+Z",
                menu_add_step: "ステップ追加  Cmd+Shift+A",
                menu_copy: "コピー        Cmd+C",
                menu_cut: "カット        Cmd+X",
                menu_paste: "ペースト      Cmd+V",
                menu_duplicate: "複製          Cmd+D",
                menu_delete_step: "ステップ削除  Delete",
                menu_view: "表示",
                menu_list: "リスト",
                menu_flow: "フロー",
                menu_canvas: "キャンバス",
                canvas_reset: "配置リセット",
                canvas_fit: "全体表示",
                canvas_snap: "スナップ",
                menu_ai_panel: "AI パネル",
                menu_manual: "マニュアル",
                menu_run_menu: "実行",
                menu_run: "実行          F5",
                menu_stop: "停止          F5",
                menu_help: "ヘルプ",
                menu_settings: "設定",
                menu_about: "バージョン情報",
                panel_nodes: "ノード",
                panel_steps: "ステップ一覧",
                panel_vars: "変数",
                panel_log: "ログ",
                tab_problems: "問題",
                btn_add_step: "+ ステップ追加",
                btn_expand_all: "開",
                btn_collapse_all: "閉",
                btn_run: "実行",
                btn_stop: "停止",
                btn_save: "保存",
                btn_open: "開く",
                settings_title: "設定",
                settings_lang: "言語",
                settings_provider: "プロバイダー",
                settings_api_key: "API キー",
                settings_model: "モデル",
                settings_save: "保存",
                settings_test: "接続テスト",
                about_title: "robost について",
                about_rpa_tool: "RPA 自動化ツール",
                about_version: "バージョン",
                about_license: "ライセンス",
                scenario_name_label: "シナリオ名",
                hint_double_click: "ダブルクリックで追加",
                onboard_1: "① 上部の「シナリオ名」を入力してください",
                onboard_2: "② 左パネルの「+ ステップ追加」でステップを選んでください",
                onboard_3: "③ ステップを選択するとフォームまたは YAML で内容を編集できます",
                onboard_4: "④ 「保存」してから「実行」でシナリオを起動できます",
                onboard_open: "既存のシナリオを開くには上部の「⌘ 開く」をご利用ください",
                clear: "クリア",
                empty_no_steps: "ステップがありません。左のパレットから追加してください。",
                empty_select_step: "ステップを選択してください。",
                empty_canvas_no_file: "シナリオを開くか、List ビューでステップを追加してください。",
            },
            Lang::En => Self {
                menu_file: "File",
                menu_new: "New              Cmd+N",
                menu_open: "Open…           Cmd+O",
                menu_save: "Save             Cmd+S",
                menu_save_as: "Save As…  Cmd+Shift+S",
                menu_edit: "Edit",
                menu_undo: "Undo        Cmd+Z",
                menu_redo: "Redo   Cmd+Shift+Z",
                menu_add_step: "Add Step  Cmd+Shift+A",
                menu_copy: "Copy          Cmd+C",
                menu_cut: "Cut            Cmd+X",
                menu_paste: "Paste         Cmd+V",
                menu_duplicate: "Duplicate    Cmd+D",
                menu_delete_step: "Delete Step  Delete",
                menu_view: "View",
                menu_list: "List",
                menu_flow: "Flow",
                menu_canvas: "Canvas",
                canvas_reset: "Reset Layout",
                canvas_fit: "Fit View",
                canvas_snap: "Snap",
                menu_ai_panel: "AI Panel",
                menu_manual: "Manual",
                menu_run_menu: "Run",
                menu_run: "Run             F5",
                menu_stop: "Stop            F5",
                menu_help: "Help",
                menu_settings: "Settings",
                menu_about: "About robost",
                panel_nodes: "Nodes",
                panel_steps: "Steps",
                panel_vars: "Variables",
                panel_log: "Log",
                tab_problems: "Problems",
                btn_add_step: "+ Add Step",
                btn_expand_all: "▾",
                btn_collapse_all: "▸",
                btn_run: "Run",
                btn_stop: "Stop",
                btn_save: "Save",
                btn_open: "Open",
                settings_title: "Settings",
                settings_lang: "Language",
                settings_provider: "Provider",
                settings_api_key: "API Key",
                settings_model: "Model",
                settings_save: "Save",
                settings_test: "Test Connection",
                about_title: "About robost",
                about_rpa_tool: "RPA Automation Tool",
                about_version: "Version",
                about_license: "License",
                scenario_name_label: "Scenario name",
                hint_double_click: "Double-click to insert",
                onboard_1: "① Enter a scenario name above",
                onboard_2: "② Use \"+ Add Step\" in the left panel to pick steps",
                onboard_3: "③ Select a step to edit its properties via form or YAML",
                onboard_4: "④ Save then Run to execute the scenario",
                onboard_open: "Use \"⌘ Open\" above to load an existing scenario",
                clear: "Clear",
                empty_no_steps: "No steps. Add one from the left palette.",
                empty_select_step: "Select a step.",
                empty_canvas_no_file: "Open a scenario or add steps in List view.",
            },
            Lang::Zh => Self {
                menu_file: "文件",
                menu_new: "新建         Cmd+N",
                menu_open: "打开…       Cmd+O",
                menu_save: "保存         Cmd+S",
                menu_save_as: "另存为  Cmd+Shift+S",
                menu_edit: "编辑",
                menu_undo: "撤销    Cmd+Z",
                menu_redo: "重做  Cmd+Shift+Z",
                menu_add_step: "添加步骤  Cmd+Shift+A",
                menu_copy: "复制          Cmd+C",
                menu_cut: "剪切          Cmd+X",
                menu_paste: "粘贴          Cmd+V",
                menu_duplicate: "复制步骤    Cmd+D",
                menu_delete_step: "删除步骤  Delete",
                menu_view: "视图",
                menu_list: "列表",
                menu_flow: "流程图",
                menu_canvas: "画布",
                canvas_reset: "重置布局",
                canvas_fit: "适配视图",
                canvas_snap: "对齐网格",
                menu_ai_panel: "AI 面板",
                menu_manual: "手册",
                menu_run_menu: "运行",
                menu_run: "运行          F5",
                menu_stop: "停止          F5",
                menu_help: "帮助",
                menu_settings: "设置",
                menu_about: "关于 robost",
                panel_nodes: "节点",
                panel_steps: "步骤列表",
                panel_vars: "变量",
                panel_log: "日志",
                tab_problems: "问题",
                btn_add_step: "+ 添加步骤",
                btn_expand_all: "开",
                btn_collapse_all: "闭",
                btn_run: "运行",
                btn_stop: "停止",
                btn_save: "保存",
                btn_open: "打开",
                settings_title: "设置",
                settings_lang: "语言",
                settings_provider: "服务商",
                settings_api_key: "API 密钥",
                settings_model: "模型",
                settings_save: "保存",
                settings_test: "连接测试",
                about_title: "关于 robost",
                about_rpa_tool: "RPA 自动化工具",
                about_version: "版本",
                about_license: "许可证",
                scenario_name_label: "场景名",
                hint_double_click: "双击以插入",
                onboard_1: "① 在上方输入场景名称",
                onboard_2: "② 在左侧面板使用「添加步骤」选择步骤",
                onboard_3: "③ 选中步骤后可通过表单或 YAML 编辑其属性",
                onboard_4: "④ 保存后点击「运行」即可执行场景",
                onboard_open: "使用上方「⌘ 打开」加载已有场景",
                clear: "清除",
                empty_no_steps: "没有步骤。从左侧面板添加。",
                empty_select_step: "请选择一个步骤。",
                empty_canvas_no_file: "打开场景或在列表视图中添加步骤。",
            },
        }
    }
}

// ---- settings ---------------------------------------------------------------

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
enum AiProvider {
    Anthropic,
    OpenAI,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AppSettings {
    provider: AiProvider,
    api_key: String,
    model: String,
    #[serde(default)]
    lang: Lang,
    #[serde(default)]
    canvas_snap: bool,
    #[serde(default)]
    minimap_show: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            provider: AiProvider::Anthropic,
            api_key: String::new(),
            model: "claude-haiku-4-5-20251001".to_owned(),
            lang: Lang::default(),
            canvas_snap: false,
            minimap_show: false,
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".config")
        .join("robost")
        .join("settings.toml")
}

const KEYRING_SERVICE: &str = "robost-editor";
const KEYRING_USER: &str = "api_key";

fn load_settings() -> AppSettings {
    let path = settings_path();
    let mut settings: AppSettings = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    // Load the API key from the OS keychain (never stored in the TOML file).
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if let Ok(key) = entry.get_password() {
            settings.api_key = key;
        }
    }
    settings
}

fn save_settings(s: &AppSettings) {
    // Store the API key in the OS keychain; write everything else to TOML.
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if s.api_key.is_empty() {
            let _ = entry.delete_credential();
        } else {
            let _ = entry.set_password(&s.api_key);
        }
    }
    // Persist non-secret settings (provider, model) to TOML without the api_key.
    let mut s_safe = s.clone();
    s_safe.api_key = String::new();
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string(&s_safe) {
        #[cfg(unix)]
        {
            use std::fs::OpenOptions;
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            if let Ok(mut f) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
            {
                let _ = f.write_all(text.as_bytes());
            }
        }
        #[cfg(not(unix))]
        {
            let _ = std::fs::write(&path, text);
        }
    }
}

// ---- AI message -------------------------------------------------------------

#[derive(Clone)]
struct AiMessage {
    role: String,
    content: String,
    yaml_blocks: Vec<String>,
}

#[derive(Clone)]
enum ConfirmAction {
    OpenFile,
    NewFile,
    DeleteStep(usize),
    DeleteSteps(usize), // bulk delete — carries the count for the confirm message
    Quit,
}

// ---- color palette (ajisai-inspired) --------------------------------------

const COL_AI: egui::Color32 = egui::Color32::from_rgb(180, 80, 220);
const COL_IMG: egui::Color32 = egui::Color32::from_rgb(70, 130, 200);
const COL_FLOW: egui::Color32 = egui::Color32::from_rgb(200, 140, 50);
const COL_INPUT: egui::Color32 = egui::Color32::from_rgb(100, 200, 120);
const COL_DLG: egui::Color32 = egui::Color32::from_rgb(180, 100, 200);
const COL_VAR: egui::Color32 = egui::Color32::from_rgb(220, 200, 80);
const COL_WAIT: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);
const COL_SCR: egui::Color32 = egui::Color32::from_rgb(220, 100, 100);
const COL_CLIP: egui::Color32 = egui::Color32::from_rgb(100, 200, 220);
const COL_LIB: egui::Color32 = egui::Color32::from_rgb(180, 180, 180);
const COL_DATA: egui::Color32 = egui::Color32::from_rgb(249, 200, 120);
const COL_FILE: egui::Color32 = egui::Color32::from_rgb(180, 140, 240);
const COL_EXCEL: egui::Color32 = egui::Color32::from_rgb(33, 160, 80);
const COL_STR: egui::Color32 = egui::Color32::from_rgb(80, 180, 200);
const COL_DATE: egui::Color32 = egui::Color32::from_rgb(230, 160, 30);
const COL_JSON: egui::Color32 = egui::Color32::from_rgb(200, 180, 50);
const COL_PATH: egui::Color32 = egui::Color32::from_rgb(160, 150, 230);
const COL_MOUSE: egui::Color32 = egui::Color32::from_rgb(50, 180, 160);
const COL_PROC: egui::Color32 = egui::Color32::from_rgb(220, 100, 50);
const COL_HTTP: egui::Color32 = egui::Color32::from_rgb(50, 100, 220);
const COL_MAIL: egui::Color32 = egui::Color32::from_rgb(30, 150, 160);
const COL_WEB: egui::Color32 = egui::Color32::from_rgb(100, 190, 60);
const COL_UIA: egui::Color32 = egui::Color32::from_rgb(140, 70, 200);
const COL_CSV: egui::Color32 = egui::Color32::from_rgb(100, 170, 100);
const COL_LIST: egui::Color32 = egui::Color32::from_rgb(220, 160, 50);
const COL_UTIL: egui::Color32 = egui::Color32::from_rgb(130, 150, 200);

fn category_color(category: &str) -> egui::Color32 {
    match category {
        "AI" => COL_AI,
        "画像操作" => COL_IMG,
        "制御フロー" => COL_FLOW,
        "入力操作" => COL_INPUT,
        "ダイアログ" => COL_DLG,
        "変数" => COL_VAR,
        "待機" => COL_WAIT,
        "スクリプト" => COL_SCR,
        "クリップボード" => COL_CLIP,
        "ライブラリ" => COL_LIB,
        "データ" => COL_DATA,
        "ファイル" => COL_FILE,
        "Excel" => COL_EXCEL,
        "文字列" => COL_STR,
        "日付" => COL_DATE,
        "JSON" => COL_JSON,
        "パス" => COL_PATH,
        "マウス" => COL_MOUSE,
        "プロセス" => COL_PROC,
        "HTTP" => COL_HTTP,
        "メール" => COL_MAIL,
        "Web" => COL_WEB,
        "UIA" => COL_UIA,
        "CSV" => COL_CSV,
        "リスト" => COL_LIST,
        "ユーティリティ" => COL_UTIL,
        _ => egui::Color32::GRAY,
    }
}

fn step_key_category(key: &str) -> &str {
    match key {
        "ai_create" => "AI",
        "wait_image" | "click_image" | "find_image" | "match_rect" | "wait_no_image"
        | "wait_change" | "ocr_match" | "ml_detect" | "screenshot_save" | "get_pixel_color"
        | "wait_color" | "window_control" => "画像操作",
        "if" | "foreach" | "repeat" | "while" | "do_while" | "try_catch" | "group" | "break"
        | "continue" | "exit" | "sub_scenario" | "call_scenario" | "switch" => "制御フロー",
        "type" | "press" | "key_combo" | "click_in_window" => "入力操作",
        "dialog_wait" | "dialog_input" | "dialog_select" => "ダイアログ",
        "set" | "copy_var" | "get_datetime" | "get_username" | "calc" | "increment"
        | "to_fullwidth" | "to_halfwidth" | "import_vars" | "save_vars" | "load_vars" => "変数",
        "wait_ms" | "wait_window" | "wait_until" | "wait_process" => "待機",
        "shell" | "script" => "スクリプト",
        "clipboard_set" | "clipboard_get" => "クリップボード",
        "library" => "ライブラリ",
        "db_query" | "db_query_one" | "db_execute" | "pdf_extract_text" | "pdf_page_count" => {
            "データ"
        }
        "file_copy" | "file_move" | "file_delete" | "file_rename" | "file_exists" | "file_read"
        | "file_write" | "file_append" | "file_size" | "file_modified_at" | "file_list"
        | "log_write" | "zip_compress" | "zip_extract" | "zip_list" | "ftp_upload"
        | "ftp_download" | "ftp_list" | "ftp_delete" | "ftp_mkdir" | "dir_create"
        | "dir_delete" | "dir_exists" => "ファイル",
        "excel_read_sheet" | "excel_read_range" | "excel_read_cell" | "excel_write_cell"
        | "excel_write_range" | "excel_get_dims" | "excel_find_row" | "excel_add_sheet"
        | "excel_delete_sheet" | "excel_rename_sheet" => "Excel",
        "string_replace" | "string_trim" | "string_upper" | "string_lower" | "string_substring"
        | "string_length" | "string_split" | "string_join" | "string_regex" | "string_format"
        | "string_contains" | "string_starts_with" | "string_ends_with" | "string_index_of"
        | "string_count" => "文字列",
        "date_format" | "date_add" | "date_diff" => "日付",
        "json_parse" | "json_stringify" => "JSON",
        "path_join" | "path_basename" | "path_dirname" | "env_get" => "パス",
        "mouse_move" | "mouse_click_xy" | "mouse_drag" | "mouse_scroll" | "mouse_hover" => "マウス",
        "process_start" | "process_kill" | "process_exists" => "プロセス",
        "http_get" | "http_post" | "http_put" | "http_patch" | "http_delete" => "HTTP",
        "mail_send" | "mail_receive" => "メール",
        "web_open"
        | "web_click"
        | "web_type"
        | "web_get"
        | "web_wait"
        | "web_screenshot"
        | "web_close"
        | "web_navigate_back"
        | "web_navigate_forward"
        | "web_wait_text"
        | "web_select"
        | "web_execute_js"
        | "web_switch_frame"
        | "web_scroll"
        | "web_alert"
        | "web_get_url"
        | "web_get_title"
        | "web_get_all" => "Web",
        "uia_get" | "uia_set" | "uia_click" | "uia_find" | "uia_wait" | "uia_select"
        | "uia_get_children" | "uia_check" => "UIA",
        "csv_read" | "csv_write" => "CSV",
        "list_length" | "list_get" | "list_push" | "list_remove" | "list_contains" => "リスト",
        "base64_encode" | "base64_decode" | "to_number" | "to_string" | "var_type"
        | "number_random" | "url_open" | "notify" => "ユーティリティ",
        _ => "",
    }
}

// ---- view mode / flowchart types ------------------------------------------

#[derive(Clone)]
enum CanvasContextAction {
    Delete(usize),
    Duplicate(usize),
    OpenInList(usize),
    Paste,
    AlignLeft,
    AlignTop,
    DistributeH,
    DistributeV,
    SelectAll,
}

#[derive(PartialEq, Clone, Copy, Default)]
enum ViewMode {
    #[default]
    List,
    Flow,
    Canvas,
}

#[derive(PartialEq, Clone, Copy, Default)]
enum PropView {
    #[default]
    Form,
    Yaml,
}

#[derive(PartialEq, Default)]
enum BottomTab {
    Variables,
    #[default]
    Log,
    Problems,
}

struct ValidationIssue {
    step_idx: usize,
    message: String,
    level: LogLevel,
}

struct FlowNode {
    step_idx: usize,
    depth: usize,
    label: String,
    color: egui::Color32,
    expand_key: Option<usize>,
    is_expanded: bool,
    is_header: bool,
}

// ---- log ------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum LogLevel {
    Info,
    Ok,
    Error,
}

impl LogLevel {
    fn color(self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::LIGHT_GRAY,
            LogLevel::Ok => egui::Color32::from_rgb(100, 220, 100),
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
        }
    }
}

struct LogEntry {
    message: String,
    level: LogLevel,
}

struct Toast {
    message: String,
    level: LogLevel,
    expires: std::time::Instant,
}

// ---- step templates -------------------------------------------------------

struct StepTemplate {
    category: &'static str,
    display_name: &'static str,
    name: &'static str,
    yaml: &'static str,
}

const STEP_TEMPLATES: &[StepTemplate] = &[
    // ── AI ─────────────────────────────────────────────────────────────────
    StepTemplate { category: "AI", display_name: "AI でステップ作成", name: "ai_create", yaml: "ai_create:\n  prompt: \"\"\n" },
    // ── 制御フロー ─────────────────────────────────────────────────────────
    StepTemplate { category: "制御フロー", display_name: "条件分岐",           name: "if",           yaml: "if:\n  cond: \"{{ var }}\"\nthen:\n  - wait_ms: 100\nelse:\n  - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "繰り返し(foreach)",  name: "foreach",      yaml: "foreach:\n  var: __rows__\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "N回繰り返し",        name: "repeat",       yaml: "repeat:\n  count: 3\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "前判定ループ",        name: "while",        yaml: "while:\n  cond: \"done == false\"\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "後判定ループ",        name: "do_while",     yaml: "do_while:\n  cond: \"done == false\"\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "例外処理",           name: "try_catch",    yaml: "try_catch:\n  try:\n    - wait_ms: 100\n  catch:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "グループ",           name: "group",        yaml: "group:\n  name: グループ1\n  steps:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "分岐(switch)",       name: "switch",       yaml: "switch:\n  on: \"{{ var }}\"\n  cases:\n    - when: A\n      do:\n        - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "サブシナリオ",       name: "sub_scenario", yaml: "sub_scenario:\n  path: sub.yaml\n" },
    StepTemplate { category: "制御フロー", display_name: "シナリオ呼び出し",   name: "call_scenario",yaml: "call_scenario:\n  path: sub.yaml\n  inputs:\n    key: value\n" },
    StepTemplate { category: "制御フロー", display_name: "シナリオ終了",       name: "exit",         yaml: "exit: ~\n" },
    StepTemplate { category: "制御フロー", display_name: "ループ脱出",         name: "break",        yaml: "break: ~\n" },
    StepTemplate { category: "制御フロー", display_name: "次の繰り返しへ",     name: "continue",     yaml: "continue: ~\n" },
    // ── 画像操作 ───────────────────────────────────────────────────────────
    StepTemplate { category: "画像操作", display_name: "画像待機",             name: "wait_image",     yaml: "wait_image:\n  template: button.png\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "画像クリック",         name: "click_image",    yaml: "click_image:\n  template: button.png\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "画像検索",             name: "find_image",     yaml: "find_image:\n  template: button.png\n  save_as: pos\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "矩形マッチング",       name: "match_rect",     yaml: "match_rect:\n  template: button.png\n  rect:\n    x: 0\n    y: 0\n    width: 100\n    height: 50\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "画像消滅待機",         name: "wait_no_image",  yaml: "wait_no_image:\n  template: button.png\n  timeout_ms: 30000\n" },
    StepTemplate { category: "画像操作", display_name: "画面変化待機",         name: "wait_change",    yaml: "wait_change:\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "OCR マッチング",       name: "ocr_match",      yaml: "ocr_match:\n  contains: \"テキスト\"\n  lang: jpn+eng\n  timeout_ms: 5000\n  save_as: result\n" },
    StepTemplate { category: "画像操作", display_name: "ML 検出",              name: "ml_detect",      yaml: "ml_detect:\n  model: model.onnx\n  threshold: 0.7\n  save_as: detections\n" },
    StepTemplate { category: "画像操作", display_name: "スクリーンショット保存", name: "screenshot_save", yaml: "screenshot_save:\n  path: screenshot.png\n" },
    StepTemplate { category: "画像操作", display_name: "ピクセル色取得",       name: "get_pixel_color", yaml: "get_pixel_color:\n  x: 500\n  y: 300\n  save_as: col\n" },
    StepTemplate { category: "画像操作", display_name: "ピクセル色待機",       name: "wait_color",     yaml: "wait_color:\n  x: 500\n  y: 300\n  color: \"#00FF00\"\n  tolerance: 10\n  timeout_ms: 10000\n" },
    StepTemplate { category: "画像操作", display_name: "ウィンドウ操作",       name: "window_control", yaml: "window_control:\n  title_contains: MyApp\n  action: focus\n" },
    // ── 入力操作 ───────────────────────────────────────────────────────────
    StepTemplate { category: "入力操作", display_name: "文字入力",             name: "type",           yaml: "type: \"hello\"\n" },
    StepTemplate { category: "入力操作", display_name: "キー押下",             name: "press",          yaml: "press: Enter\n" },
    StepTemplate { category: "入力操作", display_name: "キーコンボ",           name: "key_combo",      yaml: "key_combo:\n  keys: [ctrl, c]\n" },
    StepTemplate { category: "入力操作", display_name: "ウィンドウ内クリック", name: "click_in_window",yaml: "click_in_window:\n  window: メモ帳\n  x: 100\n  y: 50\n  action: left\n" },
    // ── 待機 ───────────────────────────────────────────────────────────────
    StepTemplate { category: "待機", display_name: "指定時間待機",             name: "wait_ms",        yaml: "wait_ms: 500\n" },
    StepTemplate { category: "待機", display_name: "ウィンドウ待機",           name: "wait_window",    yaml: "wait_window:\n  title_contains: MyApp\n  state: exists\n  timeout_ms: 10000\n" },
    StepTemplate { category: "待機", display_name: "条件成立まで待機",         name: "wait_until",     yaml: "wait_until:\n  cond: \"status == \\\"done\\\"\"\n  timeout_ms: 30000\n  interval_ms: 500\n" },
    StepTemplate { category: "待機", display_name: "プロセス待機",             name: "wait_process",   yaml: "wait_process:\n  name: notepad.exe\n  state: started\n  timeout_ms: 10000\n" },
    // ── 変数 ───────────────────────────────────────────────────────────────
    StepTemplate { category: "変数", display_name: "変数設定",                 name: "set",            yaml: "set:\n  name: my_var\n  value: \"value\"\n" },
    StepTemplate { category: "変数", display_name: "変数コピー",               name: "copy_var",       yaml: "copy_var:\n  from: src_var\n  to: dst_var\n" },
    StepTemplate { category: "変数", display_name: "日時取得",                 name: "get_datetime",   yaml: "get_datetime:\n  format: \"%Y%m%d\"\n  save_as: today\n" },
    StepTemplate { category: "変数", display_name: "ユーザー名取得",           name: "get_username",   yaml: "get_username:\n  save_as: user\n" },
    StepTemplate { category: "変数", display_name: "計算",                     name: "calc",           yaml: "calc:\n  expr: \"a + b\"\n  save_as: result\n" },
    StepTemplate { category: "変数", display_name: "カウントアップ",           name: "increment",      yaml: "increment:\n  name: counter\n" },
    StepTemplate { category: "変数", display_name: "全角変換",                 name: "to_fullwidth",   yaml: "to_fullwidth:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "変数", display_name: "半角変換",                 name: "to_halfwidth",   yaml: "to_halfwidth:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "変数", display_name: "変数インポート(CSV/Excel)", name: "import_vars",   yaml: "import_vars:\n  file: data.xlsx\n  row: 0\n" },
    StepTemplate { category: "変数", display_name: "変数保存(JSON)",           name: "save_vars",      yaml: "save_vars:\n  file: vars.json\n" },
    StepTemplate { category: "変数", display_name: "変数読み込み(JSON)",       name: "load_vars",      yaml: "load_vars:\n  file: vars.json\n" },
    // ── クリップボード ─────────────────────────────────────────────────────
    StepTemplate { category: "クリップボード", display_name: "クリップボード設定", name: "clipboard_set", yaml: "clipboard_set:\n  value: \"text\"\n" },
    StepTemplate { category: "クリップボード", display_name: "クリップボード取得", name: "clipboard_get", yaml: "clipboard_get:\n  save_as: clip\n" },
    // ── ダイアログ ─────────────────────────────────────────────────────────
    StepTemplate { category: "ダイアログ", display_name: "待機ボックス",       name: "dialog_wait",    yaml: "dialog_wait:\n  message: \"確認してください\"\n" },
    StepTemplate { category: "ダイアログ", display_name: "入力ボックス",       name: "dialog_input",   yaml: "dialog_input:\n  message: \"値を入力してください\"\n  save_as: user_input\n" },
    StepTemplate { category: "ダイアログ", display_name: "選択ボックス",       name: "dialog_select",  yaml: "dialog_select:\n  message: \"選択してください\"\n  options: [A, B]\n  save_as: choice\n" },
    // ── スクリプト ─────────────────────────────────────────────────────────
    StepTemplate { category: "スクリプト", display_name: "コマンド実行",       name: "shell",          yaml: "shell:\n  cmd: echo\n  args: [hello]\n  save_as: output\n" },
    StepTemplate { category: "スクリプト", display_name: "スクリプト実行",     name: "script",         yaml: "script:\n  script: |\n    let x = 1 + 1;\n    x\n  save_as: result\n" },
    // ── ライブラリ ─────────────────────────────────────────────────────────
    StepTemplate { category: "ライブラリ", display_name: "ライブラリ呼び出し", name: "library",        yaml: "library:\n  name: plugin.function\n  inputs:\n    arg1: value\n  save_as: out\n" },
    // ── ファイル ───────────────────────────────────────────────────────────
    StepTemplate { category: "ファイル", display_name: "ファイルコピー",       name: "file_copy",      yaml: "file_copy:\n  src: source.txt\n  dst: dest.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル移動",         name: "file_move",      yaml: "file_move:\n  src: source.txt\n  dst: dest.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル削除",         name: "file_delete",    yaml: "file_delete:\n  path: file.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル名変更",       name: "file_rename",    yaml: "file_rename:\n  path: file.txt\n  name: new_name.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル存在確認",     name: "file_exists",    yaml: "file_exists:\n  path: file.txt\n  save_as: exists\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル読み込み",     name: "file_read",      yaml: "file_read:\n  path: file.txt\n  save_as: content\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル書き込み",     name: "file_write",     yaml: "file_write:\n  path: file.txt\n  content: \"{{ text }}\"\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル追記",         name: "file_append",    yaml: "file_append:\n  path: file.txt\n  content: \"{{ text }}\"\n" },
    StepTemplate { category: "ファイル", display_name: "ファイルサイズ取得",   name: "file_size",      yaml: "file_size:\n  path: file.txt\n  save_as: size\n" },
    StepTemplate { category: "ファイル", display_name: "更新日時取得",         name: "file_modified_at",yaml: "file_modified_at:\n  path: file.txt\n  format: \"%Y-%m-%d %H:%M:%S\"\n  save_as: mtime\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル一覧取得",     name: "file_list",      yaml: "file_list:\n  dir: ./data\n  pattern: \"*.csv\"\n  save_as: files\n" },
    StepTemplate { category: "ファイル", display_name: "ログ書き込み",         name: "log_write",      yaml: "log_write:\n  file: robot.log\n  message: \"{{ msg }}\"\n  level: info\n" },
    StepTemplate { category: "ファイル", display_name: "ディレクトリ作成",     name: "dir_create",     yaml: "dir_create:\n  path: output/\n" },
    StepTemplate { category: "ファイル", display_name: "ディレクトリ削除",     name: "dir_delete",     yaml: "dir_delete:\n  path: old_dir\n  recursive: true\n" },
    StepTemplate { category: "ファイル", display_name: "ディレクトリ存在確認", name: "dir_exists",     yaml: "dir_exists:\n  path: ./output\n  save_as: exists\n" },
    StepTemplate { category: "ファイル", display_name: "ZIP 圧縮",             name: "zip_compress",   yaml: "zip_compress:\n  dest: archive.zip\n  files:\n    - file1.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ZIP 展開",             name: "zip_extract",    yaml: "zip_extract:\n  src: archive.zip\n  dest: output/\n" },
    StepTemplate { category: "ファイル", display_name: "ZIP 一覧",             name: "zip_list",       yaml: "zip_list:\n  src: archive.zip\n  save_as: entries\n" },
    StepTemplate { category: "ファイル", display_name: "FTP アップロード",     name: "ftp_upload",     yaml: "ftp_upload:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  local: report.csv\n  remote: /out/report.csv\n" },
    StepTemplate { category: "ファイル", display_name: "FTP ダウンロード",     name: "ftp_download",   yaml: "ftp_download:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /in/data.csv\n  local: data.csv\n" },
    StepTemplate { category: "ファイル", display_name: "FTP 一覧",             name: "ftp_list",       yaml: "ftp_list:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /data/\n  save_as: files\n" },
    StepTemplate { category: "ファイル", display_name: "FTP 削除",             name: "ftp_delete",     yaml: "ftp_delete:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /old/file.csv\n" },
    StepTemplate { category: "ファイル", display_name: "FTP ディレクトリ作成", name: "ftp_mkdir",      yaml: "ftp_mkdir:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /new_dir\n" },
    // ── Excel ──────────────────────────────────────────────────────────────
    StepTemplate { category: "Excel", display_name: "シート読み込み",          name: "excel_read_sheet",  yaml: "excel_read_sheet:\n  file: data.xlsx\n  sheet: Sheet1\n  has_header: true\n  save_as: rows\n" },
    StepTemplate { category: "Excel", display_name: "範囲読み込み",            name: "excel_read_range",  yaml: "excel_read_range:\n  file: data.xlsx\n  sheet: Sheet1\n  range: \"A1:D100\"\n  save_as: rows\n" },
    StepTemplate { category: "Excel", display_name: "セル読み込み",            name: "excel_read_cell",   yaml: "excel_read_cell:\n  file: data.xlsx\n  sheet: Sheet1\n  cell: A1\n  save_as: value\n" },
    StepTemplate { category: "Excel", display_name: "セル書き込み",            name: "excel_write_cell",  yaml: "excel_write_cell:\n  file: data.xlsx\n  sheet: Sheet1\n  cell: A1\n  value: \"{{ var }}\"\n" },
    StepTemplate { category: "Excel", display_name: "範囲書き込み",            name: "excel_write_range", yaml: "excel_write_range:\n  file: data.xlsx\n  sheet: Sheet1\n  cell: A2\n  data: rows\n" },
    StepTemplate { category: "Excel", display_name: "シートサイズ取得",        name: "excel_get_dims",    yaml: "excel_get_dims:\n  file: data.xlsx\n  sheet: Sheet1\n  save_as: dims\n" },
    StepTemplate { category: "Excel", display_name: "行検索",                  name: "excel_find_row",    yaml: "excel_find_row:\n  file: data.xlsx\n  col: A\n  value: \"{{ search_val }}\"\n  save_as: row_num\n" },
    StepTemplate { category: "Excel", display_name: "シート追加",              name: "excel_add_sheet",   yaml: "excel_add_sheet:\n  file: data.xlsx\n  name: NewSheet\n" },
    StepTemplate { category: "Excel", display_name: "シート削除",              name: "excel_delete_sheet",yaml: "excel_delete_sheet:\n  file: data.xlsx\n  name: OldSheet\n" },
    StepTemplate { category: "Excel", display_name: "シート名変更",            name: "excel_rename_sheet",yaml: "excel_rename_sheet:\n  file: data.xlsx\n  from_name: Sheet1\n  to_name: Data\n" },
    // ── データ ─────────────────────────────────────────────────────────────
    StepTemplate { category: "データ", display_name: "DB クエリ(複数行)",      name: "db_query",          yaml: "db_query:\n  url: \"sqlite://./data.db\"\n  sql: \"SELECT * FROM table\"\n  save_as: rows\n" },
    StepTemplate { category: "データ", display_name: "DB クエリ(1行)",         name: "db_query_one",      yaml: "db_query_one:\n  url: \"sqlite://./data.db\"\n  sql: \"SELECT * FROM table WHERE id=?\"\n  params: [\"{{ id }}\"]\n  save_as: row\n" },
    StepTemplate { category: "データ", display_name: "DB 実行",                name: "db_execute",        yaml: "db_execute:\n  url: \"sqlite://./data.db\"\n  sql: \"UPDATE table SET col=? WHERE id=?\"\n  params: [\"{{ val }}\", \"{{ id }}\"]\n" },
    StepTemplate { category: "データ", display_name: "PDF テキスト抽出",       name: "pdf_extract_text",  yaml: "pdf_extract_text:\n  file: report.pdf\n  save_as: pdf_text\n" },
    StepTemplate { category: "データ", display_name: "PDF ページ数",           name: "pdf_page_count",    yaml: "pdf_page_count:\n  file: report.pdf\n  save_as: n\n" },
    // ── 文字列 ─────────────────────────────────────────────────────────────
    StepTemplate { category: "文字列", display_name: "文字列置換",             name: "string_replace",    yaml: "string_replace:\n  value: \"{{ text }}\"\n  from: old\n  to: new\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "前後の空白除去",         name: "string_trim",       yaml: "string_trim:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "大文字変換",             name: "string_upper",      yaml: "string_upper:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "小文字変換",             name: "string_lower",      yaml: "string_lower:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "部分文字列取得",         name: "string_substring",  yaml: "string_substring:\n  value: \"{{ text }}\"\n  start: 0\n  length: 5\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "文字列長取得",           name: "string_length",     yaml: "string_length:\n  value: \"{{ text }}\"\n  save_as: len\n" },
    StepTemplate { category: "文字列", display_name: "文字列分割",             name: "string_split",      yaml: "string_split:\n  value: \"{{ text }}\"\n  delimiter: \",\"\n  save_as: parts\n" },
    StepTemplate { category: "文字列", display_name: "文字列結合",             name: "string_join",       yaml: "string_join:\n  value: parts\n  separator: \",\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "正規表現マッチ",         name: "string_regex",      yaml: "string_regex:\n  value: \"{{ text }}\"\n  pattern: \"\\\\d+\"\n  save_as: match\n" },
    StepTemplate { category: "文字列", display_name: "文字列フォーマット",     name: "string_format",     yaml: "string_format:\n  format: \"Hello, {0}!\"\n  args: [\"{{ name }}\"]\n  save_as: msg\n" },
    StepTemplate { category: "文字列", display_name: "文字列を含むか確認",     name: "string_contains",   yaml: "string_contains:\n  value: \"{{ text }}\"\n  search: keyword\n  save_as: found\n" },
    StepTemplate { category: "文字列", display_name: "前方一致確認",           name: "string_starts_with",yaml: "string_starts_with:\n  value: \"{{ text }}\"\n  search: prefix\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "後方一致確認",           name: "string_ends_with",  yaml: "string_ends_with:\n  value: \"{{ text }}\"\n  search: .pdf\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "文字列検索位置",         name: "string_index_of",   yaml: "string_index_of:\n  value: \"{{ text }}\"\n  search: keyword\n  save_as: pos\n" },
    StepTemplate { category: "文字列", display_name: "出現回数カウント",       name: "string_count",      yaml: "string_count:\n  value: \"{{ text }}\"\n  search: \",\"\n  save_as: count\n" },
    // ── 日付 ───────────────────────────────────────────────────────────────
    StepTemplate { category: "日付", display_name: "日付フォーマット変換",     name: "date_format",       yaml: "date_format:\n  value: \"{{ date }}\"\n  from_format: \"%Y%m%d\"\n  to_format: \"%Y/%m/%d\"\n  save_as: result\n" },
    StepTemplate { category: "日付", display_name: "日付加算",                 name: "date_add",          yaml: "date_add:\n  value: \"{{ date }}\"\n  format: \"%Y-%m-%d\"\n  days: 1\n  save_as: result\n" },
    StepTemplate { category: "日付", display_name: "日付差分",                 name: "date_diff",         yaml: "date_diff:\n  from: \"{{ date1 }}\"\n  to: \"{{ date2 }}\"\n  format: \"%Y-%m-%d\"\n  unit: days\n  save_as: diff\n" },
    // ── JSON ───────────────────────────────────────────────────────────────
    StepTemplate { category: "JSON", display_name: "JSON パース",              name: "json_parse",        yaml: "json_parse:\n  value: \"{{ json_str }}\"\n  save_as: obj\n" },
    StepTemplate { category: "JSON", display_name: "JSON 文字列化",            name: "json_stringify",    yaml: "json_stringify:\n  value: my_var\n  save_as: json_str\n" },
    // ── パス ───────────────────────────────────────────────────────────────
    StepTemplate { category: "パス", display_name: "パス結合",                 name: "path_join",         yaml: "path_join:\n  parts: [dir, sub, file.txt]\n  save_as: path\n" },
    StepTemplate { category: "パス", display_name: "ファイル名取得",           name: "path_basename",     yaml: "path_basename:\n  path: \"{{ filepath }}\"\n  save_as: name\n" },
    StepTemplate { category: "パス", display_name: "ディレクトリ取得",         name: "path_dirname",      yaml: "path_dirname:\n  path: \"{{ filepath }}\"\n  save_as: dir\n" },
    StepTemplate { category: "パス", display_name: "環境変数取得",             name: "env_get",           yaml: "env_get:\n  name: HOME\n  save_as: home\n" },
    // ── マウス ─────────────────────────────────────────────────────────────
    StepTemplate { category: "マウス", display_name: "マウス移動",             name: "mouse_move",        yaml: "mouse_move:\n  x: \"500\"\n  y: \"300\"\n" },
    StepTemplate { category: "マウス", display_name: "座標クリック",           name: "mouse_click_xy",    yaml: "mouse_click_xy:\n  x: \"500\"\n  y: \"300\"\n  action: left\n" },
    StepTemplate { category: "マウス", display_name: "ドラッグ",               name: "mouse_drag",        yaml: "mouse_drag:\n  from_x: \"100\"\n  from_y: \"100\"\n  to_x: \"500\"\n  to_y: \"300\"\n" },
    StepTemplate { category: "マウス", display_name: "スクロール",             name: "mouse_scroll",      yaml: "mouse_scroll:\n  direction: down\n  amount: 3\n" },
    StepTemplate { category: "マウス", display_name: "ホバー",                 name: "mouse_hover",       yaml: "mouse_hover:\n  x: \"500\"\n  y: \"300\"\n  hover_ms: 500\n" },
    // ── プロセス ───────────────────────────────────────────────────────────
    StepTemplate { category: "プロセス", display_name: "プロセス起動",         name: "process_start",     yaml: "process_start:\n  command: notepad.exe\n  wait_ms: 1000\n" },
    StepTemplate { category: "プロセス", display_name: "プロセス終了",         name: "process_kill",      yaml: "process_kill:\n  name: notepad.exe\n" },
    StepTemplate { category: "プロセス", display_name: "プロセス存在確認",     name: "process_exists",    yaml: "process_exists:\n  name: notepad.exe\n  save_as: running\n" },
    // ── HTTP ───────────────────────────────────────────────────────────────
    StepTemplate { category: "HTTP", display_name: "GET リクエスト",           name: "http_get",          yaml: "http_get:\n  url: \"https://api.example.com/data\"\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "POST リクエスト",          name: "http_post",         yaml: "http_post:\n  url: \"https://api.example.com/data\"\n  body:\n    key: value\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "PUT リクエスト",           name: "http_put",          yaml: "http_put:\n  url: \"https://api.example.com/data/1\"\n  body:\n    key: value\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "PATCH リクエスト",         name: "http_patch",        yaml: "http_patch:\n  url: \"https://api.example.com/data/1\"\n  body:\n    key: value\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "DELETE リクエスト",        name: "http_delete",       yaml: "http_delete:\n  url: \"https://api.example.com/data/1\"\n  save_as: response\n" },
    // ── メール ─────────────────────────────────────────────────────────────
    StepTemplate { category: "メール", display_name: "メール送信",             name: "mail_send",         yaml: "mail_send:\n  host: smtp.example.com\n  user: user@example.com\n  password: \"{{ env.MAIL_PASS }}\"\n  from: user@example.com\n  to: to@example.com\n  subject: 件名\n  body: 本文\n" },
    StepTemplate { category: "メール", display_name: "メール受信(IMAP)",       name: "mail_receive",      yaml: "mail_receive:\n  host: imap.example.com\n  user: user@example.com\n  password: \"{{ env.MAIL_PASS }}\"\n  count: 10\n  only_unseen: true\n  save_as: emails\n" },
    // ── Web ────────────────────────────────────────────────────────────────
    StepTemplate { category: "Web", display_name: "ブラウザを開く",            name: "web_open",          yaml: "web_open:\n  url: \"https://example.com\"\n" },
    StepTemplate { category: "Web", display_name: "要素クリック",              name: "web_click",         yaml: "web_click:\n  selector: \"#submit-btn\"\n" },
    StepTemplate { category: "Web", display_name: "テキスト入力",              name: "web_type",          yaml: "web_type:\n  selector: \"#username\"\n  text: \"{{ user }}\"\n" },
    StepTemplate { category: "Web", display_name: "テキスト/属性取得",         name: "web_get",           yaml: "web_get:\n  selector: .result\n  save_as: text\n" },
    StepTemplate { category: "Web", display_name: "要素待機",                  name: "web_wait",          yaml: "web_wait:\n  selector: \"#content\"\n  timeout_ms: 5000\n" },
    StepTemplate { category: "Web", display_name: "ブラウザスクリーンショット",name: "web_screenshot",    yaml: "web_screenshot:\n  path: web_screen.png\n" },
    StepTemplate { category: "Web", display_name: "ブラウザを閉じる",          name: "web_close",         yaml: "web_close: ~\n" },
    StepTemplate { category: "Web", display_name: "テキスト含有待機",          name: "web_wait_text",     yaml: "web_wait_text:\n  selector: .status\n  text: 完了\n  timeout_ms: 10000\n" },
    StepTemplate { category: "Web", display_name: "ドロップダウン選択",        name: "web_select",        yaml: "web_select:\n  selector: \"#country\"\n  item: Japan\n" },
    StepTemplate { category: "Web", display_name: "JavaScript 実行",           name: "web_execute_js",    yaml: "web_execute_js:\n  script: \"return document.title;\"\n  save_as: title\n" },
    StepTemplate { category: "Web", display_name: "フレーム切り替え",          name: "web_switch_frame",  yaml: "web_switch_frame:\n  selector: \"#iframe1\"\n" },
    StepTemplate { category: "Web", display_name: "スクロール",                name: "web_scroll",        yaml: "web_scroll:\n  y: 300\n" },
    StepTemplate { category: "Web", display_name: "アラート処理",              name: "web_alert",         yaml: "web_alert:\n  action: accept\n" },
    StepTemplate { category: "Web", display_name: "URL 取得",                  name: "web_get_url",       yaml: "web_get_url:\n  save_as: url\n" },
    StepTemplate { category: "Web", display_name: "タイトル取得",              name: "web_get_title",     yaml: "web_get_title:\n  save_as: title\n" },
    StepTemplate { category: "Web", display_name: "全要素テキスト取得",        name: "web_get_all",       yaml: "web_get_all:\n  selector: .item\n  save_as: items\n" },
    // ── UIA ────────────────────────────────────────────────────────────────
    StepTemplate { category: "UIA", display_name: "プロパティ取得",            name: "uia_get",           yaml: "uia_get:\n  by:\n    name: ユーザー名\n  property: value\n  save_as: result\n" },
    StepTemplate { category: "UIA", display_name: "値設定",                    name: "uia_set",           yaml: "uia_set:\n  by:\n    name: 入力欄\n  value: \"{{ text }}\"\n" },
    StepTemplate { category: "UIA", display_name: "要素クリック",              name: "uia_click",         yaml: "uia_click:\n  by:\n    name: OK\n" },
    StepTemplate { category: "UIA", display_name: "要素検索",                  name: "uia_find",          yaml: "uia_find:\n  by:\n    name: ボタン\n  save_as: element\n" },
    StepTemplate { category: "UIA", display_name: "状態待機",                  name: "uia_wait",          yaml: "uia_wait:\n  by:\n    name: OK\n  state: enabled\n  timeout_ms: 10000\n" },
    StepTemplate { category: "UIA", display_name: "項目選択",                  name: "uia_select",        yaml: "uia_select:\n  by:\n    name: Country\n  item: Japan\n" },
    StepTemplate { category: "UIA", display_name: "子要素一覧取得",            name: "uia_get_children",  yaml: "uia_get_children:\n  by:\n    name: Files\n  save_as: items\n" },
    StepTemplate { category: "UIA", display_name: "チェックボックス操作",      name: "uia_check",         yaml: "uia_check:\n  by:\n    name: 同意する\n  checked: true\n" },
    // ── CSV ────────────────────────────────────────────────────────────────
    StepTemplate { category: "CSV", display_name: "CSV 読み込み",              name: "csv_read",          yaml: "csv_read:\n  path: data.csv\n  has_header: true\n  save_as: rows\n" },
    StepTemplate { category: "CSV", display_name: "CSV 書き込み",              name: "csv_write",         yaml: "csv_write:\n  path: output.csv\n  rows: my_rows\n" },
    // ── リスト ─────────────────────────────────────────────────────────────
    StepTemplate { category: "リスト", display_name: "リスト長取得",           name: "list_length",       yaml: "list_length:\n  list: my_list\n  save_as: len\n" },
    StepTemplate { category: "リスト", display_name: "要素取得",               name: "list_get",          yaml: "list_get:\n  list: my_list\n  index: \"0\"\n  save_as: item\n" },
    StepTemplate { category: "リスト", display_name: "要素追加",               name: "list_push",         yaml: "list_push:\n  list: my_list\n  value: \"{{ item }}\"\n" },
    StepTemplate { category: "リスト", display_name: "要素削除",               name: "list_remove",       yaml: "list_remove:\n  list: my_list\n  index: \"0\"\n" },
    StepTemplate { category: "リスト", display_name: "要素の存在確認",         name: "list_contains",     yaml: "list_contains:\n  list: my_list\n  value: target\n  save_as: found\n" },
    // ── ユーティリティ ─────────────────────────────────────────────────────
    StepTemplate { category: "ユーティリティ", display_name: "Base64 エンコード", name: "base64_encode",  yaml: "base64_encode:\n  value: \"{{ text }}\"\n  save_as: encoded\n" },
    StepTemplate { category: "ユーティリティ", display_name: "Base64 デコード",   name: "base64_decode",  yaml: "base64_decode:\n  value: \"{{ encoded }}\"\n  save_as: text\n" },
    StepTemplate { category: "ユーティリティ", display_name: "数値変換",          name: "to_number",      yaml: "to_number:\n  value: \"{{ str_num }}\"\n  save_as: num\n" },
    StepTemplate { category: "ユーティリティ", display_name: "文字列変換",        name: "to_string",      yaml: "to_string:\n  value: my_num\n  save_as: str\n" },
    StepTemplate { category: "ユーティリティ", display_name: "変数の型取得",      name: "var_type",       yaml: "var_type:\n  value: my_var\n  save_as: type_name\n" },
    StepTemplate { category: "ユーティリティ", display_name: "乱数生成",          name: "number_random",  yaml: "number_random:\n  min: 1.0\n  max: 100.0\n  integer: true\n  save_as: rand\n" },
    StepTemplate { category: "ユーティリティ", display_name: "URL を開く",        name: "url_open",       yaml: "url_open:\n  url: \"https://example.com\"\n" },
    StepTemplate { category: "ユーティリティ", display_name: "デスクトップ通知",  name: "notify",         yaml: "notify:\n  title: 完了\n  message: 処理が完了しました\n" },
];

// ---- flowchart helpers ----------------------------------------------------

fn get_inner_steps(step: &serde_yml::Value) -> Vec<(&'static str, Vec<serde_yml::Value>)> {
    let m = match step.as_mapping() {
        Some(m) => m,
        None => return vec![],
    };
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
            if let Some(seq) = m
                .get(key)
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
            if let Some(seq) = m
                .get("group")
                .and_then(|v| v.as_mapping())
                .and_then(|im| im.get("steps"))
                .and_then(|v| v.as_sequence())
            {
                vec![("steps", seq.clone())]
            } else {
                vec![]
            }
        }
        "switch" => {
            let inner = m.get("switch").and_then(|v| v.as_mapping());
            let mut out = vec![];
            if let Some(im) = inner {
                if let Some(cases) = im.get("cases").and_then(|v| v.as_sequence()) {
                    for case in cases {
                        if let Some(steps) = case
                            .as_mapping()
                            .and_then(|cm| cm.get("do"))
                            .and_then(|v| v.as_sequence())
                        {
                            out.push(("case", steps.clone()));
                        }
                    }
                }
                if let Some(seq) = im.get("default").and_then(|v| v.as_sequence()) {
                    out.push(("default", seq.clone()));
                }
            }
            out
        }
        _ => vec![],
    }
}

fn count_child_steps(step: &serde_yml::Value) -> usize {
    let branches = get_inner_steps(step);
    let mut total = 0;
    for (_, children) in &branches {
        total += children.len();
        for child in children {
            total += count_child_steps(child);
        }
    }
    total
}

fn default_canvas_cols(n: usize) -> usize {
    ((n as f32).sqrt() as usize + 1).clamp(4, 8)
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
        let is_compound = matches!(
            key,
            "if" | "foreach" | "repeat" | "while" | "do_while" | "try_catch" | "group" | "switch"
        );
        let step_idx = i;
        let (expand_key, is_expanded) = if depth == 0 && is_compound {
            (Some(i), expanded.contains(&i))
        } else {
            (None, false)
        };
        let label = if depth == 0 {
            format!("{i}  {summary}")
        } else {
            summary
        };

        nodes.push(FlowNode {
            step_idx,
            depth,
            label,
            color,
            expand_key,
            is_expanded,
            is_header: false,
        });

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
    STEP_TEMPLATES
        .iter()
        .find(|t| t.name == key)
        .map(|t| t.display_name)
        .unwrap_or(key)
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
            serde_yml::Value::Bool(b) => b.to_string(),
            serde_yml::Value::Mapping(m) => {
                if let Some((sk, sv)) = m.iter().next() {
                    format!(
                        "{}: {}",
                        sk.as_str().unwrap_or("?"),
                        sv.as_str().unwrap_or("…")
                    )
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

fn step_matches(step: &serde_yml::Value, query: &str) -> bool {
    step_summary(step).to_lowercase().contains(query)
        || get_step_key(step).to_lowercase().contains(query)
        || serde_yml::to_string(step)
            .unwrap_or_default()
            .to_lowercase()
            .contains(query)
}

fn parse_scenario_steps(text: &str) -> Result<(String, serde_yml::Mapping, Vec<serde_yml::Value>)> {
    let doc: serde_yml::Value = serde_yml::from_str(text)?;
    let name = doc
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed")
        .to_owned();
    let vars = doc
        .get("variables")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let steps = doc
        .get("steps")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();
    Ok((name, vars, steps))
}

fn build_scenario_yaml(
    name: &str,
    vars: &serde_yml::Mapping,
    steps: &[serde_yml::Value],
) -> Result<String> {
    let mut map = serde_yml::Mapping::new();
    map.insert(
        serde_yml::Value::String("name".into()),
        serde_yml::Value::String(name.into()),
    );
    if !vars.is_empty() {
        map.insert(
            serde_yml::Value::String("variables".into()),
            serde_yml::Value::Mapping(vars.clone()),
        );
    }
    map.insert(
        serde_yml::Value::String("steps".into()),
        serde_yml::Value::Sequence(steps.to_vec()),
    );
    Ok(serde_yml::to_string(&serde_yml::Value::Mapping(map))?)
}

// ---- AI helpers -----------------------------------------------------------

fn build_system_prompt() -> String {
    let steps: Vec<String> = STEP_TEMPLATES
        .iter()
        .map(|t| format!("\"{}\"", t.name))
        .collect();
    format!(
        "あなたはrobost RPAツールのシナリオ作成アシスタントです。\n\
         利用可能なステップ: {steps}\n\
         YAMLを提案する際は必ず```yamlブロックで囲んでください。\n\
         変数参照は {{{{ var_name }}}} 形式です。",
        steps = steps.join(", ")
    )
}

/// Extracts fenced ```yaml blocks from text.
/// Returns `(blocks, has_unclosed)` where `has_unclosed` is true if a block
/// was opened but never closed (partial block is still returned).
fn extract_yaml_blocks(text: &str) -> (Vec<String>, bool) {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut buf = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```yaml") {
            in_block = true;
            buf.clear();
        } else if in_block && line.trim() == "```" {
            let trimmed = buf.trim().to_owned();
            if !trimmed.is_empty() {
                blocks.push(trimmed);
            }
            in_block = false;
        } else if in_block {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    let unclosed = if in_block {
        let partial = buf.trim().to_owned();
        if !partial.is_empty() {
            blocks.push(partial);
        }
        true
    } else {
        false
    };
    (blocks, unclosed)
}

fn call_ai_api(
    settings: &AppSettings,
    history: &[AiMessage],
    input: &str,
    system: &str,
) -> anyhow::Result<String> {
    match settings.provider {
        AiProvider::Anthropic => call_anthropic(settings, history, input, system),
        AiProvider::OpenAI => call_openai(settings, history, input, system),
    }
}

fn call_anthropic(
    settings: &AppSettings,
    history: &[AiMessage],
    input: &str,
    system: &str,
) -> anyhow::Result<String> {
    if settings.api_key.is_empty() {
        anyhow::bail!("Anthropic APIキーが設定されていません。「設定」から入力してください。");
    }
    let mut messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();
    messages.push(serde_json::json!({ "role": "user", "content": input }));
    let body = serde_json::json!({
        "model": settings.model,
        "max_tokens": 2048,
        "system": system,
        "messages": messages,
    });
    let resp_str = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", settings.api_key.trim())
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(body)?
        .into_string()?;
    let resp: serde_json::Value = serde_json::from_str(&resp_str)?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("Anthropic API error: {}", err);
    }
    resp.get("content")
        .and_then(|c| c.get(0))
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Unexpected Anthropic response format: {resp_str}"))
        .map(str::to_owned)
}

fn call_openai(
    settings: &AppSettings,
    history: &[AiMessage],
    input: &str,
    system: &str,
) -> anyhow::Result<String> {
    if settings.api_key.is_empty() {
        anyhow::bail!("OpenAI APIキーが設定されていません。「設定」から入力してください。");
    }
    let mut msgs = vec![serde_json::json!({ "role": "system", "content": system })];
    msgs.extend(
        history
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content })),
    );
    msgs.push(serde_json::json!({ "role": "user", "content": input }));
    let body = serde_json::json!({ "model": settings.model, "messages": msgs });
    let resp_str = ureq::post("https://api.openai.com/v1/chat/completions")
        .set(
            "Authorization",
            &format!("Bearer {}", settings.api_key.trim()),
        )
        .set("content-type", "application/json")
        .send_json(body)?
        .into_string()?;
    let resp: serde_json::Value = serde_json::from_str(&resp_str)?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("OpenAI API error: {}", err);
    }
    resp.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|first| first.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Unexpected OpenAI response format: {resp_str}"))
        .map(str::to_owned)
}

fn test_ai_connection(settings: &AppSettings) -> (bool, String) {
    let result = match settings.provider {
        AiProvider::Anthropic => {
            let body = serde_json::json!({
                "model": settings.model,
                "max_tokens": 8,
                "messages": [{ "role": "user", "content": "ping" }],
            });
            ureq::post("https://api.anthropic.com/v1/messages")
                .set("x-api-key", settings.api_key.trim())
                .set("anthropic-version", "2023-06-01")
                .set("content-type", "application/json")
                .send_json(body)
                .map(|_| "Connection OK".to_owned())
                .map_err(friendly_api_error)
        }
        AiProvider::OpenAI => {
            let body = serde_json::json!({
                "model": settings.model,
                "max_tokens": 8,
                "messages": [{ "role": "user", "content": "ping" }],
            });
            ureq::post("https://api.openai.com/v1/chat/completions")
                .set(
                    "Authorization",
                    &format!("Bearer {}", settings.api_key.trim()),
                )
                .set("content-type", "application/json")
                .send_json(body)
                .map(|_| "Connection OK".to_owned())
                .map_err(friendly_api_error)
        }
    };
    match result {
        Ok(msg) => (true, msg),
        Err(e) => (false, e),
    }
}

fn friendly_api_error(e: ureq::Error) -> String {
    let msg = e.to_string();
    if msg.contains("401") {
        "401 Unauthorized — API key is invalid or expired. Check your key at console.anthropic.com"
            .to_owned()
    } else if msg.contains("403") {
        "403 Forbidden — API key lacks permission for this model".to_owned()
    } else if msg.contains("429") {
        "429 Rate limit exceeded — try again later".to_owned()
    } else {
        msg
    }
}

// ---- undo/redo state snapshot ---------------------------------------------

#[derive(Clone)]
struct EditorState {
    name: String,
    steps: Vec<serde_yml::Value>,
    selected: Option<usize>,
    selected_child: Option<(String, usize)>,
    canvas_positions: Vec<(usize, [f32; 2])>,
    canvas_zoom: f32,
    canvas_pan: [f32; 2],
}

// ---- drag-and-drop payload ------------------------------------------------

#[derive(Clone)]
enum DragPayload {
    /// YAML snippet from a StepTemplate — insert as a new step.
    NewStep(&'static str),
    /// Indices of existing steps being reordered (multi-select aware).
    ReorderStep(Vec<usize>),
}

// ---- app ------------------------------------------------------------------

struct EditorApp {
    path: Option<PathBuf>,
    name: String,
    steps: Vec<serde_yml::Value>,
    selected: Option<usize>,
    edit_buf: String,
    parse_error: Option<String>,
    add_menu_open: bool,
    add_menu_just_opened: bool,
    add_filter: String,
    add_menu_selected_idx: usize,
    log: Vec<LogEntry>,
    toasts: Vec<Toast>,
    view_mode: ViewMode,
    expanded_steps: HashSet<usize>,
    undo_stack: VecDeque<EditorState>,
    redo_stack: VecDeque<EditorState>,
    flow_zoom: f32,
    flow_pan: egui::Vec2,
    run_progress_file: Option<PathBuf>,
    current_run_step: Option<usize>,
    last_progress_check: std::time::Instant,
    dirty: bool,
    run_child: Option<std::process::Child>,
    prop_view: PropView,
    selected_child: Option<(String, usize)>,
    child_edit_buf: String,
    child_parse_error: Option<String>,
    /// When set, branch "+" opens the step picker targeting this branch.
    add_branch_target: Option<(usize, String)>,
    confirm_dialog: Option<ConfirmAction>,
    /// Persistent raw-string buffers for numeric property-form fields.
    /// Key: "fieldname@step_idx", Value: current edit string.
    form_edit_buffers: HashMap<String, String>,
    settings: AppSettings,
    settings_open: bool,
    about_open: bool,
    ai_panel_open: bool,
    ai_messages: Vec<AiMessage>,
    ai_input: String,
    ai_loading: bool,
    ai_rx: Option<std::sync::mpsc::Receiver<String>>,
    ai_unread: bool,
    md_cache: egui_commonmark::CommonMarkCache,
    manual_open: bool,
    manual_search: String,
    /// When true, flowchart will pan to center the selected node on next frame.
    scroll_to_selected: bool,
    settings_test_result: Option<(bool, String)>,
    settings_test_rx: Option<std::sync::mpsc::Receiver<(bool, String)>>,
    scenario_vars: serde_yml::Mapping,
    bottom_tab: BottomTab,
    /// Steps currently highlighted in multi-select (always contains `selected` when set).
    multi_selected: HashSet<usize>,
    /// Internal clipboard for copy/cut/paste of steps.
    step_clipboard: Vec<serde_yml::Value>,
    /// When Some, forces all node-palette categories open (true) or closed (false) for one frame.
    palette_force_open: Option<bool>,
    canvas_positions: HashMap<usize, egui::Pos2>,
    canvas_zoom: f32,
    canvas_pan: egui::Vec2,
    canvas_dragging: Option<(usize, egui::Vec2)>,
    undo_pushed_for_current_drag: bool,
    canvas_lasso: Option<(egui::Pos2, egui::Pos2)>,
    minimap_dragging: bool,
    /// Anchor node for Shift+click range selection. Persists through background clicks
    /// so clearing selection does not break the next range-select.
    canvas_selection_anchor: Option<usize>,
    canvas_viewport_size: egui::Vec2,
    /// When Some, step list rows referencing this variable name get an amber tint.
    var_highlight: Option<String>,
    /// Active category filter in the manual window (independent of text search).
    manual_category_filter: Option<&'static str>,
    /// Tracks when the undo-limit warning toast was last shown (throttles the toast).
    undo_limit_warned_at: Option<std::time::Instant>,
    /// Background channel for ai_create step generation. Holds (step_idx, receiver).
    ai_step_rx: Option<(usize, std::sync::mpsc::Receiver<anyhow::Result<String>>)>,
    /// Per-step error from the most recent ai_create generation attempt.
    ai_step_error: Option<(usize, String)>,
    /// Steps that failed during canvas execution, mapped to their error messages.
    canvas_error_steps: HashMap<usize, String>,
    /// Active edge-drag: (source step index, current pointer screen position).
    canvas_edge_drag: Option<(usize, egui::Pos2)>,
    /// Current canvas node search query.
    canvas_search: String,
    /// Whether the canvas search bar is visible.
    canvas_search_open: bool,
    /// Whether the canvas keyboard shortcut help overlay is visible.
    canvas_help_open: bool,
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
            canvas_edge_drag: None,
            canvas_search: String::new(),
            canvas_search_open: false,
            canvas_help_open: false,
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
    fn push_log(&mut self, message: impl Into<String>, level: LogLevel) {
        self.log.push(LogEntry {
            message: message.into(),
            level,
        });
        if self.log.len() > 500 {
            self.log.drain(0..100);
        }
    }
    fn push_toast(&mut self, message: String, level: LogLevel) {
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
    fn log_ok(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        self.push_toast(s.clone(), LogLevel::Ok);
        self.push_log(s, LogLevel::Ok);
    }
    fn log_err(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        self.push_toast(s.clone(), LogLevel::Error);
        self.push_log(s, LogLevel::Error);
    }
    fn log_info(&mut self, msg: impl Into<String>) {
        self.push_log(msg, LogLevel::Info);
    }

    fn commit_step(&mut self, idx: usize, mapping: serde_yml::Mapping) {
        self.steps[idx] = serde_yml::Value::Mapping(mapping);
        self.edit_buf = serde_yml::to_string(&self.steps[idx]).unwrap_or_default();
        self.dirty = true;
    }

    fn open_file(&mut self) {
        if self.dirty {
            self.confirm_dialog = Some(ConfirmAction::OpenFile);
            return;
        }
        self.do_open_file();
    }

    fn do_open_file(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file()
        {
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

    fn write_scenario_to_path(&mut self, path: PathBuf) {
        match build_scenario_yaml(&self.name, &self.scenario_vars, &self.steps) {
            Ok(text) => match std::fs::write(&path, &text) {
                Ok(()) => {
                    self.path = Some(path.clone());
                    self.dirty = false;
                    self.log_ok(format!("保存しました: {}", path.display()));
                    self.save_canvas_layout();
                }
                Err(e) => self.log_err(format!("書き込みエラー: {e}")),
            },
            Err(e) => self.log_err(format!("シリアライズエラー: {e}")),
        }
    }

    fn save_file_as(&mut self) {
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

    fn save_file(&mut self) {
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

    fn flush_edit(&mut self) {
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

    fn select_step(&mut self, idx: usize) {
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

    fn copy_selected_steps(&mut self) {
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_unstable();
        self.step_clipboard = indices
            .into_iter()
            .filter_map(|i| self.steps.get(i).cloned())
            .collect();
    }

    fn paste_steps(&mut self) {
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
                vp.x / 2.0 / z - pan.x - 130.0 + j as f32 * 40.0,
                vp.y / 2.0 / z - pan.y - 36.0 + j as f32 * 40.0,
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

    fn delete_selected_steps(&mut self) {
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

    fn stop_run(&mut self) {
        if let Some(ref mut child) = self.run_child {
            let _ = child.kill();
            let _ = child.wait();
            self.log_info("実行を停止しました");
        }
        self.run_child = None;
        self.run_progress_file = None;
        self.current_run_step = None;
    }

    fn run_selection(&mut self) {
        if self.multi_selected.is_empty() {
            self.log_err("実行するステップが選択されていません");
            return;
        }
        self.flush_edit();
        let mut indices: Vec<usize> = self.multi_selected.iter().cloned().collect();
        indices.sort_unstable();
        let selected_steps: Vec<serde_yml::Value> = indices
            .into_iter()
            .filter_map(|i| self.steps.get(i).cloned())
            .collect();
        let tmp_path =
            std::env::temp_dir().join(format!("robost_selection_{}.yaml", std::process::id()));
        match build_scenario_yaml(&self.name, &serde_yml::Mapping::new(), &selected_steps) {
            Ok(yaml) => {
                if let Err(e) = std::fs::write(&tmp_path, &yaml) {
                    self.log_err(format!("一時ファイル書き込み失敗: {e}"));
                    return;
                }
            }
            Err(e) => {
                self.log_err(format!("YAML生成失敗: {e}"));
                return;
            }
        }
        let progress_file =
            std::env::temp_dir().join(format!("robost_progress_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&progress_file);
        self.run_progress_file = Some(progress_file.clone());
        self.current_run_step = None;
        self.canvas_error_steps.clear();
        let rpa_bin = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("rpa")))
            .unwrap_or_else(|| std::path::PathBuf::from("rpa"));
        match std::process::Command::new(&rpa_bin)
            .args([
                "run",
                &tmp_path.to_string_lossy(),
                "--progress",
                &progress_file.to_string_lossy(),
            ])
            .spawn()
        {
            Ok(child) => {
                self.run_child = Some(child);
                self.log_ok(format!(
                    "{}ステップを実行中 (選択範囲)",
                    self.multi_selected.len()
                ));
            }
            Err(e) => {
                self.run_progress_file = None;
                self.log_err(format!("起動に失敗しました: {e}"));
            }
        }
    }

    fn run_scenario(&mut self) {
        self.stop_run();
        self.flush_edit();
        if self.path.is_none() {
            self.save_file();
        }
        let Some(ref path) = self.path else {
            self.log_err("実行するにはまず保存してください");
            return;
        };
        let progress_file =
            std::env::temp_dir().join(format!("robost_progress_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&progress_file);
        self.run_progress_file = Some(progress_file.clone());
        self.current_run_step = None;
        self.canvas_error_steps.clear();
        // Prefer the `rpa` binary in the same directory as this editor to avoid PATH hijacking.
        let rpa_bin = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("rpa")))
            .unwrap_or_else(|| std::path::PathBuf::from("rpa"));
        match std::process::Command::new(&rpa_bin)
            .args([
                "run",
                &path.to_string_lossy(),
                "--progress",
                &progress_file.to_string_lossy(),
            ])
            .spawn()
        {
            Ok(child) => {
                self.run_child = Some(child);
                self.log_ok("rpa を起動しました");
            }
            Err(e) => {
                self.run_progress_file = None;
                let hint = if e.kind() == std::io::ErrorKind::NotFound {
                    "\n  ヒント: rpa コマンドが見つかりません。\
                     cargo install robost-cli でインストールしてください。"
                } else {
                    ""
                };
                self.log_err(format!("起動に失敗しました: {e}{hint}"));
            }
        }
    }

    fn build_flow_nodes(&self) -> Vec<FlowNode> {
        let mut nodes = Vec::new();
        collect_nodes(&self.steps, 0, &self.expanded_steps, &mut nodes);
        nodes
    }

    fn snapshot(&self) -> EditorState {
        EditorState {
            name: self.name.clone(),
            steps: self.steps.clone(),
            selected: self.selected,
            selected_child: self.selected_child.clone(),
            canvas_positions: self
                .canvas_positions
                .iter()
                .map(|(&k, &v)| (k, [v.x, v.y]))
                .collect(),
            canvas_zoom: self.canvas_zoom,
            canvas_pan: [self.canvas_pan.x, self.canvas_pan.y],
        }
    }

    fn restore(&mut self, state: EditorState) {
        self.name = state.name;
        self.steps = state.steps;
        self.selected = state.selected;
        self.selected_child = state.selected_child;
        self.canvas_positions = state
            .canvas_positions
            .into_iter()
            .map(|(k, [x, y])| (k, egui::pos2(x, y)))
            .collect();
        self.canvas_zoom = state.canvas_zoom;
        self.canvas_pan = egui::vec2(state.canvas_pan[0], state.canvas_pan[1]);
        self.child_edit_buf.clear();
        self.edit_buf = self
            .selected
            .and_then(|i| self.steps.get(i))
            .map(|s| serde_yml::to_string(s).unwrap_or_default())
            .unwrap_or_default();
        self.parse_error = None;
        self.form_edit_buffers.clear();
    }

    fn push_undo(&mut self) {
        self.push_undo_impl(false);
    }

    fn push_undo_forced(&mut self) {
        self.push_undo_impl(true);
    }

    fn push_undo_impl(&mut self, force: bool) {
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

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop_back() {
            let current = self.snapshot();
            self.redo_stack.push_back(current);
            self.restore(prev);
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop_back() {
            let current = self.snapshot();
            self.undo_stack.push_back(current);
            self.restore(next);
        }
    }

    fn get_branch_steps(
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

    fn mutate_branch<F>(&mut self, parent_idx: usize, branch_name: &str, f: F)
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

    fn set_branch_step(
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

    fn swap_branch_steps(&mut self, parent_idx: usize, branch_name: &str, a: usize, b: usize) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            if a < seq.len() && b < seq.len() {
                seq.swap(a, b);
            }
        });
    }

    fn remove_branch_step(&mut self, parent_idx: usize, branch_name: &str, child_idx: usize) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            if child_idx < seq.len() {
                seq.remove(child_idx);
            }
        });
    }

    fn add_branch_step(
        &mut self,
        parent_idx: usize,
        branch_name: &str,
        new_step: serde_yml::Value,
    ) {
        self.mutate_branch(parent_idx, branch_name, move |seq| {
            seq.push(new_step);
        });
    }

    fn collect_var_refs_from_value<F: FnMut(&str)>(val: &serde_yml::Value, cb: &mut F) {
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

    fn validate_scenario(&self) -> Vec<ValidationIssue> {
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

    fn show_property_form(&mut self, ui: &mut egui::Ui, idx: usize) {
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
                let escaped = prompt_buf.replace('\\', "\\\\").replace('"', "\\\"");
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
                        const AI_CREATE_SYSTEM: &str = "\
あなたはRPAシナリオ作成アシスタントです。ユーザーの指示をRPAのYAMLステップに変換してください。\n\
\n\
主要ステップ:\n\
- click_image: { template: \"xxx.png\" }  # 画像クリック\n\
- type: \"テキスト\"  # 文字入力\n\
- press: Enter  # キー押下 (Tab/Enter/Escape/F1-F12 等)\n\
- wait_image: { template: \"xxx.png\", timeout_ms: 5000 }  # 画像が現れるまで待機\n\
- wait_no_image: { template: \"xxx.png\", timeout_ms: 5000 }  # 画像が消えるまで待機\n\
- wait_ms: 1000  # ミリ秒待機 (スカラー値)\n\
- key_combo: { keys: [\"ctrl\", \"c\"] }  # キーの組み合わせ\n\
- set: { name: \"変数名\", value: \"値\" }  # 変数セット\n\
\n\
必ず ```yaml コードブロックで出力してください。複数ステップも可能です:\n\
```yaml\n\
- click_image:\n\
    template: \"button.png\"\n\
```\n\
テンプレート名はユーザーの指示から推測した説明的な名前 (例: \"login_button.png\") を使用してください。";
                        let (tx, rx) = std::sync::mpsc::channel::<anyhow::Result<String>>();
                        let settings = self.settings.clone();
                        let prompt_to_send = current_prompt.clone();
                        std::thread::spawn(move || {
                            let result =
                                call_ai_api(&settings, &[], &prompt_to_send, AI_CREATE_SYSTEM);
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
                    ui.colored_label(egui::Color32::from_rgb(220, 80, 80), format!("⚠ {msg}"));
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
                                                    egui::Color32::from_rgb(220, 60, 60)
                                                };
                                                let resp = ui.add(
                                                    egui::TextEdit::singleline(buf)
                                                        .desired_width(90.0)
                                                        .text_color(text_color),
                                                );
                                                if !buf_valid && resp.has_focus() {
                                                    ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "⚠");
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
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 100, 100),
                                format!("⚠ {fk}: {err}"),
                            );
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
                                        egui::RichText::new(format!("{label} ✕")).size(11.0),
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
                                    "▸ {}  ({} ステップ)", branch_name, branch_steps.len()
                                )).strong().size(12.0);
                                egui::CollapsingHeader::new(hdr)
                                    .default_open(true)
                                    .id_salt(format!("branch_{branch_name}"))
                                    .show(ui, |ui| {
                                        for (ci, child) in branch_steps.iter().enumerate() {
                                            let ck = get_step_key(child);
                                            let col = category_color(step_key_category(ck));
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
                                                    if ui.small_button("↑").clicked() {
                                                        child_action = Some(ChildAction::MoveUp(branch_name.clone(), ci));
                                                    }
                                                });
                                                ui.add_enabled_ui(ci + 1 < branch_len, |ui| {
                                                    if ui.small_button("↓").clicked() {
                                                        child_action = Some(ChildAction::MoveDown(branch_name.clone(), ci));
                                                    }
                                                });
                                                if ui.small_button("✕").clicked() {
                                                    child_action = Some(ChildAction::Delete(branch_name.clone(), ci));
                                                }
                                            });
                                        }
                                        if ui.small_button("+ ステップ追加…").clicked() {
                                            child_action = Some(ChildAction::Add(branch_name.clone()));
                                        }
                                    });
                            }

                            // Inline YAML editor for selected child
                            if let Some((ref branch, child_idx)) = self.selected_child.clone() {
                                if let Some(branch_steps) = self.get_branch_steps(idx, branch) {
                                    if child_idx < branch_steps.len() {
                                        ui.separator();
                                        let ck = get_step_key(&branch_steps[child_idx]);
                                        let col = category_color(step_key_category(ck));
                                        let mut deselect_child = false;
                                        ui.horizontal_wrapped(|ui| {
                                            ui.colored_label(egui::Color32::from_rgb(120, 120, 120), format!("ステップ {}", idx + 1));
                                            ui.colored_label(egui::Color32::from_rgb(120, 120, 120), "(");
                                            ui.colored_label(col, outer_key);
                                            ui.colored_label(egui::Color32::from_rgb(120, 120, 120), ")");
                                            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), "▶");
                                            ui.colored_label(egui::Color32::from_rgb(100, 160, 220), branch.as_str());
                                            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), "▶");
                                            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), format!("[{}]", child_idx));
                                            ui.colored_label(col, step_display_name(ck));
                                            if ui.small_button("✕").clicked() {
                                                deselect_child = true;
                                            }
                                        });
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
                                                    let branch_clone = branch.clone();
                                                    self.push_undo();
                                                    self.set_branch_step(idx, &branch_clone, child_idx, new_child);
                                                }
                                                Err(e) => {
                                                    self.child_parse_error = Some(e.to_string());
                                                }
                                            }
                                        }
                                        if let Some(ref err) = self.child_parse_error.clone() {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 100, 100),
                                                format!("YAML エラー: {err}"),
                                            );
                                        }
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
                                self.commit_step(idx, rebuilt);
                            }
                        });
                    }
                    None => {
                        ui.label("ステップ形式が不明です。YAML タブで編集してください。");
                    }
                }
            });

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
                            self.child_edit_buf =
                                serde_yml::to_string(&steps[ci]).unwrap_or_default();
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

    fn show_flowchart(&mut self, ui: &mut egui::Ui) {
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

    fn insert_yaml_snippet(&mut self, yaml: &str) {
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
                    message: format!("✅ {}ステップを{}番目の後に挿入しました", count, at),
                    level: LogLevel::Ok,
                });
            }
            Err(e) => {
                self.log.push(LogEntry {
                    message: format!("⚠ YAML解析エラー (挿入失敗): {e}"),
                    level: LogLevel::Error,
                });
            }
        }
    }

    fn send_ai_request(&mut self) {
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
            let _ = tx.send(result.unwrap_or_else(|e| format!("⚠ API エラー: {e}")));
        });
        self.ai_messages.push(AiMessage {
            role: "user".into(),
            content: input,
            yaml_blocks: vec![],
        });
        self.ai_input.clear();
    }

    fn show_ai_panel(&mut self, ctx: &egui::Context) {
        // Floating button at bottom-right corner
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("ai_fab"))
            .fixed_pos(egui::pos2(screen.max.x - 60.0, screen.max.y - 60.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let fill = if self.ai_unread {
                    egui::Color32::from_rgb(220, 80, 40)
                } else {
                    egui::Color32::from_rgb(60, 100, 200)
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
                    .on_hover_text("AI アシスタント")
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
        egui::Window::new("AI アシスタント")
            .default_pos(default_pos)
            .default_size([360.0, 320.0])
            .min_size([280.0, 220.0])
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("AI アシスタント");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕").clicked() {
                            self.ai_panel_open = false;
                        }
                        if ui.small_button("クリア").clicked() {
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
                                    ui.strong("あなた:");
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
                                                format!(" … ({}件)", seq.len())
                                            } else {
                                                String::new()
                                            };
                                            format!("{}件: {}{}", seq.len(), names.join(", "), suffix)
                                        }
                                        Ok(ref v) => format!("1件: {}", get_step_key(v)),
                                        Err(_) => "(プレビュー不可)".to_owned(),
                                    };
                                    if ui
                                        .button(format!("📋 挿入 #{}", bi + 1))
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
                            ui.label("処理中…");
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    let resp = ui.add(
                        egui::TextEdit::multiline(&mut self.ai_input)
                            .desired_rows(2)
                            .desired_width(ui.available_width() - 55.0)
                            .hint_text(
                                "e.g. \"Create an Excel loop\"\n\"画像クリックのステップを追加して\"",
                            ),
                    );
                    let send = ui.add_enabled(
                        !self.ai_loading,
                        egui::Button::new("送信").min_size(egui::vec2(50.0, 44.0)),
                    );
                    if send.clicked()
                        || (resp.has_focus()
                            && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                            && ctx.input(|i| i.modifiers.ctrl))
                    {
                        self.send_ai_request();
                    }
                });
                ui.weak("Ctrl+Enter で送信");
            });
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
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
                        ui.label(format!("{}:", s.settings_lang));
                        egui::ComboBox::from_id_salt("lang_combo")
                            .selected_text(match self.settings.lang {
                                Lang::Ja => "日本語",
                                Lang::En => "English",
                                Lang::Zh => "中文",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.settings.lang, Lang::En, "English");
                                ui.selectable_value(&mut self.settings.lang, Lang::Ja, "日本語");
                                ui.selectable_value(&mut self.settings.lang, Lang::Zh, "中文");
                            });
                        ui.end_row();

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
                        let models: &[&str] = match self.settings.provider {
                            AiProvider::Anthropic => &[
                                "claude-haiku-4-5-20251001",
                                "claude-sonnet-4-6",
                                "claude-opus-4-7",
                            ],
                            AiProvider::OpenAI => &["gpt-4o-mini", "gpt-4o"],
                        };
                        egui::ComboBox::from_id_salt("model_combo")
                            .selected_text(&self.settings.model)
                            .show_ui(ui, |ui| {
                                for m in models {
                                    ui.selectable_value(
                                        &mut self.settings.model,
                                        m.to_string(),
                                        *m,
                                    );
                                }
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
                            ui.colored_label(
                                egui::Color32::from_rgb(80, 200, 80),
                                format!("✓ {msg}"),
                            );
                        } else {
                            ui.colored_label(
                                egui::Color32::from_rgb(220, 60, 60),
                                format!("✗ {msg}"),
                            );
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

    fn show_manual_window(&mut self, ctx: &egui::Context) {
        if !self.manual_open {
            return;
        }
        let mut open = self.manual_open;
        let mut insert_yaml: Option<&'static str> = None;

        egui::Window::new("マニュアル — ステップ一覧")
            .open(&mut open)
            .resizable(true)
            .default_size([500.0, 560.0])
            .show(ctx, |ui| {
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
                                egui::Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 35)
                            };
                            if ui
                                .add(
                                    egui::Button::new(egui::RichText::new(t.category).size(10.0))
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
                egui::ScrollArea::vertical()
                    .max_height(440.0)
                    .show(ui, |ui| {
                        let mut last_cat = "";
                        for t in STEP_TEMPLATES {
                            let match_filter = (kw_filter.is_empty()
                                || t.name.to_lowercase().contains(&kw_filter)
                                || t.display_name.to_lowercase().contains(&kw_filter))
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
                                ui.label(egui::RichText::new(t.display_name).size(12.0))
                                    .on_hover_text(format!(
                                        "{}\n\n```yaml\n{}\n```",
                                        t.name, t.yaml
                                    ));
                                ui.weak(egui::RichText::new(format!("({})", t.name)).size(10.0));
                                if ui.small_button("挿入").clicked() {
                                    insert_yaml = Some(t.yaml);
                                }
                            });
                        }
                    });
            });

        self.manual_open = open;
        if let Some(yaml) = insert_yaml {
            self.insert_yaml_snippet(yaml);
        }
    }

    fn ensure_canvas_layout(&mut self) {
        const NODE_W: f32 = 260.0;
        const NODE_H: f32 = 72.0;
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

    fn canvas_shift_positions(positions: &mut HashMap<usize, egui::Pos2>, at: usize, delta: isize) {
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

    fn save_canvas_layout(&self) {
        let Some(ref path) = self.path else { return };
        let lpath = layout_path(path);
        let mut positions = serde_json::Map::new();
        for (k, v) in &self.canvas_positions {
            positions.insert(k.to_string(), serde_json::json!([v.x, v.y]));
        }
        let layout = serde_json::json!({ "positions": positions });
        if let Ok(text) = serde_json::to_string(&layout) {
            let _ = std::fs::write(&lpath, text);
        }
    }

    fn load_canvas_layout(&mut self, scenario_path: &std::path::Path) {
        self.canvas_positions.clear();
        let lpath = layout_path(scenario_path);
        let Ok(text) = std::fs::read_to_string(&lpath) else {
            return;
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        let Some(obj) = val["positions"].as_object() else {
            return;
        };
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

    fn canvas_fit_view(&mut self, viewport_size: egui::Vec2) {
        let n = self.steps.len();
        if n == 0 {
            return;
        }
        const NODE_W: f32 = 260.0;
        const NODE_H: f32 = 72.0;
        let margin = 40.0_f32;

        // Use the same default-position formula as show_canvas so nodes without a stored
        // position (e.g. freshly loaded scenarios) are included in the bounding box.
        let cols = default_canvas_cols(n);
        let pos_of = |i: usize| {
            self.canvas_positions.get(&i).copied().unwrap_or_else(|| {
                egui::pos2(
                    (i % cols) as f32 * 340.0 + 40.0,
                    (i / cols) as f32 * 132.0 + 40.0,
                )
            })
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
        let effective_w = if n >= 15 {
            (viewport_size.x - MM_W - MM_MARGIN * 2.0).max(viewport_size.x * 0.5)
        } else {
            viewport_size.x
        };
        let zoom_x = effective_w / content_w;
        let zoom_y = viewport_size.y / content_h;
        self.canvas_zoom = zoom_x.min(zoom_y).clamp(0.25, 2.0);

        // Center in the visible area (minimap occupies bottom-right when n >= 15)
        const MM_W_FV: f32 = 160.0;
        const MM_MARGIN_FV: f32 = 8.0;
        let vis_w = if n >= 15 {
            (viewport_size.x - MM_W_FV - MM_MARGIN_FV * 2.0).max(viewport_size.x * 0.5)
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
    fn form_edit_buffers_shift(buffers: &mut HashMap<String, String>, at: usize, delta: isize) {
        let keys: Vec<String> = buffers.keys().cloned().collect();
        let mut to_move: Vec<(String, usize)> = keys
            .iter()
            .filter_map(|k| {
                let at_pos = k.rfind('@')?;
                let n: usize = k[at_pos + 1..].parse().ok()?;
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
                    let prefix = &key[..key.rfind('@').unwrap()];
                    buffers.insert(format!("{prefix}@{}", n + 1), val);
                }
            }
        } else {
            to_move.sort_by_key(|a| a.1);
            for (key, n) in to_move {
                if n == at {
                    buffers.remove(&key);
                } else if let Some(val) = buffers.remove(&key) {
                    let prefix = &key[..key.rfind('@').unwrap()];
                    buffers.insert(format!("{prefix}@{}", n - 1), val);
                }
            }
        }
    }

    fn canvas_align_left(&mut self) {
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
        self.save_canvas_layout();
    }

    fn canvas_align_top(&mut self) {
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
        self.save_canvas_layout();
    }

    fn canvas_distribute_h(&mut self) {
        if self.multi_selected.len() < 3 {
            return;
        }
        const NODE_W: f32 = 260.0;
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
        self.save_canvas_layout();
    }

    fn canvas_distribute_v(&mut self) {
        if self.multi_selected.len() < 3 {
            return;
        }
        const NODE_H: f32 = 72.0;
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
        self.save_canvas_layout();
    }

    fn show_canvas(&mut self, ui: &mut egui::Ui) {
        use egui::{epaint::CubicBezierShape, Align2, Color32, FontId, Rect, Sense, Stroke, Vec2};

        // Pre-compute search state so it can be used throughout without borrow conflicts
        let search_query = self.canvas_search.to_lowercase();
        let search_active = !search_query.is_empty();

        const NODE_W: f32 = 260.0;
        const NODE_H: f32 = 72.0;

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
                let p = self.canvas_positions.get(&i).copied().unwrap_or(egui::pos2(
                    (i % default_cols) as f32 * 340.0 + 40.0,
                    (i / default_cols) as f32 * 132.0 + 40.0,
                ));
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

        // Dark canvas background
        painter.rect_filled(resp.rect, 0.0, egui::Color32::from_rgb(26, 27, 30));

        // Background dot grid (every 40px in canvas space, drawn as small dots)
        {
            let grid = 40.0 * z;
            let offset_x = (self.canvas_pan.x * z).rem_euclid(grid);
            let offset_y = (self.canvas_pan.y * z).rem_euclid(grid);
            let mut x = resp.rect.min.x + offset_x;
            while x < resp.rect.max.x {
                let mut y = resp.rect.min.y + offset_y;
                while y < resp.rect.max.y {
                    painter.circle_filled(
                        egui::pos2(x, y),
                        1.0_f32.max(0.5 * z),
                        egui::Color32::from_gray(50),
                    );
                    y += grid;
                }
                x += grid;
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
                let z_new = (z_old * (1.0 + scroll.y * 0.001)).clamp(0.25, 2.0);
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
                            let p = self.canvas_positions.get(&i).copied().unwrap_or(egui::pos2(
                                (i % default_cols) as f32 * 340.0 + 40.0,
                                (i / default_cols) as f32 * 132.0 + 40.0,
                            ));
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
                let p = self.canvas_positions.get(&i).copied().unwrap_or(egui::pos2(
                    (i % default_cols) as f32 * 340.0 + 40.0,
                    (i / default_cols) as f32 * 132.0 + 40.0,
                ));
                origin + (p.to_vec2() + self.canvas_pan) * z
            })
            .collect();

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
                Color32::from_gray(70)
            };
            let stroke_width = if is_compound { 1.0 * z } else { 1.5 * z };
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
            // Midpoint "+" insertion button (visible only on hover over edge area, z >= 0.6)
            if z >= 0.6 {
                let mid = from.lerp(to, 0.5);
                let btn_size = 14.0 * z;
                let btn_rect = egui::Rect::from_center_size(mid, egui::vec2(btn_size, btn_size));
                let btn_resp = ui.allocate_rect(btn_rect, Sense::click());
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
                        Color32::from_rgba_premultiplied(50, 80, 140, 50),
                    );
                    painter.text(
                        mid,
                        Align2::CENTER_CENTER,
                        "+",
                        FontId::proportional(10.0 * z),
                        Color32::from_rgba_premultiplied(180, 180, 200, 80),
                    );
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
                        Stroke::new(2.0 * z, Color32::from_rgb(100, 180, 255)),
                    ),
                ));
                painter.circle_filled(from_p, 5.0 * z, Color32::from_rgb(100, 180, 255));
                painter.circle_filled(end_pos, 5.0 * z, Color32::from_rgb(100, 180, 255));
                // Show "→ ここに移動" label at midpoint of the drag line
                let mid = from_p.lerp(end_pos, 0.5);
                painter.text(
                    mid + egui::Vec2::new(8.0 * z, -10.0 * z),
                    egui::Align2::LEFT_CENTER,
                    "→ ここに移動",
                    egui::FontId::proportional(10.0 * z),
                    egui::Color32::from_rgb(100, 200, 255),
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
        let mut panel_click_step: Option<usize> = None;
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
                if is_cpd && z >= 0.7 && badge_rect.contains(click_pos) {
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
                if ui.button("削除").clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::Delete(idx));
                    ui.close();
                }
                if ui.button("複製").clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::Duplicate(idx));
                    ui.close();
                }
                if !self.step_clipboard.is_empty() && ui.button("貼り付け").clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::Paste);
                    ui.close();
                }
                ui.separator();
                if ui.button("List ビューで開く").clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::OpenInList(idx));
                    ui.close();
                }
                if self.multi_selected.len() >= 2 {
                    ui.separator();
                    ui.weak(format!("整列 ({} 選択)", self.multi_selected.len()));
                    if ui.button("← 左揃え").clicked() {
                        canvas_ctx_action = Some(CanvasContextAction::AlignLeft);
                        ui.close();
                    }
                    if ui.button("↑ 上揃え").clicked() {
                        canvas_ctx_action = Some(CanvasContextAction::AlignTop);
                        ui.close();
                    }
                    if self.multi_selected.len() >= 3 {
                        if ui.button("↔ 水平等間隔").clicked() {
                            canvas_ctx_action = Some(CanvasContextAction::DistributeH);
                            ui.close();
                        }
                        if ui.button("↕ 垂直等間隔").clicked() {
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

            // Show full label as tooltip on hover when text is truncated (before culling)
            if hovered {
                if let Some(err_msg) = self.canvas_error_steps.get(&idx) {
                    let msg = err_msg.clone();
                    node_resp.show_tooltip_ui(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(220, 80, 80), &msg);
                    });
                } else if full_label.chars().count() > 32 {
                    node_resp.show_tooltip_ui(|ui| {
                        ui.label(&full_label);
                    });
                }
            }

            // ── Draw (skip nodes fully outside the viewport) ────────────────
            if !resp.rect.intersects(node_rect) {
                continue;
            }

            let selected = self.selected == Some(idx);
            let is_multi_only = self.multi_selected.contains(&idx) && !selected;
            let is_running = self.current_run_step == Some(idx);
            let key = get_step_key(step);
            let cat_color = category_color(step_key_category(key));
            // Blend category color into base background
            let base_bg = if selected {
                Color32::from_rgb(28, 52, 88)
            } else if is_running {
                Color32::from_rgb(80, 60, 10)
            } else {
                Color32::from_gray(40)
            };
            let blended_bg = {
                let [r0, g0, b0, _] = base_bg.to_array();
                let [r1, g1, b1, _] = cat_color.to_array();
                Color32::from_rgb(
                    ((r0 as u16 * 7 + r1 as u16) / 8) as u8,
                    ((g0 as u16 * 7 + g1 as u16) / 8) as u8,
                    ((b0 as u16 * 7 + b1 as u16) / 8) as u8,
                )
            };
            let (border, border_w) = if selected {
                (Color32::from_rgb(90, 150, 230), (1.5 * z).max(1.0))
            } else if is_multi_only {
                (Color32::from_rgb(75, 115, 215), (1.5 * z).max(1.0))
            } else {
                (Color32::from_gray(68), 1.0)
            };

            // Drop shadow (offset 3px down-right)
            let shadow_rect = node_rect.translate(egui::vec2(3.0 * z, 3.0 * z));
            painter.rect_filled(
                shadow_rect,
                4.0 * z,
                Color32::from_rgba_premultiplied(0, 0, 0, 60),
            );

            painter.rect_filled(node_rect, 4.0 * z, blended_bg);
            if is_multi_only {
                painter.rect_filled(
                    node_rect,
                    4.0 * z,
                    egui::Color32::from_rgba_premultiplied(60, 100, 200, 40),
                );
            }
            painter.rect_stroke(
                node_rect,
                4.0 * z,
                Stroke::new(border_w, border),
                egui::StrokeKind::Inside,
            );
            if is_multi_only {
                painter.rect_stroke(
                    node_rect,
                    4.0 * z,
                    egui::Stroke::new(1.5, Color32::from_rgb(60, 100, 200)),
                    egui::StrokeKind::Outside,
                );
            }
            // Hover outline
            if hovered && !selected {
                painter.rect_stroke(
                    node_rect,
                    4.0 * z,
                    Stroke::new(1.0, Color32::from_rgb(120, 140, 160)),
                    egui::StrokeKind::Outside,
                );
            }

            // Left color stripe (widened)
            let stripe = Rect::from_min_size(node_rect.min, Vec2::new(6.0 * z, NODE_H * z));
            painter.rect_filled(stripe, 0.0, cat_color);

            // Progressive node content based on zoom level
            if z < 0.5 {
                // Minimal: only step index centered — text too small to read otherwise
                painter.text(
                    node_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{}", idx + 1),
                    FontId::proportional(10.0 * z),
                    Color32::from_gray(160),
                );
            } else {
                let (idx_y, label_y) = if z >= 1.5 {
                    (NODE_H * z * 0.22, NODE_H * z * 0.52)
                } else {
                    (NODE_H * z * 0.3, NODE_H * z * 0.68)
                };
                painter.text(
                    node_rect.min + Vec2::new(10.0 * z, idx_y),
                    Align2::LEFT_CENTER,
                    format!("{}", idx + 1),
                    FontId::proportional(9.0 * z),
                    Color32::from_gray(140),
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
                    FontId::proportional(11.0 * z),
                    Color32::from_gray(220),
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
                                panel_click_step = Some(idx);
                            }
                            let text_color = if row_resp.hovered() {
                                Color32::from_rgb(220, 210, 180)
                            } else {
                                Color32::from_rgb(180, 170, 150)
                            };
                            let text = format!("▸ {}  {} steps", label, steps_vec.len());
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
                    egui::Stroke::new((2.0 * z).max(1.0), egui::Color32::from_rgb(249, 226, 175)),
                    egui::StrokeKind::Outside,
                );
            }
            if self.canvas_error_steps.contains_key(&idx) {
                painter.rect_stroke(
                    node_rect.expand((2.0 * z).max(1.0)),
                    4.0 * z,
                    egui::Stroke::new((2.0 * z).max(1.0), egui::Color32::from_rgb(220, 60, 60)),
                    egui::StrokeKind::Outside,
                );
            }

            // Drag-handle at bottom-center — communicates reorder, not graph connection
            let port_center = node_rect.min + Vec2::new(NODE_W * z / 2.0, NODE_H * z);
            let port_hovered = ui
                .input(|i| i.pointer.hover_pos())
                .map(|p| p.distance(port_center) <= 10.0 * z)
                .unwrap_or(false);
            let in_edge_drag = self.canvas_edge_drag.is_some();
            if z >= 0.5 {
                let handle_color = if port_hovered && !in_edge_drag {
                    Color32::from_rgb(180, 180, 200)
                } else if in_edge_drag {
                    Color32::from_rgb(100, 180, 255)
                } else {
                    Color32::from_rgba_premultiplied(140, 140, 160, 90)
                };
                painter.text(
                    port_center,
                    Align2::CENTER_CENTER,
                    "⠿",
                    FontId::proportional(11.0 * z),
                    handle_color,
                );
                if port_hovered && !in_edge_drag {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    // Allocate a tiny invisible rect to show tooltip via egui
                    let tip_rect =
                        egui::Rect::from_center_size(port_center, egui::vec2(20.0 * z, 12.0 * z));
                    let tip_resp = ui.allocate_rect(tip_rect, Sense::hover());
                    tip_resp.on_hover_text("ドラッグして並び替え");
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
                        egui::Stroke::new(2.5 * z, egui::Color32::from_rgb(80, 200, 120)),
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
                        egui::Color32::from_rgb(80, 200, 120),
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
                    Self::canvas_shift_positions(&mut self.canvas_positions, insert_at, 1);
                    if let Some(pos) = src_pos {
                        self.canvas_positions.insert(insert_at, pos);
                    }
                    self.steps.insert(insert_at, step);
                    self.dirty = true;
                    self.save_canvas_layout();
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
            self.save_canvas_layout();
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
        if let Some(idx) = panel_click_step {
            self.select_step(idx);
            // Stay in canvas; just select the node (don't force List view)
        }

        // Background right-click menu (shown when clicking empty canvas space, not on a node)
        resp.context_menu(|ui| {
            if ui.button("全選択").clicked() {
                canvas_ctx_action = Some(CanvasContextAction::SelectAll);
                ui.close();
            }
            if !self.step_clipboard.is_empty() && ui.button("貼り付け").clicked() {
                canvas_ctx_action = Some(CanvasContextAction::Paste);
                ui.close();
            }
            if self.multi_selected.len() >= 2 {
                ui.separator();
                ui.weak(format!("整列 ({} 選択)", self.multi_selected.len()));
                if ui.button("← 左揃え").clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::AlignLeft);
                    ui.close();
                }
                if ui.button("↑ 上揃え").clicked() {
                    canvas_ctx_action = Some(CanvasContextAction::AlignTop);
                    ui.close();
                }
                if self.multi_selected.len() >= 3 {
                    if ui.button("↔ 水平等間隔").clicked() {
                        canvas_ctx_action = Some(CanvasContextAction::DistributeH);
                        ui.close();
                    }
                    if ui.button("↕ 垂直等間隔").clicked() {
                        canvas_ctx_action = Some(CanvasContextAction::DistributeV);
                        ui.close();
                    }
                }
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
                CanvasContextAction::Paste => self.paste_steps(),
                CanvasContextAction::AlignLeft => self.canvas_align_left(),
                CanvasContextAction::AlignTop => self.canvas_align_top(),
                CanvasContextAction::DistributeH => self.canvas_distribute_h(),
                CanvasContextAction::DistributeV => self.canvas_distribute_v(),
                CanvasContextAction::SelectAll => {
                    self.multi_selected = (0..self.steps.len()).collect();
                    self.selected = self.multi_selected.iter().min().cloned();
                }
            }
        }

        // Empty state hint
        if n == 0 {
            let s = S::for_lang(&self.settings.lang);
            let (msg, cta) = if self.path.is_none() {
                (
                    s.empty_canvas_no_file,
                    "Cmd+N で新規シナリオ / Cmd+O で開く",
                )
            } else {
                (s.empty_no_steps, "Cmd+Shift+A でステップを追加")
            };
            let center = resp.rect.center();
            painter.text(
                center + egui::vec2(0.0, -10.0),
                Align2::CENTER_CENTER,
                msg,
                FontId::proportional(13.0),
                Color32::from_gray(100),
            );
            painter.text(
                center + egui::vec2(0.0, 14.0),
                Align2::CENTER_CENTER,
                cta,
                FontId::proportional(11.0),
                Color32::from_gray(65),
            );
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
                    self.multi_selected.clear();
                    for (i, &sp) in screen_positions.iter().enumerate() {
                        let nr = egui::Rect::from_min_size(sp, Vec2::new(NODE_W * z, NODE_H * z));
                        if lasso_rect.contains_rect(nr) {
                            self.multi_selected.insert(i);
                        }
                    }
                    if !self.multi_selected.is_empty() {
                        self.selected = self.multi_selected.iter().min().cloned();
                        self.canvas_selection_anchor = self.selected;
                    }
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
                let p = self.canvas_positions.get(&i).copied().unwrap_or(egui::pos2(
                    (i % default_cols) as f32 * 340.0 + 40.0,
                    (i / default_cols) as f32 * 132.0 + 40.0,
                ));
                let mm_min = to_mm(p);
                let mm_node = egui::Rect::from_min_size(
                    mm_min,
                    egui::vec2((NODE_W * mm_scale).max(2.0), (NODE_H * mm_scale).max(2.0)),
                );
                let is_sel = self.selected == Some(i) || self.multi_selected.contains(&i);
                let node_color = if is_sel {
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
                        .unwrap_or_else(|| {
                            egui::pos2(
                                (match_idx % cols) as f32 * 340.0 + 40.0,
                                (match_idx / cols) as f32 * 132.0 + 40.0,
                            )
                        });
                    let z = self.canvas_zoom;
                    let vp = resp.rect.size();
                    self.canvas_pan = egui::vec2(
                        vp.x / 2.0 / z - pos.x - 130.0,
                        vp.y / 2.0 / z - pos.y - 36.0,
                    );
                    self.select_step(match_idx);
                }
            }
            self.canvas_search = search_text;
        }

        // Zoom level indicator at bottom-left of canvas
        {
            let zoom_text = format!("{:.0}%", self.canvas_zoom * 100.0);
            let zoom_pos = egui::pos2(resp.rect.min.x + 8.0, resp.rect.max.y - 10.0);
            painter.text(
                zoom_pos,
                Align2::LEFT_BOTTOM,
                &zoom_text,
                FontId::proportional(10.0),
                Color32::from_gray(80),
            );
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
                                ("背景ドラッグ", "パン"),
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

fn layout_path(scenario_path: &std::path::Path) -> std::path::PathBuf {
    let mut p = scenario_path.to_path_buf();
    let fname = p
        .file_name()
        .map(|n| format!("{}.layout.json", n.to_string_lossy()))
        .unwrap_or_else(|| "layout.json".into());
    p.set_file_name(fname);
    p
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                const NODE_W: f32 = 260.0;
                const NODE_H: f32 = 72.0;
                let positions = &self.canvas_positions;
                let default_cols = default_canvas_cols(n);
                let pos_of = |i: usize| {
                    positions.get(&i).copied().unwrap_or_else(|| {
                        egui::pos2(
                            (i % default_cols) as f32 * 340.0 + 40.0,
                            (i / default_cols) as f32 * 132.0 + 40.0,
                        )
                    })
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
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F))
        {
            self.add_menu_open = true;
            self.add_menu_just_opened = true;
            self.add_filter.clear();
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
                    self.form_edit_buffers.clear();
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
                    self.form_edit_buffers.clear();
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
                        .unwrap_or_else(|| {
                            egui::pos2(
                                (new_sel % cols) as f32 * 340.0 + 40.0,
                                (new_sel / cols) as f32 * 132.0 + 40.0,
                            )
                        });
                    let z = self.canvas_zoom;
                    let vp = self.canvas_viewport_size;
                    self.canvas_pan = egui::vec2(
                        vp.x / 2.0 / z - pos.x - 130.0,
                        vp.y / 2.0 / z - pos.y - 36.0,
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
                        .unwrap_or_else(|| {
                            egui::pos2(
                                (new_sel % cols) as f32 * 340.0 + 40.0,
                                (new_sel / cols) as f32 * 132.0 + 40.0,
                            )
                        });
                    let z = self.canvas_zoom;
                    let vp = self.canvas_viewport_size;
                    self.canvas_pan = egui::vec2(
                        vp.x / 2.0 / z - pos.x - 130.0,
                        vp.y / 2.0 / z - pos.y - 36.0,
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

        // ── Confirm dialog (modal) ────────────────────────────────────────────
        if let Some(action) = self.confirm_dialog.clone() {
            let delete_msg_buf;
            let desc = match &action {
                ConfirmAction::OpenFile => "変更が保存されていません。破棄して開きますか？",
                ConfirmAction::NewFile => "変更が保存されていません。破棄して新規作成しますか？",
                ConfirmAction::DeleteStep(idx) => {
                    let child_count = self.steps.get(*idx).map(count_child_steps).unwrap_or(0);
                    if child_count > 0 {
                        delete_msg_buf = format!(
                            "選択中のステップを削除しますか？\n（内部に {} ステップが含まれています）",
                            child_count
                        );
                        &delete_msg_buf
                    } else {
                        "選択中のステップを削除しますか？"
                    }
                }
                ConfirmAction::DeleteSteps(count) => {
                    delete_msg_buf = format!("選択中の {} ステップをまとめて削除しますか？", count);
                    &delete_msg_buf
                }
                ConfirmAction::Quit => "変更が保存されていません。保存せずに終了しますか？",
            };
            let mut yes = false;
            let mut no = false;
            egui::Modal::new(egui::Id::new("confirm_modal")).show(ctx, |ui| {
                ui.set_min_width(240.0);
                ui.strong("確認");
                ui.add_space(4.0);
                ui.label(desc);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("はい").clicked() {
                        yes = true;
                    }
                    if ui.button("キャンセル").clicked() {
                        no = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.weak("Enter / Esc: キャンセル");
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
                            self.dirty = true;
                        }
                    }
                    ConfirmAction::DeleteSteps(_) => {
                        self.delete_selected_steps();
                    }
                    ConfirmAction::Quit => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            } else if no
                || ctx.input(|i| i.key_pressed(egui::Key::Escape))
                || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
            {
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
            self.current_run_step = None;
            if status.success() {
                self.log_ok("実行が完了しました");
            } else {
                let code = status.code().unwrap_or(-1);
                if let Some(step_idx) = last_run_step {
                    self.canvas_error_steps
                        .insert(step_idx, format!("終了コード: {code}"));
                }
                self.log_err(format!("実行に失敗しました (終了コード: {code})"));
            }
        }
        if self.last_progress_check.elapsed() > std::time::Duration::from_millis(100) {
            if let Some(ref f) = self.run_progress_file {
                if let Ok(s) = std::fs::read_to_string(f) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                        self.current_run_step = v["step"].as_u64().map(|n| n as usize);
                    }
                }
            }
            self.last_progress_check = std::time::Instant::now();
        }
        if self.run_child.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
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
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(s.menu_file, |ui| {
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
                });
                ui.menu_button(s.menu_edit, |ui| {
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
                ui.menu_button(s.menu_view, |ui| {
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
                ui.menu_button(s.menu_run_menu, |ui| {
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
                            if ui.button("Run Selection  Cmd+Shift+F5").clicked() {
                                ui.close();
                                self.run_selection();
                            }
                        });
                    }
                });
                ui.menu_button(s.menu_help, |ui| {
                    if ui
                        .selectable_label(self.manual_open, s.menu_manual)
                        .clicked()
                    {
                        self.manual_open = !self.manual_open;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(s.menu_settings).clicked() {
                        ui.close();
                        self.settings_open = true;
                    }
                    ui.separator();
                    if ui.button(s.menu_about).clicked() {
                        ui.close();
                        self.about_open = true;
                    }
                });
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
                        self.canvas_positions.clear();
                        self.ensure_canvas_layout();
                        self.save_canvas_layout();
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
                        .selectable_label(self.settings.minimap_show, "ミニマップ")
                        .on_hover_text("ミニマップの表示/非表示")
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
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Variables, s.panel_vars);
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Log, s.panel_log);
                    if self.bottom_tab == BottomTab::Log && ui.small_button(s.clear).clicked() {
                        self.log.clear();
                    }
                    let problems_label = if problem_count > 0 {
                        format!("{} ({})", s.tab_problems, problem_count)
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
                                    ui.strong("変数名");
                                    ui.strong("初期値");
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
                                ui.colored_label(
                                    LogLevel::Ok.color(),
                                    "問題は見つかりませんでした",
                                );
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
            .default_width(200.0)
            .min_width(140.0)
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
                let force = self.palette_force_open.take();
                egui::ScrollArea::vertical().show(ui, |ui| {
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
                                    let label = egui::RichText::new(t.display_name).size(11.0);
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
                });
            });

        // ── Left: step list ───────────────────────────────────────────────
        egui::SidePanel::left("steps_panel")
            .min_width(210.0)
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
                                        format!("▶ {i}: {summary}")
                                    } else {
                                        format!("{i}: {summary}")
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
                                                    let bar_color = if is_running {
                                                        egui::Color32::from_rgb(249, 226, 175)
                                                    } else {
                                                        color
                                                    };
                                                    ui.colored_label(bar_color, "▌");
                                                    let resp = ui
                                                        .selectable_label(is_primary, &label_text);
                                                    if is_multi && !is_primary && !is_running {
                                                        ui.painter().rect_filled(
                                                            resp.rect.expand2(egui::vec2(2.0, 1.0)),
                                                            2.0,
                                                            egui::Color32::from_rgba_premultiplied(
                                                                60, 100, 200, 60,
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
                                                        if ui.small_button("↑").clicked() {
                                                            action = Some(StepAction::MoveUp(i));
                                                        }
                                                    });
                                                    ui.add_enabled_ui(i + 1 < step_count, |ui| {
                                                        if ui.small_button("↓").clicked() {
                                                            action = Some(StepAction::MoveDown(i));
                                                        }
                                                    });
                                                    if ui.small_button("✕").clicked() {
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
                                    Self::canvas_shift_positions(
                                        &mut self.canvas_positions,
                                        from,
                                        -1,
                                    );
                                    Self::canvas_shift_positions(&mut self.canvas_positions, to, 1);
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
                                    // Reset canvas positions to be repopulated by ensure_canvas_layout
                                    self.canvas_positions.clear();
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
                self.show_canvas(ui);
                return;
            }

            if let Some(idx) = self.selected {
                // Inline parse error banner
                if let Some(ref err) = self.parse_error.clone() {
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
                        if ui.button("キャンセル").clicked() {
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
                            self.select_step(idx);
                            self.log_info("ステップを追加しました");
                        }
                    }
                    Err(e) => self.log_err(format!("テンプレートエラー: {e}")),
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
    }
}

enum StepAction {
    Select(usize),
    ToggleMulti(usize),
    ShiftSelect(usize),
    MoveUp(usize),
    MoveDown(usize),
    Delete(usize),
}

// ---- main -----------------------------------------------------------------

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates: &[&str] = &[
        // macOS
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        // Windows
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
        "C:\\Windows\\Fonts\\YuGothR.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJKjp-Regular.otf",
    ];

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("cjk".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

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
        "robost-editor",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            setup_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(EditorApp::default()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}
