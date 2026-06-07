// ---- shared types -----------------------------------------------------------

use crate::tokens;

// ---- AI message -------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct AiMessage {
    pub(crate) role: String,
    pub(crate) content: String,
    pub(crate) yaml_blocks: Vec<String>,
}

#[derive(Clone)]
pub(crate) enum ConfirmAction {
    OpenFile,
    NewFile,
    DeleteStep(usize),
    DeleteSteps(usize),
    Quit,
}

// ---- color palette (ajisai-inspired) ----------------------------------------

pub(crate) const COL_AI: egui::Color32 = egui::Color32::from_rgb(180, 80, 220);
pub(crate) const COL_IMG: egui::Color32 = egui::Color32::from_rgb(70, 130, 200);
pub(crate) const COL_FLOW: egui::Color32 = egui::Color32::from_rgb(200, 140, 50);
pub(crate) const COL_INPUT: egui::Color32 = egui::Color32::from_rgb(100, 200, 120);
pub(crate) const COL_DLG: egui::Color32 = egui::Color32::from_rgb(180, 100, 200);
pub(crate) const COL_VAR: egui::Color32 = egui::Color32::from_rgb(220, 200, 80);
pub(crate) const COL_WAIT: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);
pub(crate) const COL_SCR: egui::Color32 = egui::Color32::from_rgb(220, 100, 100);
pub(crate) const COL_CLIP: egui::Color32 = egui::Color32::from_rgb(100, 200, 220);
pub(crate) const COL_LIB: egui::Color32 = egui::Color32::from_rgb(0x7B, 0x68, 0xEE);
pub(crate) const COL_DATA: egui::Color32 = egui::Color32::from_rgb(249, 200, 120);
pub(crate) const COL_FILE: egui::Color32 = egui::Color32::from_rgb(180, 140, 240);
pub(crate) const COL_EXCEL: egui::Color32 = egui::Color32::from_rgb(33, 160, 80);
pub(crate) const COL_STR: egui::Color32 = egui::Color32::from_rgb(80, 180, 200);
pub(crate) const COL_DATE: egui::Color32 = egui::Color32::from_rgb(230, 160, 30);
pub(crate) const COL_JSON: egui::Color32 = egui::Color32::from_rgb(200, 180, 50);
pub(crate) const COL_PATH: egui::Color32 = egui::Color32::from_rgb(160, 150, 230);
pub(crate) const COL_MOUSE: egui::Color32 = egui::Color32::from_rgb(50, 180, 160);
pub(crate) const COL_PROC: egui::Color32 = egui::Color32::from_rgb(220, 100, 50);
pub(crate) const COL_HTTP: egui::Color32 = egui::Color32::from_rgb(50, 100, 220);
pub(crate) const COL_MAIL: egui::Color32 = egui::Color32::from_rgb(30, 150, 160);
pub(crate) const COL_WEB: egui::Color32 = egui::Color32::from_rgb(100, 190, 60);
pub(crate) const COL_UIA: egui::Color32 = egui::Color32::from_rgb(140, 70, 200);
pub(crate) const COL_CSV: egui::Color32 = egui::Color32::from_rgb(100, 170, 100);
pub(crate) const COL_LIST: egui::Color32 = egui::Color32::from_rgb(220, 160, 50);
pub(crate) const COL_UTIL: egui::Color32 = egui::Color32::from_rgb(130, 150, 200);

pub(crate) fn category_color(category: &str) -> egui::Color32 {
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

pub(crate) fn step_key_category(key: &str) -> &str {
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

// ---- nodes panel tab --------------------------------------------------------

#[derive(Clone, PartialEq, Default)]
pub(crate) enum NodesPanelTab {
    #[default]
    Nodes,
    Templates,
}

// ---- activity bar / sidebar tab ---------------------------------------------

/// Which content is shown in the sidebar, driven by Activity Bar icon clicks.
#[derive(Clone, PartialEq, Copy, Default)]
pub(crate) enum SidebarTab {
    #[default]
    Steps, // Current scenario step list
    Nodes,     // Step template palette (add new steps)
    Templates, // PNG template gallery
}

// ---- view mode / flowchart types --------------------------------------------

#[derive(Clone)]
pub(crate) enum CanvasContextAction {
    Delete(usize),
    Duplicate(usize),
    OpenInList(usize),
    RunFrom(usize),
    CopySelected,
    CutSelected,
    ToggleEnabled(usize),
    Paste,
    AlignLeft,
    AlignTop,
    DistributeH,
    DistributeV,
    SelectAll,
    AddComment(egui::Pos2),
    CanvasReset,
    CanvasFit,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub(crate) enum ViewMode {
    #[default]
    List,
    Flow,
    Canvas,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub(crate) enum PropView {
    #[default]
    Form,
    Yaml,
}

#[derive(PartialEq, Default)]
pub(crate) enum BottomTab {
    Variables,
    #[default]
    Log,
    Problems,
}

pub(crate) struct ValidationIssue {
    pub(crate) step_idx: usize,
    pub(crate) message: String,
    pub(crate) level: LogLevel,
}

pub(crate) struct FlowNode {
    pub(crate) step_idx: usize,
    pub(crate) depth: usize,
    pub(crate) label: String,
    pub(crate) color: egui::Color32,
    pub(crate) expand_key: Option<usize>,
    pub(crate) is_expanded: bool,
    pub(crate) is_header: bool,
}

// ---- log --------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LogLevel {
    Info,
    Ok,
    Warning,
    Error,
}

impl LogLevel {
    pub(crate) fn color(self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::LIGHT_GRAY,
            LogLevel::Ok => tokens::SUCCESS,
            LogLevel::Warning => tokens::WARNING,
            LogLevel::Error => tokens::ERROR,
        }
    }
}

pub(crate) struct LogEntry {
    pub(crate) message: String,
    pub(crate) level: LogLevel,
}

pub(crate) struct Toast {
    pub(crate) message: String,
    pub(crate) level: LogLevel,
    pub(crate) expires: std::time::Instant,
}

// ---- undo/redo state snapshot -----------------------------------------------

#[derive(Clone)]
pub(crate) struct EditorState {
    pub(crate) name: String,
    pub(crate) steps: Vec<serde_yml::Value>,
    pub(crate) scenario_vars: serde_yml::Mapping,
    pub(crate) selected: Option<usize>,
    pub(crate) selected_child: Option<(String, usize)>,
    pub(crate) canvas_positions: Vec<(usize, [f32; 2])>,
    pub(crate) canvas_zoom: f32,
    pub(crate) canvas_pan: [f32; 2],
    pub(crate) multi_selected: Vec<usize>,
    pub(crate) expanded_steps: Vec<usize>,
    /// Human-readable description of the action that produced this snapshot.
    /// Shown in the undo button tooltip and status bar.
    pub(crate) action_name: String,
}

// ---- canvas comment (sticky note) -------------------------------------------

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanvasComment {
    pub(crate) id: u64,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
    pub(crate) text: String,
    /// RGBA color packed as [r, g, b, a]
    pub(crate) color: [u8; 4],
}

// ---- drag-and-drop payload --------------------------------------------------

#[derive(Clone)]
pub(crate) enum DragPayload {
    /// YAML snippet from a StepTemplate — insert as a new step.
    NewStep(&'static str),
    /// Indices of existing steps being reordered (multi-select aware).
    ReorderStep(Vec<usize>),
}

// ---- step list action -------------------------------------------------------

pub(crate) enum StepAction {
    Select(usize),
    ToggleMulti(usize),
    ShiftSelect(usize),
    MoveUp(usize),
    MoveDown(usize),
    Delete(usize),
    ToggleEnabled(usize),
}
