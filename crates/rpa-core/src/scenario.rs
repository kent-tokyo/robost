use rpa_template::Target;
use serde::{Deserialize, Serialize};

/// Specifies an external data source (CSV or XLSX) that is loaded before execution.
/// Each row becomes one iteration of `foreach: __rows__`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Path to the CSV or XLSX file (relative to the scenario file).
    pub file: String,
    /// Sheet name for XLSX files (ignored for CSV). Defaults to first sheet.
    #[serde(default)]
    pub sheet: Option<String>,
}

/// Top-level scenario loaded from a YAML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub target: Option<Target>,
    pub steps: Vec<ScenarioStep>,
    #[serde(default)]
    pub variables: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub data_source: Option<DataSource>,
    /// How long (ms) to wait for the target window to reappear after an RDP/VNC reconnect.
    /// 0 or absent disables the reconnect retry. Applied per capture-bound step.
    #[serde(default)]
    pub reconnect_timeout_ms: Option<u64>,
}

/// A single step inside a scenario or sub-scenario.
///
/// The YAML format uses `{variant_name: struct_value}` maps.
/// Unit variants (`break`, `continue`, `exit`) may be written as bare strings
/// or as `{break: null}` maps.
///
/// Serialization uses serde's default externally-tagged YAML format (YAML tags).
/// Custom Deserialize parses the user-friendly `{variant: value}` map format.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioStep {
    // --- basic actions ---
    WaitImage(WaitImageStep),
    ClickImage(ClickImageStep),
    Type(TypeStep),
    Press(String),
    Library(LibraryStep),
    Script(ScriptStep),
    Foreach(ForeachStep),
    SubScenario(SubScenarioStep),
    Set(SetStep),
    WaitMs(u64),

    // --- flow control ---
    Group(GroupStep),
    If(IfStep),
    Switch(SwitchStep),
    Repeat(RepeatStep),
    While(WhileStep),
    DoWhile(DoWhileStep),
    TryCatch(TryCatchStep),
    Break,
    Continue,
    CallScenario(CallScenarioStep),
    Exit,

    // --- additional action nodes ---
    FindImage(FindImageStep),
    Shell(ShellStep),
    ClipboardSet(ClipboardSetStep),
    ClipboardGet(ClipboardGetStep),

    // --- variable nodes ---
    CopyVar(CopyVarStep),
    GetDatetime(GetDatetimeStep),
    GetUsername(GetUsernameStep),
    Calc(CalcStep),
    Increment(IncrementStep),
    ToFullwidth(WidthStep),
    ToHalfwidth(WidthStep),

    // --- user interaction ---
    DialogWait(DialogWaitStep),
    DialogInput(DialogInputStep),
    DialogSelect(DialogSelectStep),

    // --- window / region nodes ---
    WaitWindow(WaitWindowStep),
    MatchRect(MatchRectStep),
    WindowControl(WindowControlStep),

    // --- OCR ---
    OcrMatch(OcrMatchStep),

    // --- ML detection ---
    MlDetect(MlDetectStep),

    // --- variable persistence ---
    ImportVars(ImportVarsStep),
    SaveVars(SaveVarsStep),
    LoadVars(LoadVarsStep),

    // --- file operations ---
    FileExists(FileExistsStep),
    FileCopy(FileCopyStep),
    FileMove(FileMoveStep),
    FileDelete(FileDeleteStep),
    FileRename(FileRenameStep),

    // --- logging ---
    LogWrite(LogWriteStep),

    // --- date operations ---
    DateFormat(DateFormatStep),
    DateAdd(DateAddStep),
    DateDiff(DateDiffStep),

    // --- string operations ---
    StringReplace(StringReplaceStep),
    StringTrim(StringTrimStep),
    StringUpper(StringCaseStep),
    StringLower(StringCaseStep),
    StringSubstring(StringSubstringStep),
    StringLength(StringLengthStep),
    StringSplit(StringSplitStep),
    StringJoin(StringJoinStep),
    StringRegex(StringRegexStep),

    // --- json helpers ---
    JsonParse(JsonParseStep),
    JsonStringify(JsonStringifyStep),

    // --- path helpers ---
    PathJoin(PathJoinStep),
    PathBasename(PathBasenameStep),
    PathDirname(PathDirnameStep),

    // --- env / misc ---
    EnvGet(EnvGetStep),
    FileList(FileListStep),

    // --- mouse coordinate nodes ---
    MouseMove(MouseMoveStep),
    MouseClickXy(MouseClickXyStep),
    MouseDrag(MouseDragStep),
    MouseScroll(MouseScrollStep),

    // --- HTTP client nodes ---
    #[cfg(feature = "http")]
    HttpGet(HttpGetStep),
    #[cfg(feature = "http")]
    HttpPost(HttpPostStep),
    #[cfg(feature = "http")]
    HttpPut(HttpPutStep),
    #[cfg(feature = "http")]
    HttpDelete(HttpDeleteStep),
    #[cfg(feature = "http")]
    HttpPatch(HttpPatchStep),

    // --- mail ---
    MailReceive(MailReceiveStep),

    // --- Excel cell nodes ---
    ExcelReadCell(ExcelReadCellStep),
    ExcelReadRange(ExcelReadRangeStep),
    ExcelWriteCell(ExcelWriteCellStep),

    // --- Excel sheet management nodes ---
    ExcelAddSheet(ExcelAddSheetStep),
    ExcelDeleteSheet(ExcelDeleteSheetStep),
    ExcelRenameSheet(ExcelRenameSheetStep),

    // --- text file read/write ---
    FileRead(FileReadStep),
    FileWrite(FileWriteStep),
    FileAppend(FileAppendStep),

    // --- process operations ---
    ProcessStart(ProcessStartStep),
    ProcessKill(ProcessKillStep),
    ProcessExists(ProcessExistsStep),

    // --- key combination ---
    KeyCombo(KeyComboStep),

    // --- CSV read/write ---
    CsvRead(CsvReadStep),
    CsvWrite(CsvWriteStep),

    // --- screenshot / observation ---
    ScreenshotSave(ScreenshotSaveStep),
    WaitNoImage(WaitNoImageStep),

    // --- system integration ---
    UrlOpen(UrlOpenStep),
    Notify(NotifyStep),

    // --- UI Automation (Windows) ---
    /// Get a UIA element property (name, value, class).
    UiaGet(UiaGetStep),
    /// Set a UIA element's value.
    UiaSet(UiaSetStep),
    /// Invoke (click) a UIA element.
    UiaClick(UiaClickStep),
    /// Find a UIA element and store its bounding rect.
    UiaFind(UiaFindStep),

    // --- Web browser automation ---
    /// Open a URL in a WebDriver-controlled browser.
    WebOpen(WebOpenStep),
    /// Click an element by CSS selector.
    WebClick(WebClickStep),
    /// Type text into an element.
    WebType(WebTypeStep),
    /// Get text or attribute from an element.
    WebGet(WebGetStep),
    /// Wait for an element to appear.
    WebWait(WebWaitStep),
    /// Save a browser screenshot.
    WebScreenshot(WebScreenshotStep),
    /// Close the browser session.
    WebClose,
}

// ── existing step types ────────────────────────────────────────────────────

/// Which mouse button / gesture to use when clicking a matched image.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClickAction {
    #[default]
    Left,
    Right,
    Double,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitImageStep {
    pub template: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Mask regions (template-local coords) excluded from NCC matching.
    #[serde(default)]
    pub masks: Vec<rpa_template::MaskRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickImageStep {
    pub template: String,
    pub anchor: Option<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub action: ClickAction,
    /// Pixel offset from the matched centre (positive = right / down).
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Mask regions excluded from NCC matching.
    #[serde(default)]
    pub masks: Vec<rpa_template::MaskRegion>,
    /// Milliseconds to wait after a successful click (for slow UI animations).
    #[serde(default)]
    pub post_click_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeStep {
    Plain(String),
    SecretEnv { secret_env: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStep {
    /// Fully-qualified library function name, e.g. `"excel-reader.read_sheet"`.
    pub name: String,
    #[serde(default)]
    pub inputs: std::collections::HashMap<String, serde_json::Value>,
    pub save_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStep {
    pub script: String,
    pub save_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeachStep {
    /// Name of the list variable to iterate over.
    pub var: String,
    #[serde(rename = "do")]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubScenarioStep {
    pub path: String,
    #[serde(default)]
    pub inputs: std::collections::HashMap<String, serde_json::Value>,
    pub save_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStep {
    pub name: String,
    pub value: serde_json::Value,
}

// ── flow control step types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStep {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "do")]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStep {
    /// Rhai boolean expression, e.g. `"count > 10"`.
    pub cond: String,
    pub then: Vec<ScenarioStep>,
    #[serde(default, rename = "else")]
    pub else_steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchStep {
    /// Variable name to switch on.
    pub on: String,
    pub cases: Vec<SwitchCase>,
    #[serde(default)]
    pub default: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCase {
    pub when: serde_json::Value,
    #[serde(rename = "do")]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatStep {
    pub count: u64,
    #[serde(rename = "do")]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileStep {
    /// Rhai boolean expression evaluated before each iteration.
    pub cond: String,
    #[serde(rename = "do")]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoWhileStep {
    /// Rhai boolean expression evaluated after each iteration.
    pub cond: String,
    #[serde(rename = "do")]
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryCatchStep {
    #[serde(rename = "try")]
    pub try_steps: Vec<ScenarioStep>,
    #[serde(default)]
    pub catch: Vec<ScenarioStep>,
    #[serde(default)]
    pub finally: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallScenarioStep {
    pub path: String,
    #[serde(default)]
    pub inputs: std::collections::HashMap<String, serde_json::Value>,
    pub save_as: Option<String>,
}

// ── additional action step types ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindImageStep {
    pub template: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    pub save_as: Option<String>,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellStep {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub save_as: Option<String>,
    #[serde(default = "default_shell_timeout_ms")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSetStep {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardGetStep {
    pub save_as: String,
}

// ── variable node step types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyVarStep {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDatetimeStep {
    #[serde(default = "default_datetime_format")]
    pub format: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsernameStep {
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcStep {
    /// Rhai arithmetic expression, e.g. `"a + b * 2"`.
    pub expr: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementStep {
    pub name: String,
    #[serde(default = "default_increment_by")]
    pub by: i64,
}

/// Shared by `to_fullwidth` and `to_halfwidth`; direction determined by the step variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidthStep {
    pub value: String,
    pub save_as: String,
}

// ── window / region step types ────────────────────────────────────────────

/// Which window existence state to wait for.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowState {
    /// Wait until a window with matching title appears.
    #[default]
    Exists,
    /// Wait until no window with matching title is present.
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitWindowStep {
    pub title_contains: String,
    #[serde(default)]
    pub state: WindowState,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// If provided, stores `true` (condition met) or `false` (timed out) instead of erroring.
    pub save_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRectStep {
    pub template: String,
    /// Screen-global rectangle to search within.
    pub rect: rpa_template::Rect,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// Stores `{found, x, y, score}`. If absent and not found, returns an error.
    pub save_as: Option<String>,
}

// ── user interaction step types ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogWaitStep {
    pub message: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogInputStep {
    pub message: String,
    pub title: Option<String>,
    pub default: Option<String>,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogSelectStep {
    pub message: String,
    pub title: Option<String>,
    pub options: Vec<String>,
    pub save_as: String,
}

// ── Window control step type ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowControlAction {
    /// Bring the window to the foreground.
    #[default]
    Focus,
    /// Maximize the window.
    Maximize,
    /// Minimize the window to the taskbar.
    Minimize,
    /// Close the window.
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowControlStep {
    pub title_contains: String,
    #[serde(default)]
    pub action: WindowControlAction,
}

// ── OCR step type ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrMatchStep {
    /// Restrict OCR to this screen-global rectangle. `None` = full capture area.
    #[serde(default)]
    pub region: Option<rpa_template::Rect>,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Text substring the OCR result must contain. `None` = any text passes.
    #[serde(default)]
    pub contains: Option<String>,
    /// Tesseract language code(s): `"eng"`, `"jpn"`, `"jpn+eng"`, etc.
    #[serde(default = "default_ocr_lang")]
    pub lang: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// Stores `{found: bool, text: str}`. If absent and text doesn't match, returns an error.
    pub save_as: Option<String>,
}

// ── ML detection step type ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlDetectStep {
    /// Path to the ONNX model file (relative to the scenario directory).
    pub model: String,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Minimum confidence score to report a detection, in [0, 1].
    #[serde(default = "default_ml_threshold")]
    pub threshold: f32,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Stores `[{label, score, x, y, width, height}, ...]`.
    /// Requires the `ml` feature; rebuild with `cargo build --features ml`.
    pub save_as: Option<String>,
}

// ── variable persistence step types ──────────────────────────────────────

/// Import variables from a CSV or XLSX file.
/// The first row is the header (variable names); the row at `row` index (0-based)
/// provides the values. Each column becomes one variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportVarsStep {
    /// Path to the CSV or XLSX file (relative to the scenario file).
    pub file: String,
    /// Sheet name for XLSX (ignored for CSV). Defaults to the first sheet.
    #[serde(default)]
    pub sheet: Option<String>,
    /// Which data row to use (0-based, after the header row). Default: 0.
    #[serde(default)]
    pub row: usize,
    /// Optional prefix prepended to each variable name.
    #[serde(default)]
    pub prefix: String,
}

/// Persist current variables to a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveVarsStep {
    /// Destination path (relative to the scenario file).
    pub file: String,
    /// If non-empty, save only these variable names. Default: save all.
    #[serde(default)]
    pub vars: Vec<String>,
}

/// Load variables from a JSON file previously written by `save_vars`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadVarsStep {
    /// Source path (relative to the scenario file).
    pub file: String,
    /// Optional prefix prepended to each loaded variable name.
    #[serde(default)]
    pub prefix: String,
}

// ── file operation step types ─────────────────────────────────────────────

/// Check whether a file or directory exists; store result as bool in `save_as`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExistsStep {
    pub path: String,
    pub save_as: String,
}

/// Copy a file from `src` to `dst`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCopyStep {
    pub src: String,
    pub dst: String,
    /// Overwrite the destination file if it already exists. Default: false.
    #[serde(default)]
    pub overwrite: bool,
}

/// Move (rename) a file from `src` to `dst`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMoveStep {
    pub src: String,
    pub dst: String,
}

/// Delete a file. Set `ignore_missing: true` to skip without error if absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeleteStep {
    pub path: String,
    #[serde(default = "default_true")]
    pub ignore_missing: bool,
}

/// Rename a file within the same directory (new filename only, no directory part).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRenameStep {
    pub path: String,
    pub name: String,
}

// ── log write step type ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    #[default]
    Info,
    Warn,
    Error,
    Debug,
}

/// Append a timestamped line to a log file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogWriteStep {
    pub file: String,
    pub message: String,
    #[serde(default)]
    pub level: LogLevel,
}

// ── date operation step types ─────────────────────────────────────────────

/// Reformat a date string from one format to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFormatStep {
    pub value: String,
    #[serde(default = "default_date_format")]
    pub from_format: String,
    pub to_format: String,
    pub save_as: String,
}

/// Add (or subtract) days/months/years to a date string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateAddStep {
    pub value: String,
    #[serde(default = "default_date_format")]
    pub format: String,
    #[serde(default)]
    pub days: i64,
    #[serde(default)]
    pub months: i64,
    #[serde(default)]
    pub years: i64,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiffUnit {
    #[default]
    Days,
    Months,
}

/// Compute the difference between two date strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateDiffStep {
    pub from: String,
    pub to: String,
    #[serde(default = "default_date_format")]
    pub format: String,
    #[serde(default)]
    pub unit: DiffUnit,
    pub save_as: String,
}

// ── string operation step types ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringReplaceStep {
    pub value: String,
    pub from: String,
    pub to: String,
    /// Replace all occurrences when true (default), only the first when false.
    #[serde(default = "default_true")]
    pub all: bool,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrimSide {
    #[default]
    Both,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringTrimStep {
    pub value: String,
    #[serde(default)]
    pub side: TrimSide,
    pub save_as: String,
}

/// Shared struct for `string_upper` and `string_lower`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringCaseStep {
    pub value: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringSubstringStep {
    pub value: String,
    pub start: usize,
    /// Number of characters to extract. If absent, returns from `start` to end.
    #[serde(default)]
    pub length: Option<usize>,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLengthStep {
    pub value: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringSplitStep {
    pub value: String,
    pub delimiter: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringJoinStep {
    /// Name of the array variable to join.
    pub value: String,
    #[serde(default)]
    pub separator: String,
    pub save_as: String,
}

/// Regex match; stores `{found: bool, full: str, groups: [str, ...]}` in `save_as`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringRegexStep {
    pub value: String,
    pub pattern: String,
    pub save_as: String,
}

// ── json helper step types ────────────────────────────────────────────────

/// Parse a JSON string into a structured variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonParseStep {
    pub value: String,
    pub save_as: String,
}

/// Serialize a variable to a JSON string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStringifyStep {
    pub value: String,
    pub save_as: String,
}

// ── path helper step types ────────────────────────────────────────────────

/// Join path segments into a single path string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathJoinStep {
    pub parts: Vec<String>,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathBasenameStep {
    pub path: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathDirnameStep {
    pub path: String,
    pub save_as: String,
}

// ── env / file-list step types ────────────────────────────────────────────

/// Read an environment variable; optionally fall back to `default`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvGetStep {
    pub name: String,
    #[serde(default)]
    pub default: Option<String>,
    pub save_as: String,
}

/// List files matching a glob pattern. Each entry: `{name, path, is_dir, size_bytes}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListStep {
    pub dir: String,
    #[serde(default = "default_glob_pattern")]
    pub pattern: String,
    pub save_as: String,
}

// ── mouse coordinate step types ───────────────────────────────────────────

/// Move the mouse cursor to absolute screen coordinates.
/// `x` and `y` accept literal integers or `{{ var }}` template expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveStep {
    pub x: String,
    pub y: String,
}

/// Click at absolute screen coordinates.
/// `x` and `y` accept literal integers or `{{ var }}` template expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseClickXyStep {
    pub x: String,
    pub y: String,
    #[serde(default)]
    pub action: ClickAction,
}

/// Click and drag from one point to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseDragStep {
    pub from_x: String,
    pub from_y: String,
    pub to_x: String,
    pub to_y: String,
    #[serde(default = "default_hold_ms")]
    pub hold_ms: u64,
}

/// Scroll the mouse wheel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseScrollStep {
    /// `"up"` | `"down"` | `"left"` | `"right"`
    #[serde(default = "default_scroll_direction")]
    pub direction: String,
    #[serde(default = "default_scroll_amount")]
    pub amount: i32,
}

// ── HTTP client step types ─────────────────────────────────────────────────

/// Content-type for HTTP request bodies.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    #[default]
    Json,
    Form,
    Text,
}

/// HTTP authentication (Basic or Bearer).
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HttpAuth {
    Basic { user: String, password: String },
    Bearer { token: String },
}

/// HTTP GET request.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpGetStep {
    pub url: String,
    /// Variable name to store `{status, body, body_json}`.
    pub save_as: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_http_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub auth: Option<HttpAuth>,
}

/// HTTP POST request.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPostStep {
    pub url: String,
    pub save_as: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_http_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub content_type: ContentType,
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub auth: Option<HttpAuth>,
}

/// HTTP PUT request.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPutStep {
    pub url: String,
    pub save_as: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_http_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub content_type: ContentType,
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub auth: Option<HttpAuth>,
}

/// HTTP DELETE request.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDeleteStep {
    pub url: String,
    pub save_as: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_http_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub auth: Option<HttpAuth>,
}

/// HTTP PATCH request.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPatchStep {
    pub url: String,
    pub save_as: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_http_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub content_type: ContentType,
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub auth: Option<HttpAuth>,
}

// ── Excel cell step types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelReadCellStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    pub cell: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelReadRangeStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    pub range: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelWriteCellStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    pub cell: String,
    pub value: String,
}

// ── Mail receive step type ────────────────────────────────────────────────────

/// Receive emails via IMAP.  Saves a list of `{subject, from, date, body, seen}` maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailReceiveStep {
    pub host: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_imap_port")]
    pub port: u16,
    #[serde(default = "default_imap_folder")]
    pub folder: String,
    #[serde(default = "default_mail_count")]
    pub count: u32,
    #[serde(default)]
    pub only_unseen: bool,
    pub save_as: String,
}

fn default_imap_port() -> u16 {
    993
}
fn default_imap_folder() -> String {
    "INBOX".to_owned()
}
fn default_mail_count() -> u32 {
    10
}

// ── Excel sheet management step types ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelAddSheetStep {
    pub file: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelDeleteSheetStep {
    pub file: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelRenameSheetStep {
    pub file: String,
    pub from_name: String,
    pub to_name: String,
}

// ── text file read/write step types ──────────────────────────────────────────

/// Read the entire contents of a text file into a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadStep {
    pub path: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileWriteMode {
    #[default]
    Overwrite,
    Append,
}

/// Write text to a file (overwrite or append).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteStep {
    pub path: String,
    pub content: String,
    #[serde(default)]
    pub mode: FileWriteMode,
}

/// Append text to a file (shorthand for `file_write` with `mode: append`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAppendStep {
    pub path: String,
    pub content: String,
}

// ── process operation step types ─────────────────────────────────────────────

/// Start a process and optionally save its PID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStartStep {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Milliseconds to wait after spawning (for the app to initialize). Default: 0.
    #[serde(default)]
    pub wait_ms: u64,
    /// Variable name to store the spawned process PID. Optional.
    #[serde(default)]
    pub save_pid_as: Option<String>,
}

/// Terminate a process by PID or name. `pid` takes priority over `name`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessKillStep {
    #[serde(default)]
    pub pid: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Check whether a process with the given name is currently running.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessExistsStep {
    pub name: String,
    pub save_as: String,
}

// ── key combination step type ─────────────────────────────────────────────────

/// Press a key combination: all but the last key are held as modifiers.
/// Key names are case-insensitive. Examples: `["ctrl", "c"]`, `["alt", "F4"]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyComboStep {
    pub keys: Vec<String>,
}

// ── CSV read/write step types ─────────────────────────────────────────────────

/// Read a CSV file into a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvReadStep {
    pub path: String,
    /// When true (default), the first row is the header; each row becomes a map.
    /// When false, each row becomes a list of strings.
    #[serde(default = "default_true")]
    pub has_header: bool,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvWriteMode {
    #[default]
    Overwrite,
    Append,
}

/// Write rows to a CSV file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvWriteStep {
    pub path: String,
    /// Variable name containing `list<map>` or `list<list>` to write.
    pub rows: String,
    /// Column headers for `list<list>` rows. If rows is `list<map>`, headers
    /// are inferred from the first row's keys unless overridden here.
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub mode: CsvWriteMode,
}

// ── Sprint 8 step types ────────────────────────────────────────────────────

/// Save a screenshot (full screen or a specific window) to a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotSaveStep {
    /// Destination file path. Supports `{{ var }}` expansion. Parent dirs are created.
    pub path: String,
    /// If set, capture the window whose title contains this string; otherwise full screen.
    #[serde(default)]
    pub window: Option<String>,
}

/// Wait until a template image is no longer visible on screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitNoImageStep {
    pub template: String,
    /// How long to wait in total before failing (ms). Default: 30000.
    #[serde(default = "default_wait_no_image_timeout_ms")]
    pub timeout_ms: u64,
    /// Polling interval (ms). Default: 500.
    #[serde(default = "default_wait_no_image_interval_ms")]
    pub interval_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

/// Open a URL in the system default browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlOpenStep {
    pub url: String,
}

/// Display a desktop notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyStep {
    pub title: String,
    pub message: String,
}

// ── UI Automation step types ──────────────────────────────────────────────

/// How to locate a UIA element.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiaBy {
    /// Match by the element's Name property (accessibility label).
    Name(String),
    /// Match by the element's AutomationId property.
    Id(String),
    /// Match by the element's ClassName.
    Class(String),
}

/// Get a UI Automation element property.
///
/// ```yaml
/// - uia_get:
///     by: { name: "ユーザー名" }
///     property: value   # name | value | class | rect
///     save_as: result
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaGetStep {
    pub by: UiaBy,
    #[serde(default = "default_uia_property")]
    pub property: String,
    pub save_as: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Set a UIA element value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaSetStep {
    pub by: UiaBy,
    pub value: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Invoke (click) a UIA element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaClickStep {
    pub by: UiaBy,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Find a UIA element and store its bounding rect.
/// Output: { x, y, width, height, name }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaFindStep {
    pub by: UiaBy,
    pub save_as: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_uia_property() -> String {
    "value".to_owned()
}

// ── Web automation step types ─────────────────────────────────────────────

/// Open a URL in a WebDriver-controlled browser.
///
/// ```yaml
/// - web_open:
///     url: "https://example.com"
///     driver: "http://localhost:4444"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebOpenStep {
    pub url: String,
    /// WebDriver endpoint URL (ChromeDriver, GeckoDriver, etc.).
    #[serde(default = "default_webdriver_url")]
    pub driver: String,
}

/// Click an element by CSS selector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebClickStep {
    pub selector: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Type text into an element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebTypeStep {
    pub selector: String,
    pub text: String,
    /// Clear field before typing (default true).
    #[serde(default = "default_true")]
    pub clear: bool,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Get text or attribute from an element.
///
/// ```yaml
/// - web_get:
///     selector: ".result"
///     attr: "href"   # omit for innerText
///     save_as: link
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGetStep {
    pub selector: String,
    /// Element attribute name. If absent, returns the element's visible text.
    #[serde(default)]
    pub attr: Option<String>,
    pub save_as: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Wait for an element to be present.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebWaitStep {
    pub selector: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Save the browser viewport as a PNG screenshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScreenshotStep {
    pub path: String,
}

fn default_webdriver_url() -> String {
    "http://localhost:4444".to_owned()
}

// ── default value helpers ──────────────────────────────────────────────────

fn default_true() -> bool {
    true
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_owned()
}

fn default_glob_pattern() -> String {
    "*".to_owned()
}

fn default_hold_ms() -> u64 {
    50
}

fn default_scroll_direction() -> String {
    "down".to_owned()
}

fn default_scroll_amount() -> i32 {
    3
}

fn default_ml_threshold() -> f32 {
    0.7
}

fn default_ocr_lang() -> String {
    "jpn+eng".to_owned()
}

fn default_timeout_ms() -> u64 {
    5000
}
fn default_retry_interval_ms() -> u64 {
    200
}
fn default_wait_no_image_timeout_ms() -> u64 {
    30_000
}
fn default_wait_no_image_interval_ms() -> u64 {
    500
}
fn default_shell_timeout_ms() -> u64 {
    30_000
}
fn default_datetime_format() -> String {
    "%Y-%m-%d %H:%M:%S".to_owned()
}
fn default_increment_by() -> i64 {
    1
}
#[cfg(feature = "http")]
fn default_http_timeout_ms() -> u64 {
    30_000
}

// ── Custom Deserialize for ScenarioStep ───────────────────────────────────
//
// serde_yml uses YAML tags (!variant) for enum serialization.
// Users write {variant: value} maps — this custom impl handles that format.
// It also accepts bare strings for unit variants ("break", "continue", "exit").

const KNOWN_VARIANTS: &[&str] = &[
    "wait_image",
    "click_image",
    "type",
    "press",
    "library",
    "script",
    "foreach",
    "sub_scenario",
    "set",
    "wait_ms",
    "group",
    "if",
    "switch",
    "repeat",
    "while",
    "do_while",
    "try_catch",
    "break",
    "continue",
    "call_scenario",
    "exit",
    "find_image",
    "shell",
    "clipboard_set",
    "clipboard_get",
    "copy_var",
    "get_datetime",
    "get_username",
    "calc",
    "increment",
    "to_fullwidth",
    "to_halfwidth",
    "dialog_wait",
    "dialog_input",
    "dialog_select",
    "wait_window",
    "match_rect",
    "window_control",
    "ocr_match",
    "ml_detect",
    "import_vars",
    "save_vars",
    "load_vars",
    "file_exists",
    "file_copy",
    "file_move",
    "file_delete",
    "file_rename",
    "log_write",
    "date_format",
    "date_add",
    "date_diff",
    "string_replace",
    "string_trim",
    "string_upper",
    "string_lower",
    "string_substring",
    "string_length",
    "string_split",
    "string_join",
    "string_regex",
    "json_parse",
    "json_stringify",
    "path_join",
    "path_basename",
    "path_dirname",
    "env_get",
    "file_list",
    "mouse_move",
    "mouse_click_xy",
    "mouse_drag",
    "mouse_scroll",
    "http_get",
    "http_post",
    "http_put",
    "http_delete",
    "http_patch",
    "mail_receive",
    "excel_read_cell",
    "excel_read_range",
    "excel_write_cell",
    "excel_add_sheet",
    "excel_delete_sheet",
    "excel_rename_sheet",
    "file_read",
    "file_write",
    "file_append",
    "process_start",
    "process_kill",
    "process_exists",
    "key_combo",
    "csv_read",
    "csv_write",
    "screenshot_save",
    "wait_no_image",
    "url_open",
    "notify",
    "uia_get",
    "uia_set",
    "uia_click",
    "uia_find",
    "web_open",
    "web_click",
    "web_type",
    "web_get",
    "web_wait",
    "web_screenshot",
    "web_close",
];

struct ScenarioStepVisitor;

impl<'de> serde::de::Visitor<'de> for ScenarioStepVisitor {
    type Value = ScenarioStep;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a scenario step (map or unit-variant string)")
    }

    /// Bare string → unit variant.
    fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<ScenarioStep, E> {
        match s {
            "break" => Ok(ScenarioStep::Break),
            "continue" => Ok(ScenarioStep::Continue),
            "exit" => Ok(ScenarioStep::Exit),
            other => Err(E::unknown_variant(other, KNOWN_VARIANTS)),
        }
    }

    /// {variant: value} map → enum variant.
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<ScenarioStep, A::Error> {
        use serde::de::Error;

        let key: String = map
            .next_key()?
            .ok_or_else(|| Error::custom("empty step map"))?;

        let step = match key.as_str() {
            // --- basic actions ---
            "wait_image" => ScenarioStep::WaitImage(map.next_value()?),
            "click_image" => ScenarioStep::ClickImage(map.next_value()?),
            "type" => ScenarioStep::Type(map.next_value()?),
            "press" => ScenarioStep::Press(map.next_value()?),
            "library" => ScenarioStep::Library(map.next_value()?),
            "script" => ScenarioStep::Script(map.next_value()?),
            "foreach" => ScenarioStep::Foreach(map.next_value()?),
            "sub_scenario" => ScenarioStep::SubScenario(map.next_value()?),
            "set" => ScenarioStep::Set(map.next_value()?),
            "wait_ms" => ScenarioStep::WaitMs(map.next_value()?),
            // --- flow control ---
            "group" => ScenarioStep::Group(map.next_value()?),
            "if" => ScenarioStep::If(map.next_value()?),
            "switch" => ScenarioStep::Switch(map.next_value()?),
            "repeat" => ScenarioStep::Repeat(map.next_value()?),
            "while" => ScenarioStep::While(map.next_value()?),
            "do_while" => ScenarioStep::DoWhile(map.next_value()?),
            "try_catch" => ScenarioStep::TryCatch(map.next_value()?),
            "call_scenario" => ScenarioStep::CallScenario(map.next_value()?),
            // unit variants written as {key: null}
            "break" => {
                let _: serde::de::IgnoredAny = map.next_value()?;
                ScenarioStep::Break
            }
            "continue" => {
                let _: serde::de::IgnoredAny = map.next_value()?;
                ScenarioStep::Continue
            }
            "exit" => {
                let _: serde::de::IgnoredAny = map.next_value()?;
                ScenarioStep::Exit
            }
            // --- action nodes ---
            "find_image" => ScenarioStep::FindImage(map.next_value()?),
            "shell" => ScenarioStep::Shell(map.next_value()?),
            "clipboard_set" => ScenarioStep::ClipboardSet(map.next_value()?),
            "clipboard_get" => ScenarioStep::ClipboardGet(map.next_value()?),
            // --- variable nodes ---
            "copy_var" => ScenarioStep::CopyVar(map.next_value()?),
            "get_datetime" => ScenarioStep::GetDatetime(map.next_value()?),
            "get_username" => ScenarioStep::GetUsername(map.next_value()?),
            "calc" => ScenarioStep::Calc(map.next_value()?),
            "increment" => ScenarioStep::Increment(map.next_value()?),
            "to_fullwidth" => ScenarioStep::ToFullwidth(map.next_value()?),
            "to_halfwidth" => ScenarioStep::ToHalfwidth(map.next_value()?),
            // --- user interaction ---
            "dialog_wait" => ScenarioStep::DialogWait(map.next_value()?),
            "dialog_input" => ScenarioStep::DialogInput(map.next_value()?),
            "dialog_select" => ScenarioStep::DialogSelect(map.next_value()?),
            // --- window / region nodes ---
            "wait_window" => ScenarioStep::WaitWindow(map.next_value()?),
            "match_rect" => ScenarioStep::MatchRect(map.next_value()?),
            "window_control" => ScenarioStep::WindowControl(map.next_value()?),
            "ocr_match" => ScenarioStep::OcrMatch(map.next_value()?),
            // --- ML detection ---
            "ml_detect" => ScenarioStep::MlDetect(map.next_value()?),
            // --- variable persistence ---
            "import_vars" => ScenarioStep::ImportVars(map.next_value()?),
            "save_vars" => ScenarioStep::SaveVars(map.next_value()?),
            "load_vars" => ScenarioStep::LoadVars(map.next_value()?),
            // --- file operations ---
            "file_exists" => ScenarioStep::FileExists(map.next_value()?),
            "file_copy" => ScenarioStep::FileCopy(map.next_value()?),
            "file_move" => ScenarioStep::FileMove(map.next_value()?),
            "file_delete" => ScenarioStep::FileDelete(map.next_value()?),
            "file_rename" => ScenarioStep::FileRename(map.next_value()?),
            // --- logging ---
            "log_write" => ScenarioStep::LogWrite(map.next_value()?),
            // --- date operations ---
            "date_format" => ScenarioStep::DateFormat(map.next_value()?),
            "date_add" => ScenarioStep::DateAdd(map.next_value()?),
            "date_diff" => ScenarioStep::DateDiff(map.next_value()?),
            // --- string operations ---
            "string_replace" => ScenarioStep::StringReplace(map.next_value()?),
            "string_trim" => ScenarioStep::StringTrim(map.next_value()?),
            "string_upper" => ScenarioStep::StringUpper(map.next_value()?),
            "string_lower" => ScenarioStep::StringLower(map.next_value()?),
            "string_substring" => ScenarioStep::StringSubstring(map.next_value()?),
            "string_length" => ScenarioStep::StringLength(map.next_value()?),
            "string_split" => ScenarioStep::StringSplit(map.next_value()?),
            "string_join" => ScenarioStep::StringJoin(map.next_value()?),
            "string_regex" => ScenarioStep::StringRegex(map.next_value()?),
            // --- json helpers ---
            "json_parse" => ScenarioStep::JsonParse(map.next_value()?),
            "json_stringify" => ScenarioStep::JsonStringify(map.next_value()?),
            // --- path helpers ---
            "path_join" => ScenarioStep::PathJoin(map.next_value()?),
            "path_basename" => ScenarioStep::PathBasename(map.next_value()?),
            "path_dirname" => ScenarioStep::PathDirname(map.next_value()?),
            // --- env / misc ---
            "env_get" => ScenarioStep::EnvGet(map.next_value()?),
            "file_list" => ScenarioStep::FileList(map.next_value()?),
            // --- mouse coordinate nodes ---
            "mouse_move" => ScenarioStep::MouseMove(map.next_value()?),
            "mouse_click_xy" => ScenarioStep::MouseClickXy(map.next_value()?),
            "mouse_drag" => ScenarioStep::MouseDrag(map.next_value()?),
            "mouse_scroll" => ScenarioStep::MouseScroll(map.next_value()?),
            // --- HTTP client nodes ---
            #[cfg(feature = "http")]
            "http_get" => ScenarioStep::HttpGet(map.next_value()?),
            #[cfg(feature = "http")]
            "http_post" => ScenarioStep::HttpPost(map.next_value()?),
            #[cfg(feature = "http")]
            "http_put" => ScenarioStep::HttpPut(map.next_value()?),
            #[cfg(feature = "http")]
            "http_delete" => ScenarioStep::HttpDelete(map.next_value()?),
            #[cfg(feature = "http")]
            "http_patch" => ScenarioStep::HttpPatch(map.next_value()?),
            // --- mail ---
            "mail_receive" => ScenarioStep::MailReceive(map.next_value()?),
            // --- Excel cell nodes ---
            "excel_read_cell" => ScenarioStep::ExcelReadCell(map.next_value()?),
            "excel_read_range" => ScenarioStep::ExcelReadRange(map.next_value()?),
            "excel_write_cell" => ScenarioStep::ExcelWriteCell(map.next_value()?),
            // --- Excel sheet management nodes ---
            "excel_add_sheet" => ScenarioStep::ExcelAddSheet(map.next_value()?),
            "excel_delete_sheet" => ScenarioStep::ExcelDeleteSheet(map.next_value()?),
            "excel_rename_sheet" => ScenarioStep::ExcelRenameSheet(map.next_value()?),
            // --- text file read/write ---
            "file_read" => ScenarioStep::FileRead(map.next_value()?),
            "file_write" => ScenarioStep::FileWrite(map.next_value()?),
            "file_append" => ScenarioStep::FileAppend(map.next_value()?),
            // --- process operations ---
            "process_start" => ScenarioStep::ProcessStart(map.next_value()?),
            "process_kill" => ScenarioStep::ProcessKill(map.next_value()?),
            "process_exists" => ScenarioStep::ProcessExists(map.next_value()?),
            // --- key combination ---
            "key_combo" => ScenarioStep::KeyCombo(map.next_value()?),
            // --- CSV read/write ---
            "csv_read" => ScenarioStep::CsvRead(map.next_value()?),
            "csv_write" => ScenarioStep::CsvWrite(map.next_value()?),
            // --- screenshot / observation ---
            "screenshot_save" => ScenarioStep::ScreenshotSave(map.next_value()?),
            "wait_no_image" => ScenarioStep::WaitNoImage(map.next_value()?),
            // --- system integration ---
            "url_open" => ScenarioStep::UrlOpen(map.next_value()?),
            "notify" => ScenarioStep::Notify(map.next_value()?),
            // --- UI Automation ---
            "uia_get" => ScenarioStep::UiaGet(map.next_value()?),
            "uia_set" => ScenarioStep::UiaSet(map.next_value()?),
            "uia_click" => ScenarioStep::UiaClick(map.next_value()?),
            "uia_find" => ScenarioStep::UiaFind(map.next_value()?),
            // --- Web automation ---
            "web_open" => ScenarioStep::WebOpen(map.next_value()?),
            "web_click" => ScenarioStep::WebClick(map.next_value()?),
            "web_type" => ScenarioStep::WebType(map.next_value()?),
            "web_get" => ScenarioStep::WebGet(map.next_value()?),
            "web_wait" => ScenarioStep::WebWait(map.next_value()?),
            "web_screenshot" => ScenarioStep::WebScreenshot(map.next_value()?),
            "web_close" => {
                let _: serde::de::IgnoredAny = map.next_value()?;
                ScenarioStep::WebClose
            }
            other => return Err(A::Error::unknown_variant(other, KNOWN_VARIANTS)),
        };
        Ok(step)
    }
}

impl<'de> Deserialize<'de> for ScenarioStep {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ScenarioStepVisitor)
    }
}

// ── Scenario constructors ──────────────────────────────────────────────────

impl Scenario {
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yml::Error> {
        serde_yml::from_str(yaml)
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_yaml(&content)?)
    }
}

impl ScenarioStep {
    /// Short name for debug display.
    pub fn name(&self) -> &'static str {
        match self {
            Self::WaitImage(_) => "wait_image",
            Self::ClickImage(_) => "click_image",
            Self::Type(_) => "type",
            Self::Press(_) => "press",
            Self::Library(_) => "library",
            Self::Script(_) => "script",
            Self::Foreach(_) => "foreach",
            Self::SubScenario(_) => "sub_scenario",
            Self::Set(_) => "set",
            Self::WaitMs(_) => "wait_ms",
            Self::Group(_) => "group",
            Self::If(_) => "if",
            Self::Switch(_) => "switch",
            Self::Repeat(_) => "repeat",
            Self::While(_) => "while",
            Self::DoWhile(_) => "do_while",
            Self::TryCatch(_) => "try_catch",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::CallScenario(_) => "call_scenario",
            Self::Exit => "exit",
            Self::FindImage(_) => "find_image",
            Self::Shell(_) => "shell",
            Self::ClipboardSet(_) => "clipboard_set",
            Self::ClipboardGet(_) => "clipboard_get",
            Self::CopyVar(_) => "copy_var",
            Self::GetDatetime(_) => "get_datetime",
            Self::GetUsername(_) => "get_username",
            Self::Calc(_) => "calc",
            Self::Increment(_) => "increment",
            Self::ToFullwidth(_) => "to_fullwidth",
            Self::ToHalfwidth(_) => "to_halfwidth",
            Self::DialogWait(_) => "dialog_wait",
            Self::DialogInput(_) => "dialog_input",
            Self::DialogSelect(_) => "dialog_select",
            Self::WaitWindow(_) => "wait_window",
            Self::MatchRect(_) => "match_rect",
            Self::WindowControl(_) => "window_control",
            Self::OcrMatch(_) => "ocr_match",
            Self::MlDetect(_) => "ml_detect",
            Self::ImportVars(_) => "import_vars",
            Self::SaveVars(_) => "save_vars",
            Self::LoadVars(_) => "load_vars",
            Self::FileExists(_) => "file_exists",
            Self::FileCopy(_) => "file_copy",
            Self::FileMove(_) => "file_move",
            Self::FileDelete(_) => "file_delete",
            Self::FileRename(_) => "file_rename",
            Self::LogWrite(_) => "log_write",
            Self::DateFormat(_) => "date_format",
            Self::DateAdd(_) => "date_add",
            Self::DateDiff(_) => "date_diff",
            Self::StringReplace(_) => "string_replace",
            Self::StringTrim(_) => "string_trim",
            Self::StringUpper(_) => "string_upper",
            Self::StringLower(_) => "string_lower",
            Self::StringSubstring(_) => "string_substring",
            Self::StringLength(_) => "string_length",
            Self::StringSplit(_) => "string_split",
            Self::StringJoin(_) => "string_join",
            Self::StringRegex(_) => "string_regex",
            Self::JsonParse(_) => "json_parse",
            Self::JsonStringify(_) => "json_stringify",
            Self::PathJoin(_) => "path_join",
            Self::PathBasename(_) => "path_basename",
            Self::PathDirname(_) => "path_dirname",
            Self::EnvGet(_) => "env_get",
            Self::FileList(_) => "file_list",
            Self::MouseMove(_) => "mouse_move",
            Self::MouseClickXy(_) => "mouse_click_xy",
            Self::MouseDrag(_) => "mouse_drag",
            Self::MouseScroll(_) => "mouse_scroll",
            #[cfg(feature = "http")]
            Self::HttpGet(_) => "http_get",
            #[cfg(feature = "http")]
            Self::HttpPost(_) => "http_post",
            #[cfg(feature = "http")]
            Self::HttpPut(_) => "http_put",
            #[cfg(feature = "http")]
            Self::HttpDelete(_) => "http_delete",
            #[cfg(feature = "http")]
            Self::HttpPatch(_) => "http_patch",
            Self::MailReceive(_) => "mail_receive",
            Self::ExcelReadCell(_) => "excel_read_cell",
            Self::ExcelReadRange(_) => "excel_read_range",
            Self::ExcelWriteCell(_) => "excel_write_cell",
            Self::ExcelAddSheet(_) => "excel_add_sheet",
            Self::ExcelDeleteSheet(_) => "excel_delete_sheet",
            Self::ExcelRenameSheet(_) => "excel_rename_sheet",
            Self::FileRead(_) => "file_read",
            Self::FileWrite(_) => "file_write",
            Self::FileAppend(_) => "file_append",
            Self::ProcessStart(_) => "process_start",
            Self::ProcessKill(_) => "process_kill",
            Self::ProcessExists(_) => "process_exists",
            Self::KeyCombo(_) => "key_combo",
            Self::CsvRead(_) => "csv_read",
            Self::CsvWrite(_) => "csv_write",
            Self::ScreenshotSave(_) => "screenshot_save",
            Self::WaitNoImage(_) => "wait_no_image",
            Self::UrlOpen(_) => "url_open",
            Self::Notify(_) => "notify",
            Self::UiaGet(_) => "uia_get",
            Self::UiaSet(_) => "uia_set",
            Self::UiaClick(_) => "uia_click",
            Self::UiaFind(_) => "uia_find",
            Self::WebOpen(_) => "web_open",
            Self::WebClick(_) => "web_click",
            Self::WebType(_) => "web_type",
            Self::WebGet(_) => "web_get",
            Self::WebWait(_) => "web_wait",
            Self::WebScreenshot(_) => "web_screenshot",
            Self::WebClose => "web_close",
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_vars_step_parses() {
        let yaml = r#"
name: test
steps:
  - import_vars:
      file: "data.xlsx"
      sheet: "Sheet1"
      row: 1
      prefix: "inp_"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::ImportVars(step) = &s.steps[0] else {
            panic!("wrong variant");
        };
        assert_eq!(step.file, "data.xlsx");
        assert_eq!(step.sheet.as_deref(), Some("Sheet1"));
        assert_eq!(step.row, 1);
        assert_eq!(step.prefix, "inp_");
    }

    #[test]
    fn import_vars_defaults() {
        let yaml = r#"
name: test
steps:
  - import_vars:
      file: "params.csv"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::ImportVars(step) = &s.steps[0] else {
            panic!("wrong variant");
        };
        assert!(step.sheet.is_none());
        assert_eq!(step.row, 0);
        assert!(step.prefix.is_empty());
    }

    #[test]
    fn save_and_load_vars_step_parses() {
        let yaml = r#"
name: test
steps:
  - save_vars:
      file: "state.json"
      vars: ["counter", "status"]
  - load_vars:
      file: "state.json"
      prefix: "loaded_"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::SaveVars(sv) = &s.steps[0] else {
            panic!("wrong variant");
        };
        assert_eq!(sv.file, "state.json");
        assert_eq!(sv.vars, vec!["counter", "status"]);

        let ScenarioStep::LoadVars(lv) = &s.steps[1] else {
            panic!("wrong variant");
        };
        assert_eq!(lv.file, "state.json");
        assert_eq!(lv.prefix, "loaded_");
    }

    #[test]
    fn save_vars_no_filter_default() {
        let yaml = r#"
name: test
steps:
  - save_vars:
      file: "out.json"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::SaveVars(sv) = &s.steps[0] else {
            panic!("wrong variant");
        };
        assert!(sv.vars.is_empty());
    }

    #[test]
    fn file_ops_parse() {
        let yaml = r#"
name: test
steps:
  - file_exists:
      path: "out/report.xlsx"
      save_as: found
  - file_copy:
      src: "template.xlsx"
      dst: "out/result.xlsx"
      overwrite: true
  - file_move:
      src: "inbox/data.csv"
      dst: "done/data.csv"
  - file_delete:
      path: "tmp/work.tmp"
      ignore_missing: false
  - file_rename:
      path: "out/old.csv"
      name: "new.csv"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 5);
        let ScenarioStep::FileExists(fe) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(fe.path, "out/report.xlsx");
        assert_eq!(fe.save_as, "found");
        let ScenarioStep::FileCopy(fc) = &s.steps[1] else {
            panic!()
        };
        assert!(fc.overwrite);
        let ScenarioStep::FileDelete(fd) = &s.steps[3] else {
            panic!()
        };
        assert!(!fd.ignore_missing);
    }

    #[test]
    fn file_delete_defaults_ignore_missing() {
        let yaml = r#"
name: test
steps:
  - file_delete:
      path: "x.tmp"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::FileDelete(fd) = &s.steps[0] else {
            panic!()
        };
        assert!(fd.ignore_missing);
    }

    #[test]
    fn log_write_parses() {
        let yaml = r#"
name: test
steps:
  - log_write:
      file: "logs/run.log"
      message: "done"
      level: warn
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::LogWrite(lw) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(lw.file, "logs/run.log");
        assert_eq!(lw.level, LogLevel::Warn);
    }

    #[test]
    fn date_steps_parse() {
        let yaml = r#"
name: test
steps:
  - date_format:
      value: "2024-01-15"
      from_format: "%Y-%m-%d"
      to_format: "%Y/%m/%d"
      save_as: formatted
  - date_add:
      value: "{{ today }}"
      format: "%Y-%m-%d"
      days: 30
      months: -1
      save_as: due
  - date_diff:
      from: "2024-01-01"
      to: "2024-03-31"
      format: "%Y-%m-%d"
      unit: days
      save_as: elapsed
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 3);
        let ScenarioStep::DateFormat(df) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(df.to_format, "%Y/%m/%d");
        assert_eq!(df.save_as, "formatted");
        let ScenarioStep::DateAdd(da) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(da.days, 30);
        assert_eq!(da.months, -1);
        let ScenarioStep::DateDiff(dd) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(dd.unit, DiffUnit::Days);
        assert_eq!(dd.save_as, "elapsed");
    }

    #[test]
    fn string_steps_parse() {
        let yaml = r#"
name: test
steps:
  - string_replace:
      value: "{{ text }}"
      from: "　"
      to: " "
      all: true
      save_as: normalized
  - string_trim:
      value: "  hello  "
      side: left
      save_as: trimmed
  - string_upper:
      value: "hello"
      save_as: up
  - string_lower:
      value: "HELLO"
      save_as: down
  - string_substring:
      value: "abcdef"
      start: 2
      length: 3
      save_as: sub
  - string_length:
      value: "hello"
      save_as: len
  - string_split:
      value: "a,b,c"
      delimiter: ","
      save_as: parts
  - string_join:
      value: parts
      separator: "-"
      save_as: joined
  - string_regex:
      value: "2024-01-15"
      pattern: '(\d{4})-(\d{2})-(\d{2})'
      save_as: match
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 9);
        let ScenarioStep::StringReplace(sr) = &s.steps[0] else {
            panic!()
        };
        assert!(sr.all);
        assert_eq!(sr.save_as, "normalized");
        let ScenarioStep::StringTrim(st) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(st.side, TrimSide::Left);
        let ScenarioStep::StringSubstring(ss) = &s.steps[4] else {
            panic!()
        };
        assert_eq!(ss.start, 2);
        assert_eq!(ss.length, Some(3));
        let ScenarioStep::StringRegex(rx) = &s.steps[8] else {
            panic!()
        };
        assert_eq!(rx.save_as, "match");
    }

    #[test]
    fn string_replace_defaults_all_true() {
        let yaml = r#"
name: test
steps:
  - string_replace:
      value: "aaa"
      from: "a"
      to: "b"
      save_as: out
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::StringReplace(sr) = &s.steps[0] else {
            panic!()
        };
        assert!(sr.all);
    }

    #[test]
    fn json_and_path_steps_parse() {
        let yaml = r#"
name: test
steps:
  - json_parse:
      value: '{"key": 1}'
      save_as: obj
  - json_stringify:
      value: obj
      save_as: text
  - path_join:
      parts: ["reports", "{{ date }}", "out.csv"]
      save_as: full_path
  - path_basename:
      path: "/tmp/data/file.csv"
      save_as: name
  - path_dirname:
      path: "/tmp/data/file.csv"
      save_as: dir
  - env_get:
      name: HOME
      default: "/home/user"
      save_as: home
  - file_list:
      dir: "incoming/"
      pattern: "*.xlsx"
      save_as: files
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 7);
        let ScenarioStep::PathJoin(pj) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(pj.parts.len(), 3);
        let ScenarioStep::EnvGet(eg) = &s.steps[5] else {
            panic!()
        };
        assert_eq!(eg.default.as_deref(), Some("/home/user"));
        let ScenarioStep::FileList(fl) = &s.steps[6] else {
            panic!()
        };
        assert_eq!(fl.pattern, "*.xlsx");
    }

    #[test]
    fn mouse_steps_parse() {
        let yaml = r#"
name: test
steps:
  - mouse_move:
      x: "640"
      y: "480"
  - mouse_click_xy:
      x: "{{ btn_x }}"
      y: "{{ btn_y }}"
      action: right
  - mouse_drag:
      from_x: "100"
      from_y: "200"
      to_x: "500"
      to_y: "200"
      hold_ms: 100
  - mouse_scroll:
      direction: up
      amount: 5
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 4);
        let ScenarioStep::MouseMove(mm) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(mm.x, "640");
        let ScenarioStep::MouseClickXy(mc) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(mc.x, "{{ btn_x }}");
        assert_eq!(mc.action, ClickAction::Right);
        let ScenarioStep::MouseDrag(md) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(md.hold_ms, 100);
        let ScenarioStep::MouseScroll(ms) = &s.steps[3] else {
            panic!()
        };
        assert_eq!(ms.direction, "up");
        assert_eq!(ms.amount, 5);
    }

    #[test]
    fn mouse_scroll_defaults() {
        let yaml = r#"
name: test
steps:
  - mouse_scroll: {}
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::MouseScroll(ms) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(ms.direction, "down");
        assert_eq!(ms.amount, 3);
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_steps_parse() {
        let yaml = r#"
name: test
steps:
  - http_get:
      url: "https://example.com/api"
      save_as: resp
      headers:
        Authorization: "Bearer token123"
      timeout_ms: 10000
  - http_post:
      url: "https://example.com/api/data"
      save_as: post_resp
      content_type: json
      body:
        key: value
  - http_put:
      url: "https://example.com/api/data/1"
      save_as: put_resp
      content_type: text
      body: "plain text"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 3);
        let ScenarioStep::HttpGet(g) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(g.url, "https://example.com/api");
        assert_eq!(g.save_as, "resp");
        assert_eq!(g.headers.get("Authorization").unwrap(), "Bearer token123");
        assert_eq!(g.timeout_ms, 10000);
        let ScenarioStep::HttpPost(p) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(p.url, "https://example.com/api/data");
        assert_eq!(p.content_type, ContentType::Json);
        let ScenarioStep::HttpPut(u) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(u.content_type, ContentType::Text);
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_get_defaults() {
        let yaml = r#"
name: test
steps:
  - http_get:
      url: "https://example.com"
      save_as: result
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::HttpGet(g) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(g.timeout_ms, 30_000);
        assert!(g.headers.is_empty());
        assert!(g.auth.is_none());
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_delete_and_patch_parse() {
        let yaml = r#"
name: test
steps:
  - http_delete:
      url: "https://example.com/api/1"
      save_as: del_resp
      auth:
        type: bearer
        token: "mytoken"
  - http_patch:
      url: "https://example.com/api/1"
      save_as: patch_resp
      content_type: json
      body:
        status: active
      auth:
        type: basic
        user: alice
        password: secret
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 2);
        let ScenarioStep::HttpDelete(d) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(d.url, "https://example.com/api/1");
        assert_eq!(d.save_as, "del_resp");
        let Some(HttpAuth::Bearer { token }) = &d.auth else {
            panic!("expected bearer auth")
        };
        assert_eq!(token, "mytoken");

        let ScenarioStep::HttpPatch(p) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(p.content_type, ContentType::Json);
        let Some(HttpAuth::Basic { user, password }) = &p.auth else {
            panic!("expected basic auth")
        };
        assert_eq!(user, "alice");
        assert_eq!(password, "secret");
    }

    #[test]
    fn key_combo_step_parses() {
        let yaml = r#"
name: test
steps:
  - key_combo:
      keys: ["ctrl", "c"]
  - key_combo:
      keys: ["alt", "F4"]
  - key_combo:
      keys: ["ctrl", "shift", "tab"]
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 3);
        let ScenarioStep::KeyCombo(kc) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(kc.keys, vec!["ctrl", "c"]);
        let ScenarioStep::KeyCombo(kc2) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(kc2.keys, vec!["alt", "F4"]);
        let ScenarioStep::KeyCombo(kc3) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(kc3.keys.len(), 3);
    }

    #[test]
    fn csv_steps_parse() {
        let yaml = r#"
name: test
steps:
  - csv_read:
      path: "data.csv"
      has_header: true
      save_as: rows
  - csv_read:
      path: "raw.csv"
      has_header: false
      save_as: raw_rows
  - csv_write:
      path: "output.csv"
      rows: rows
      mode: overwrite
  - csv_write:
      path: "log.csv"
      rows: results
      headers: ["name", "value"]
      mode: append
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 4);
        let ScenarioStep::CsvRead(cr) = &s.steps[0] else {
            panic!()
        };
        assert!(cr.has_header);
        assert_eq!(cr.save_as, "rows");
        let ScenarioStep::CsvRead(cr2) = &s.steps[1] else {
            panic!()
        };
        assert!(!cr2.has_header);
        let ScenarioStep::CsvWrite(cw) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(cw.mode, CsvWriteMode::Overwrite);
        assert!(cw.headers.is_empty());
        let ScenarioStep::CsvWrite(cw2) = &s.steps[3] else {
            panic!()
        };
        assert_eq!(cw2.mode, CsvWriteMode::Append);
        assert_eq!(cw2.headers, vec!["name", "value"]);
    }

    #[test]
    fn csv_read_default_has_header() {
        let yaml = r#"
name: test
steps:
  - csv_read:
      path: "data.csv"
      save_as: rows
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::CsvRead(cr) = &s.steps[0] else {
            panic!()
        };
        assert!(cr.has_header);
    }

    #[test]
    fn csv_write_default_mode() {
        let yaml = r#"
name: test
steps:
  - csv_write:
      path: "out.csv"
      rows: data
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::CsvWrite(cw) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(cw.mode, CsvWriteMode::Overwrite);
    }

    #[test]
    fn file_read_write_steps_parse() {
        let yaml = r#"
name: test
steps:
  - file_read:
      path: "config.txt"
      save_as: content
  - file_write:
      path: "out/result.txt"
      content: "{{ output }}"
      mode: overwrite
  - file_write:
      path: "log.txt"
      content: "line\n"
      mode: append
  - file_append:
      path: "log.txt"
      content: "another line\n"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 4);
        let ScenarioStep::FileRead(fr) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(fr.path, "config.txt");
        assert_eq!(fr.save_as, "content");
        let ScenarioStep::FileWrite(fw) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(fw.mode, FileWriteMode::Overwrite);
        let ScenarioStep::FileWrite(fw2) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(fw2.mode, FileWriteMode::Append);
        let ScenarioStep::FileAppend(fa) = &s.steps[3] else {
            panic!()
        };
        assert_eq!(fa.path, "log.txt");
    }

    #[test]
    fn file_write_default_mode() {
        let yaml = r#"
name: test
steps:
  - file_write:
      path: "out.txt"
      content: "hello"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::FileWrite(fw) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(fw.mode, FileWriteMode::Overwrite);
    }

    #[test]
    fn process_steps_parse() {
        let yaml = r#"
name: test
steps:
  - process_start:
      command: "sleep"
      args: ["5"]
      wait_ms: 100
      save_pid_as: my_pid
  - process_exists:
      name: "sleep"
      save_as: running
  - process_kill:
      pid: "{{ my_pid }}"
  - process_kill:
      name: "notepad.exe"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 4);
        let ScenarioStep::ProcessStart(ps) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(ps.command, "sleep");
        assert_eq!(ps.args, vec!["5"]);
        assert_eq!(ps.wait_ms, 100);
        assert_eq!(ps.save_pid_as.as_deref(), Some("my_pid"));
        let ScenarioStep::ProcessExists(pe) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(pe.name, "sleep");
        assert_eq!(pe.save_as, "running");
        let ScenarioStep::ProcessKill(pk1) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(pk1.pid.as_deref(), Some("{{ my_pid }}"));
        assert!(pk1.name.is_none());
        let ScenarioStep::ProcessKill(pk2) = &s.steps[3] else {
            panic!()
        };
        assert!(pk2.pid.is_none());
        assert_eq!(pk2.name.as_deref(), Some("notepad.exe"));
    }

    #[test]
    fn process_start_defaults() {
        let yaml = r#"
name: test
steps:
  - process_start:
      command: "notepad.exe"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::ProcessStart(ps) = &s.steps[0] else {
            panic!()
        };
        assert!(ps.args.is_empty());
        assert_eq!(ps.wait_ms, 0);
        assert!(ps.save_pid_as.is_none());
    }

    #[test]
    fn excel_cell_steps_parse() {
        let yaml = r#"
name: test
steps:
  - excel_read_cell:
      file: "data.xlsx"
      sheet: "Sheet1"
      cell: "B5"
      save_as: val
  - excel_read_range:
      file: "data.xlsx"
      range: "A1:D10"
      save_as: table
  - excel_write_cell:
      file: "report.xlsx"
      sheet: "結果"
      cell: "C{{ row }}"
      value: "{{ amount }}"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 3);
        let ScenarioStep::ExcelReadCell(rc) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(rc.cell, "B5");
        assert_eq!(rc.sheet.as_deref(), Some("Sheet1"));
        let ScenarioStep::ExcelReadRange(rr) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(rr.range, "A1:D10");
        assert!(rr.sheet.is_none());
        let ScenarioStep::ExcelWriteCell(wc) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(wc.cell, "C{{ row }}");
    }

    #[test]
    fn mail_receive_step_parses() {
        let yaml = r#"
name: test
steps:
  - mail_receive:
      host: "imap.example.com"
      user: "user@example.com"
      password: "{{ env.MAIL_PASS }}"
      folder: "INBOX"
      count: 20
      only_unseen: true
      save_as: new_mails
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::MailReceive(mr) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(mr.host, "imap.example.com");
        assert_eq!(mr.count, 20);
        assert!(mr.only_unseen);
        assert_eq!(mr.save_as, "new_mails");
    }

    #[test]
    fn mail_receive_defaults() {
        let yaml = r#"
name: test
steps:
  - mail_receive:
      host: "imap.example.com"
      user: alice
      password: secret
      save_as: msgs
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::MailReceive(mr) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(mr.port, 993);
        assert_eq!(mr.folder, "INBOX");
        assert_eq!(mr.count, 10);
        assert!(!mr.only_unseen);
    }

    #[test]
    fn excel_sheet_management_steps_parse() {
        let yaml = r#"
name: test
steps:
  - excel_add_sheet:
      file: "book.xlsx"
      name: "Summary"
  - excel_delete_sheet:
      file: "book.xlsx"
      name: "OldSheet"
  - excel_rename_sheet:
      file: "book.xlsx"
      from_name: "Sheet1"
      to_name: "Data"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 3);
        let ScenarioStep::ExcelAddSheet(add) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(add.file, "book.xlsx");
        assert_eq!(add.name, "Summary");
        let ScenarioStep::ExcelDeleteSheet(del) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(del.name, "OldSheet");
        let ScenarioStep::ExcelRenameSheet(ren) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(ren.from_name, "Sheet1");
        assert_eq!(ren.to_name, "Data");
    }

    #[test]
    fn sprint8_steps_parse() {
        let yaml = r#"
name: test
steps:
  - screenshot_save:
      path: "caps/shot.png"
      window: "MyApp"
  - screenshot_save:
      path: "caps/full.png"
  - wait_no_image:
      template: "spinner.png"
      timeout_ms: 15000
      interval_ms: 300
  - url_open:
      url: "https://example.com"
  - notify:
      title: "Done"
      message: "{{ count }} rows"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 5);
        let ScenarioStep::ScreenshotSave(ss) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(ss.path, "caps/shot.png");
        assert_eq!(ss.window.as_deref(), Some("MyApp"));
        let ScenarioStep::ScreenshotSave(ss2) = &s.steps[1] else {
            panic!()
        };
        assert!(ss2.window.is_none());
        let ScenarioStep::WaitNoImage(wn) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(wn.template, "spinner.png");
        assert_eq!(wn.timeout_ms, 15000);
        assert_eq!(wn.interval_ms, 300);
        let ScenarioStep::UrlOpen(uo) = &s.steps[3] else {
            panic!()
        };
        assert_eq!(uo.url, "https://example.com");
        let ScenarioStep::Notify(n) = &s.steps[4] else {
            panic!()
        };
        assert_eq!(n.title, "Done");
        assert_eq!(n.message, "{{ count }} rows");
    }

    #[test]
    fn wait_no_image_defaults() {
        let yaml = r#"
name: test
steps:
  - wait_no_image:
      template: "loading.png"
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        let ScenarioStep::WaitNoImage(step) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(step.timeout_ms, 30_000);
        assert_eq!(step.interval_ms, 500);
        assert!(step.window.is_none());
    }
}
