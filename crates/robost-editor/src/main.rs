use anyhow::Result;
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            provider: AiProvider::Anthropic,
            api_key: String::new(),
            model: "claude-haiku-4-5-20251001".to_owned(),
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

fn load_settings() -> AppSettings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(s: &AppSettings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string(s) {
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
    DeleteStep(usize),
    Quit,
}

// ---- color palette (ajisai-inspired) --------------------------------------

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

#[derive(PartialEq, Clone, Copy, Default)]
enum ViewMode {
    #[default]
    List,
    Flow,
}

#[derive(PartialEq, Clone, Copy, Default)]
enum PropView {
    #[default]
    Form,
    Yaml,
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

// ---- step templates -------------------------------------------------------

struct StepTemplate {
    category: &'static str,
    display_name: &'static str,
    name: &'static str,
    yaml: &'static str,
}

const STEP_TEMPLATES: &[StepTemplate] = &[
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

fn parse_scenario_steps(text: &str) -> Result<(String, Vec<serde_yml::Value>)> {
    let doc: serde_yml::Value = serde_yml::from_str(text)?;
    let name = doc
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed")
        .to_owned();
    let steps = doc
        .get("steps")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();
    Ok((name, steps))
}

fn build_scenario_yaml(name: &str, steps: &[serde_yml::Value]) -> Result<String> {
    let mut map = serde_yml::Mapping::new();
    map.insert(
        serde_yml::Value::String("name".into()),
        serde_yml::Value::String(name.into()),
    );
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
        .set("x-api-key", &settings.api_key)
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
        .set("Authorization", &format!("Bearer {}", settings.api_key))
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
                .set("x-api-key", &settings.api_key)
                .set("anthropic-version", "2023-06-01")
                .set("content-type", "application/json")
                .send_json(body)
                .map(|_| "接続成功".to_owned())
                .map_err(|e| format!("{e}"))
        }
        AiProvider::OpenAI => {
            let body = serde_json::json!({
                "model": settings.model,
                "max_tokens": 8,
                "messages": [{ "role": "user", "content": "ping" }],
            });
            ureq::post("https://api.openai.com/v1/chat/completions")
                .set("Authorization", &format!("Bearer {}", settings.api_key))
                .set("content-type", "application/json")
                .send_json(body)
                .map(|_| "接続成功".to_owned())
                .map_err(|e| format!("{e}"))
        }
    };
    match result {
        Ok(msg) => (true, msg),
        Err(e) => (false, e),
    }
}

// ---- undo/redo state snapshot ---------------------------------------------

#[derive(Clone)]
struct EditorState {
    name: String,
    steps: Vec<serde_yml::Value>,
    selected: Option<usize>,
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
    log: Vec<LogEntry>,
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
            log: Vec::new(),
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
    fn log_ok(&mut self, msg: impl Into<String>) {
        self.push_log(msg, LogLevel::Ok);
    }
    fn log_err(&mut self, msg: impl Into<String>) {
        self.push_log(msg, LogLevel::Error);
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
                    Ok((name, steps)) => {
                        self.name = name;
                        self.steps = steps;
                        self.selected = None;
                        self.edit_buf.clear();
                        self.path = Some(p.clone());
                        self.dirty = false;
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.form_edit_buffers.clear();
                        self.log_ok(format!("開きました: {}", p.display()));
                    }
                    Err(e) => self.log_err(format!("構文エラー: {e}")),
                },
                Err(e) => self.log_err(format!("読み込みエラー: {e}")),
            }
        }
    }

    fn write_scenario_to_path(&mut self, path: PathBuf) {
        match build_scenario_yaml(&self.name, &self.steps) {
            Ok(text) => match std::fs::write(&path, &text) {
                Ok(()) => {
                    self.path = Some(path.clone());
                    self.dirty = false;
                    self.log_ok(format!("保存しました: {}", path.display()));
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
        self.push_undo();
        self.flush_edit();
        self.selected = Some(idx);
        self.selected_child = None;
        self.child_edit_buf.clear();
        if let Some(step) = self.steps.get(idx) {
            self.edit_buf = serde_yml::to_string(step).unwrap_or_default();
            self.parse_error = None;
        }
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
        }
    }

    fn restore(&mut self, state: EditorState) {
        self.name = state.name;
        self.steps = state.steps;
        self.selected = state.selected;
        self.selected_child = None;
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
        let snap = self.snapshot();
        let changed = self
            .undo_stack
            .back()
            .map(|s| s.steps != snap.steps || s.name != snap.name)
            .unwrap_or(true);
        if changed {
            self.undo_stack.push_back(snap);
            if self.undo_stack.len() > 50 {
                self.undo_stack.pop_front();
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
                                        ui.horizontal(|ui| {
                                            ui.colored_label(col, "▌");
                                            ui.strong(format!(
                                                "ステップ {} ({}) > {} [{}]  —  {}",
                                                idx + 1, outer_key, branch, child_idx, step_display_name(ck)
                                            ));
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
            ui.centered_and_justified(|ui| {
                ui.label("ステップがありません。左パネルで追加してください。");
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
                        }
                    }
                    other => self.steps.insert(at, other),
                }
                self.dirty = true;
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
        if !self.ai_panel_open {
            return;
        }
        self.ai_unread = false;
        egui::SidePanel::right("ai_panel")
            .min_width(360.0)
            .max_width(480.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("AI アシスタント");
                    if ui.small_button("✕").clicked() {
                        self.ai_panel_open = false;
                    }
                    if ui.small_button("🗑 クリア").clicked() {
                        self.ai_messages.clear();
                        self.md_cache = egui_commonmark::CommonMarkCache::default();
                    }
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
                            .desired_width(ui.available_width() - 55.0),
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
        let mut open = self.settings_open;
        egui::Window::new("設定")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_size([380.0, 220.0])
            .show(ctx, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("プロバイダー:");
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

                        ui.label("APIキー:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.settings.api_key)
                                .password(true)
                                .desired_width(240.0),
                        );
                        ui.end_row();

                        ui.label("モデル:");
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
                    if ui.button("保存").clicked() {
                        save_settings(&self.settings);
                        self.settings_open = false;
                    }
                    let testing = self.settings_test_rx.is_some();
                    if ui
                        .add_enabled(
                            !testing && !self.settings.api_key.is_empty(),
                            egui::Button::new("接続テスト"),
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

                // Category chips
                ui.horizontal_wrapped(|ui| {
                    let mut seen = std::collections::HashSet::new();
                    for t in STEP_TEMPLATES {
                        if seen.insert(t.category) {
                            let col = category_color(t.category);
                            if ui
                                .add(
                                    egui::Button::new(egui::RichText::new(t.category).size(10.0))
                                        .fill(egui::Color32::from_rgba_unmultiplied(
                                            col.r(),
                                            col.g(),
                                            col.b(),
                                            35,
                                        ))
                                        .min_size(egui::vec2(0.0, 18.0)),
                                )
                                .clicked()
                            {
                                self.manual_search = t.category.to_owned();
                            }
                        }
                    }
                });
                ui.separator();

                let filter = self.manual_search.to_lowercase();
                egui::ScrollArea::vertical()
                    .max_height(440.0)
                    .show(ui, |ui| {
                        let mut last_cat = "";
                        for t in STEP_TEMPLATES {
                            let match_filter = filter.is_empty()
                                || t.name.to_lowercase().contains(&filter)
                                || t.display_name.to_lowercase().contains(&filter)
                                || t.category.to_lowercase().contains(&filter);
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
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        if !self.add_menu_open
            && self.confirm_dialog.is_none()
            && self.prop_view != PropView::Yaml
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete))
        {
            if let Some(idx) = self.selected {
                if idx < self.steps.len() {
                    self.confirm_dialog = Some(ConfirmAction::DeleteStep(idx));
                }
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.add_menu_open = false;
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

        // ── Confirm dialog (modal) ────────────────────────────────────────────
        if let Some(action) = self.confirm_dialog.clone() {
            let delete_msg_buf;
            let desc = match &action {
                ConfirmAction::OpenFile => "変更が保存されていません。破棄して開きますか？",
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
                    ConfirmAction::DeleteStep(idx) => {
                        if idx < self.steps.len() {
                            self.push_undo();
                            self.steps.remove(idx);
                            self.selected = None;
                            self.edit_buf.clear();
                            self.parse_error = None;
                            let suffix = format!("@{idx}");
                            self.form_edit_buffers.retain(|k, _| !k.ends_with(&suffix));
                        }
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
        let child_exited = if let Some(ref mut child) = self.run_child {
            matches!(child.try_wait(), Ok(Some(_)))
        } else {
            false
        };
        if child_exited {
            self.run_child = None;
            self.run_progress_file = None;
            self.log_info("実行が完了しました");
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

        // ── Settings test-connection polling ──────────────────────────────────
        if let Some(ref rx) = self.settings_test_rx {
            if let Ok(result) = rx.try_recv() {
                self.settings_test_result = Some(result);
                self.settings_test_rx = None;
                ctx.request_repaint();
            }
        }

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
                ui.label("シナリオ名:");
                if ui.text_edit_singleline(&mut self.name).changed() {
                    self.dirty = true;
                }
                ui.separator();
                if self.run_child.is_some() {
                    if ui
                        .button("⏹ 停止")
                        .on_hover_text("実行を停止 (F5)")
                        .clicked()
                    {
                        self.stop_run();
                    }
                } else if ui.button("▶ 実行").on_hover_text("実行 (F5)").clicked() {
                    self.run_scenario();
                }
                ui.separator();
                if ui
                    .selectable_value(&mut self.view_mode, ViewMode::List, "リスト")
                    .clicked()
                {
                    // nothing extra
                }
                if ui
                    .selectable_value(&mut self.view_mode, ViewMode::Flow, "フロー")
                    .clicked()
                {
                    self.scroll_to_selected = true;
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
                ui.separator();
                {
                    let ai_label = if self.ai_unread {
                        egui::RichText::new("AI ●").color(egui::Color32::from_rgb(240, 120, 40))
                    } else {
                        egui::RichText::new("AI")
                    };
                    let hover = if self.ai_unread {
                        "AIアシスタント (未読メッセージあり)"
                    } else {
                        "AIアシスタント"
                    };
                    if ui.button(ai_label).on_hover_text(hover).clicked() {
                        self.ai_panel_open = !self.ai_panel_open;
                        if self.ai_panel_open {
                            self.ai_unread = false;
                        }
                    }
                }
                if ui.button("設定").clicked() {
                    self.settings_open = true;
                }
                if ui.button("? マニュアル").clicked() {
                    self.manual_open = !self.manual_open;
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

        // ── Log panel (bottom, resizable) ─────────────────────────────────
        egui::TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .min_height(60.0)
            .default_height(130.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("ログ");
                    if ui.small_button("クリア").clicked() {
                        self.log.clear();
                    }
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
        egui::SidePanel::left("steps_panel")
            .min_width(210.0)
            .show(ctx, |ui| {
                ui.heading("ステップ一覧");
                ui.separator();

                let mut action: Option<StepAction> = None;
                let step_count = self.steps.len();

                egui::ScrollArea::vertical()
                    .id_salt("step_list")
                    .show(ui, |ui| {
                        for i in 0..step_count {
                            let key = get_step_key(&self.steps[i]);
                            let cat = step_key_category(key);
                            let color = category_color(cat);
                            let summary = step_summary(&self.steps[i]);
                            let selected = self.selected == Some(i);
                            let is_running = self.current_run_step == Some(i);

                            ui.horizontal(|ui| {
                                let bar_color = if is_running {
                                    egui::Color32::from_rgb(249, 226, 175)
                                } else {
                                    color
                                };
                                ui.colored_label(bar_color, "▌");
                                let label = if is_running {
                                    format!("▶ {i}: {summary}")
                                } else {
                                    format!("{i}: {summary}")
                                };
                                if ui.selectable_label(selected, label).clicked() {
                                    action = Some(StepAction::Select(i));
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
                            });
                        }
                    });

                ui.separator();
                if ui.button("+ ステップ追加").clicked() {
                    self.add_menu_open = true;
                    self.add_menu_just_opened = true;
                    self.add_filter.clear();
                }

                if let Some(act) = action {
                    match act {
                        StepAction::Select(i) => self.select_step(i),
                        StepAction::MoveUp(i) => {
                            self.push_undo();
                            self.steps.swap(i - 1, i);
                            if self.selected == Some(i) {
                                self.selected = Some(i - 1);
                            }
                            self.form_edit_buffers.clear();
                        }
                        StepAction::MoveDown(i) => {
                            self.push_undo();
                            self.steps.swap(i, i + 1);
                            if self.selected == Some(i) {
                                self.selected = Some(i + 1);
                            }
                            self.form_edit_buffers.clear();
                        }
                        StepAction::Delete(i) => {
                            self.confirm_dialog = Some(ConfirmAction::DeleteStep(i));
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
                    ui.heading("シナリオを作成しましょう");
                    ui.add_space(20.0);
                    ui.label("① 上部の「シナリオ名」を入力してください");
                    ui.add_space(8.0);
                    ui.label("② 左パネルの「+ ステップ追加」でステップを選んでください");
                    ui.add_space(8.0);
                    ui.label("③ ステップを選択するとフォームまたは YAML で内容を編集できます");
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
            let mut close = false;
            let mut insert: Option<&'static str> = None;

            egui::Window::new("ステップを追加")
                .collapsible(false)
                .resizable(true)
                .default_size([340.0, 420.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("絞り込み:");
                        let filter_resp = ui.text_edit_singleline(&mut self.add_filter);
                        if self.add_menu_just_opened {
                            filter_resp.request_focus();
                            self.add_menu_just_opened = false;
                        }
                    });
                    // Category jump chips
                    ui.horizontal_wrapped(|ui| {
                        let mut seen = std::collections::HashSet::new();
                        for t in STEP_TEMPLATES {
                            if seen.insert(t.category) {
                                let col = category_color(t.category);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(t.category).size(10.0),
                                        )
                                        .fill(egui::Color32::from_rgba_unmultiplied(
                                            col.r(),
                                            col.g(),
                                            col.b(),
                                            35,
                                        ))
                                        .min_size(egui::vec2(0.0, 18.0)),
                                    )
                                    .clicked()
                                {
                                    self.add_filter = t.category.to_owned();
                                }
                            }
                        }
                    });
                    ui.separator();

                    let filter = self.add_filter.to_lowercase();
                    let first_match = STEP_TEMPLATES.iter().find(|t| {
                        filter.is_empty()
                            || t.name.to_lowercase().contains(&filter)
                            || t.display_name.to_lowercase().contains(&filter)
                            || t.category.to_lowercase().contains(&filter)
                    });

                    egui::ScrollArea::vertical()
                        .max_height(340.0)
                        .show(ui, |ui| {
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
                                    ui.colored_label(
                                        col,
                                        egui::RichText::new(t.category).strong().size(12.0),
                                    );
                                    ui.separator();
                                    last_cat = t.category;
                                }

                                let col = category_color(t.category);
                                let label_text = format!("  {} ({})", t.display_name, t.name);
                                let btn =
                                    egui::Button::new(egui::RichText::new(label_text).size(12.0))
                                        .min_size(egui::vec2(250.0, 26.0));

                                // Subtle tint matching category color
                                let btn = btn.fill(egui::Color32::from_rgba_unmultiplied(
                                    col.r(),
                                    col.g(),
                                    col.b(),
                                    18,
                                ));

                                if ui.add(btn).clicked() {
                                    insert = Some(t.yaml);
                                    close = true;
                                }
                            }
                        });

                    // Enter key inserts the first visible match
                    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                        if let Some(t) = first_match {
                            insert = Some(t.yaml);
                            close = true;
                        }
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("キャンセル").clicked() {
                            close = true;
                        }
                        ui.weak("Esc でキャンセル / Enter で最初の候補を挿入");
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

        // ── Settings window ───────────────────────────────────────────────
        self.show_settings_window(ctx);

        // ── Manual window ─────────────────────────────────────────────────
        self.show_manual_window(ctx);
    }
}

enum StepAction {
    Select(usize),
    MoveUp(usize),
    MoveDown(usize),
    Delete(usize),
}

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
        "robost-editor",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(EditorApp::default()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}
