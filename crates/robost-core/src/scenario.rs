use robost_template::Target;
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
    /// How long (ms) to sleep between reconnect poll attempts. Default: 1000ms.
    #[serde(default = "default_reconnect_retry_ms")]
    pub reconnect_retry_ms: u64,
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
    WaitChange(WaitChangeStep),
    WindowControl(WindowControlStep),

    // --- OCR ---
    OcrMatch(OcrMatchStep),
    ClickText(ClickTextStep),
    MoveToText(MoveToTextStep),

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
    FileSize(FileSizeStep),
    FileModifiedAt(FileModifiedAtStep),

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
    MailSend(MailSendStep),

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
    /// Wait until a UIA element reaches the specified state.
    UiaWait(UiaWaitStep),
    /// Select a named item in a ComboBox or ListBox.
    UiaSelect(UiaSelectStep),
    /// Get immediate children of a UIA element as a JSON array.
    UiaGetChildren(UiaGetChildrenStep),
    /// Set or clear a checkbox.
    UiaCheck(UiaCheckStep),

    // --- Pixel / colour ---
    /// Read the RGB colour of a screen pixel.
    GetPixelColor(GetPixelColorStep),
    /// Wait until a pixel matches the expected colour.
    WaitColor(WaitColorStep),

    // --- Window-relative click ---
    /// Click at coordinates relative to a window's top-left corner.
    ClickInWindow(ClickInWindowStep),

    // --- Directory operations ---
    DirCreate(DirCreateStep),
    DirDelete(DirDeleteStep),
    DirExists(DirExistsStep),

    // --- Process wait ---
    WaitProcess(WaitProcessStep),

    // --- Mouse hover ---
    MouseHover(MouseHoverStep),

    // --- Excel sheet-level read ---
    ExcelReadSheet(ExcelReadSheetStep),
    ExcelGetDims(ExcelGetDimsStep),
    ExcelFindRow(ExcelFindRowStep),

    // --- Excel range write ---
    ExcelWriteRange(ExcelWriteRangeStep),

    // --- String format ---
    StringFormat(StringFormatStep),

    // --- Base64 encode/decode ---
    Base64Encode(Base64EncodeStep),
    Base64Decode(Base64DecodeStep),

    // --- String query ---
    StringContains(StringContainsStep),
    StringStartsWith(StringStartsWithStep),
    StringEndsWith(StringEndsWithStep),
    StringIndexOf(StringIndexOfStep),

    // --- Type conversion ---
    ToNumber(ToNumberStep),
    ToString(ToStringStep),
    VarType(VarTypeStep),

    // --- List operations ---
    ListLength(ListLengthStep),
    ListGet(ListGetStep),
    ListPush(ListPushStep),
    ListRemove(ListRemoveStep),
    ListContains(ListContainsStep),

    // --- Number ---
    NumberRandom(NumberRandomStep),

    // --- String count ---
    StringCount(StringCountStep),

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
    /// Navigate back in browser history.
    WebNavigateBack,
    /// Navigate forward in browser history.
    WebNavigateForward,
    /// Wait until an element's text contains the expected string.
    WebWaitText(WebWaitTextStep),
    /// Select an option in a `<select>` element.
    WebSelect(WebSelectStep),
    /// Execute JavaScript in the browser.
    WebExecuteJs(WebExecuteJsStep),
    /// Switch the active frame.
    WebSwitchFrame(WebSwitchFrameStep),
    /// Scroll an element or the window.
    WebScroll(WebScrollStep),
    /// Handle a JavaScript alert/confirm/prompt.
    WebAlert(WebAlertStep),
    /// Get the current browser URL.
    WebGetUrl(WebGetUrlStep),
    /// Get the current page title.
    WebGetTitle(WebGetTitleStep),
    /// Get text or attribute from all elements matching a CSS selector.
    WebGetAll(WebGetAllStep),

    // --- DB ---
    /// Query rows from a database. Returns Vec<HashMap>.
    #[cfg(feature = "db")]
    DbQuery(DbQueryStep),
    /// Query a single row from a database.
    #[cfg(feature = "db")]
    DbQueryOne(DbQueryOneStep),
    /// Execute a non-returning SQL statement.
    #[cfg(feature = "db")]
    DbExecute(DbExecuteStep),

    // --- PDF ---
    PdfExtractText(PdfExtractTextStep),
    PdfPageCount(PdfPageCountStep),

    // --- ZIP ---
    ZipCompress(ZipCompressStep),
    ZipExtract(ZipExtractStep),
    ZipList(ZipListStep),

    // --- FTP ---
    #[cfg(feature = "ftp")]
    FtpUpload(FtpUploadStep),
    #[cfg(feature = "ftp")]
    FtpDownload(FtpDownloadStep),
    #[cfg(feature = "ftp")]
    FtpList(FtpListStep),
    #[cfg(feature = "ftp")]
    FtpDelete(FtpDeleteStep),
    #[cfg(feature = "ftp")]
    FtpMkdir(FtpMkdirStep),

    // --- wait_until ---
    WaitUntil(WaitUntilStep),

    /// A step that has `enabled: false` in the YAML. The engine skips it transparently.
    /// Serialization note: round-trips through `{<step_key>: ..., enabled: false}` YAML,
    /// not through the derived `{"disabled": ...}` form — use Scenario::from_yaml only.
    #[serde(skip)]
    Disabled(Box<ScenarioStep>),
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
    pub masks: Vec<robost_template::MaskRegion>,
    /// Number of consecutive successful matches required before accepting.
    /// 0 or 1 means a single match is sufficient (default behaviour).
    #[serde(default = "default_stable_frames")]
    pub stable_frames: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickImageStep {
    pub template: String,
    pub anchor: Option<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
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
    pub masks: Vec<robost_template::MaskRegion>,
    /// Milliseconds to wait after a successful click (for slow UI animations).
    #[serde(default)]
    pub post_click_ms: Option<u64>,
    /// Number of consecutive successful matches required before clicking.
    /// 0 or 1 means a single match is sufficient (default behaviour).
    #[serde(default = "default_stable_frames")]
    pub stable_frames: u8,
    /// How long (ms) to poll after clicking, waiting for the template to disappear.
    /// If absent, skip post-click verification.
    #[serde(default)]
    pub verify_gone_ms: Option<u64>,
    /// Maximum click-and-verify attempts. Default: 3.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
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
    /// Variable name to receive the current item (default: "item").
    #[serde(default)]
    pub item_var: Option<String>,
    /// Optional variable name to receive the 0-based loop index.
    #[serde(default)]
    pub index_var: Option<String>,
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
    /// Number of consecutive successful matches required before accepting.
    #[serde(default = "default_stable_frames")]
    pub stable_frames: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellStep {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub save_as: Option<String>,
    #[serde(default)]
    pub save_exit_code: Option<String>,
    #[serde(default)]
    pub fail_on_error: bool,
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
    /// Wait until the window exists AND is not hung (Windows: `IsHungAppWindow`).
    /// Falls back to `Exists` behaviour on non-Windows platforms.
    Operable,
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
    pub rect: robost_template::Rect,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// Stores `{found, x, y, score}`. If absent and not found, returns an error.
    pub save_as: Option<String>,
    /// Number of consecutive successful matches required before accepting.
    #[serde(default = "default_stable_frames")]
    pub stable_frames: u8,
}

/// Wait until the screen changes (pixel diff exceeds a threshold) within a region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitChangeStep {
    /// Restrict comparison to this screen-global region. If absent, compare full capture.
    #[serde(default)]
    pub region: Option<robost_template::Rect>,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Pixel-level change threshold (0–255). Default: 15.
    #[serde(default = "default_wait_change_threshold")]
    pub threshold: u8,
    /// Minimum fraction of pixels that must change before the step succeeds (0.0–1.0).
    /// Default: 0.001 (0.1%).
    #[serde(default = "default_wait_change_ratio")]
    pub min_ratio: f32,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
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

#[cfg(feature = "llm-ocr")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    #[default]
    Anthropic,
    Openai,
}

#[cfg(feature = "llm-ocr")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOcrConfig {
    #[serde(default)]
    pub provider: LlmProvider,
    /// Override the default model. Defaults: anthropic=`claude-3-5-haiku-20241022`, openai=`gpt-4o-mini`.
    #[serde(default)]
    pub model: Option<String>,
    /// Override the text extraction prompt.
    #[serde(default)]
    pub prompt: Option<String>,
}

#[cfg(feature = "ocrs-cjk-ocr")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrsCjkConfig {
    /// Directory containing `detection.rten/.onnx` and `recognition.rten/.onnx`.
    /// Falls back to `ROBOST_OCR_MODEL_DIR` env var, then `~/.robost/models`.
    #[serde(default)]
    pub model_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OcrEngineKind {
    Tesseract,
    /// ocrs-cjk: Pure Rust, offline, CJK-enhanced OCR (PP-OCRv5).
    #[cfg(feature = "ocrs-cjk-ocr")]
    OcrsCjk(OcrsCjkConfig),
    #[cfg(feature = "llm-ocr")]
    Llm(LlmOcrConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrMatchStep {
    /// Restrict OCR to this screen-global rectangle. `None` = full capture area.
    #[serde(default)]
    pub region: Option<robost_template::Rect>,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Text substring the OCR result must contain. `None` = any text passes.
    #[serde(default)]
    pub contains: Option<String>,
    /// Tesseract language code(s): `"eng"`, `"jpn"`, `"jpn+eng"`, etc. Ignored for llm engine.
    #[serde(default = "default_ocr_lang")]
    pub lang: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// Stores `{found: bool, text: str}`. If absent and text doesn't match, returns an error.
    pub save_as: Option<String>,
    /// OCR engine to use. Absent = Tesseract (backward compatible).
    #[serde(default)]
    pub engine: Option<OcrEngineKind>,
}

// ── Click text via OCR ────────────────────────────────────────────────────

/// Find the specified text on screen using OCR and click its center.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickTextStep {
    /// The text string to locate on screen.
    pub text: String,
    /// Tesseract language code(s): `"jpn"`, `"eng"`, `"jpn+eng"`, etc.
    #[serde(default = "default_ocr_lang")]
    pub lang: String,
    /// Which mouse button/gesture to use.
    #[serde(default)]
    pub action: ClickAction,
    /// Horizontal offset in logical pixels from the text center (positive = right).
    #[serde(default)]
    pub offset_x: i32,
    /// Vertical offset in logical pixels from the text center (positive = down).
    #[serde(default)]
    pub offset_y: i32,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    /// Keep retrying until the text appears or this many milliseconds elapse.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// OCR engine to use. Absent = Tesseract/WinRT (backward compatible).
    #[serde(default)]
    pub engine: Option<OcrEngineKind>,
}

// ── Move mouse to text via OCR ────────────────────────────────────────────

/// Find the specified text on screen using OCR and move the mouse to its center (no click).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveToTextStep {
    /// The text string to locate on screen.
    pub text: String,
    /// Tesseract language code(s): `"jpn"`, `"eng"`, `"jpn+eng"`, etc.
    #[serde(default = "default_ocr_lang")]
    pub lang: String,
    /// Horizontal offset in logical pixels from the text center (positive = right).
    #[serde(default)]
    pub offset_x: i32,
    /// Vertical offset in logical pixels from the text center (positive = down).
    #[serde(default)]
    pub offset_y: i32,
    /// Restrict capture to the window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// OCR engine to use. Absent = Tesseract/WinRT (backward compatible).
    #[serde(default)]
    pub engine: Option<OcrEngineKind>,
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

/// Direction for mouse scroll.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

impl ScrollDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// Scroll the mouse wheel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseScrollStep {
    #[serde(default)]
    pub direction: ScrollDirection,
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
    /// Match by control type name: "Button", "Edit", "Window", etc.
    ControlType(String),
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
    /// Scope the search to a window whose title contains this string.
    #[serde(default)]
    pub window: Option<String>,
}

/// Set a UIA element value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaSetStep {
    pub by: UiaBy,
    pub value: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

/// Invoke (click) a UIA element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaClickStep {
    pub by: UiaBy,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

/// Find a UIA element and store its bounding rect.
/// Output: { x, y, width, height, name }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaFindStep {
    pub by: UiaBy,
    pub save_as: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

fn default_uia_property() -> String {
    "value".to_owned()
}

/// State predicate for UIA wait.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UiaState {
    /// Element appears in the accessibility tree (default).
    #[default]
    Exists,
    /// Element is interactable.
    Enabled,
    /// Element is on-screen (not off-screen).
    Visible,
}

/// Wait until a UIA element reaches the specified state.
///
/// ```yaml
/// - uia_wait:
///     by: { name: "OK" }
///     state: enabled   # exists (default) | enabled | visible
///     timeout_ms: 10000
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaWaitStep {
    pub by: UiaBy,
    #[serde(default)]
    pub state: UiaState,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

/// Select a named item in a ComboBox or ListBox.
///
/// ```yaml
/// - uia_select:
///     by: { name: "Country" }
///     item: "Japan"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaSelectStep {
    pub by: UiaBy,
    pub item: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

/// Enumerate immediate children of a UIA element.
/// Saves `[{ name, value, class }]` to a variable.
///
/// ```yaml
/// - uia_get_children:
///     by: { name: "Files" }
///     save_as: list_items
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaGetChildrenStep {
    pub by: UiaBy,
    pub save_as: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

/// Set or clear a checkbox via `IUIAutomationTogglePattern`.
///
/// ```yaml
/// - uia_check:
///     by: { name: "同意する" }
///     checked: true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiaCheckStep {
    pub by: UiaBy,
    pub checked: bool,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

// ── Pixel / colour step types ─────────────────────────────────────────────

/// Read the RGB colour of a screen pixel.
/// Saves `{ r, g, b, hex }` to a variable.
///
/// ```yaml
/// - get_pixel_color:
///     x: 500
///     y: 300
///     save_as: col
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPixelColorStep {
    pub x: i32,
    pub y: i32,
    /// Restrict capture to this window (title substring). Coordinates remain screen-global.
    #[serde(default)]
    pub window: Option<String>,
    pub save_as: String,
}

/// Wait until the pixel at `(x, y)` matches `color` within `tolerance`.
///
/// ```yaml
/// - wait_color:
///     x: 500
///     y: 300
///     color: "#00FF00"
///     tolerance: 10
///     timeout_ms: 10000
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitColorStep {
    pub x: i32,
    pub y: i32,
    /// Expected colour in `#RRGGBB` format.
    pub color: String,
    /// Per-channel tolerance (0–255). Default 10.
    #[serde(default = "default_color_tolerance")]
    pub tolerance: u8,
    #[serde(default = "default_wait_color_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub window: Option<String>,
}

fn default_color_tolerance() -> u8 {
    10
}

fn default_wait_color_timeout_ms() -> u64 {
    10_000
}

// ── Window-relative click step type ──────────────────────────────────────

/// Click at a position specified relative to the top-left corner of a window.
///
/// ```yaml
/// - click_in_window:
///     window: "メモ帳"
///     x: 100
///     y: 50
///     action: left   # left (default) | right | double
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickInWindowStep {
    pub window: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub action: ClickAction,
}

// ── Directory operation step types ───────────────────────────────────────

/// Create a directory (including all parent directories).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirCreateStep {
    pub path: String,
}

/// Delete a directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirDeleteStep {
    pub path: String,
    /// Remove the directory and all its contents (default: false).
    #[serde(default)]
    pub recursive: bool,
    /// Do nothing if the directory does not exist (default: false).
    #[serde(default)]
    pub ignore_missing: bool,
}

/// Check whether a directory exists and save the result to a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirExistsStep {
    pub path: String,
    pub save_as: String,
}

// ── Process wait step type ────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessWaitState {
    /// Wait until the process is running.
    #[default]
    Started,
    /// Wait until the process has exited.
    Exited,
}

/// Wait until a process reaches the specified state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitProcessStep {
    pub name: String,
    #[serde(default)]
    pub state: ProcessWaitState,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
    /// If set, stores `true` (matched) or `false` (timed out) instead of erroring.
    pub save_as: Option<String>,
}

// ── Mouse hover step type ─────────────────────────────────────────────────

/// Move the cursor to `(x, y)` and dwell for `hover_ms` milliseconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseHoverStep {
    pub x: String,
    pub y: String,
    /// How long to remain at the position in milliseconds (default: 500).
    #[serde(default = "default_hover_ms")]
    pub hover_ms: u64,
}

fn default_hover_ms() -> u64 {
    500
}

// ── Excel sheet-level read step types ────────────────────────────────────

/// Read an entire worksheet as a list.
///
/// When `has_header` is true (default: false) the first row is used as keys and each
/// subsequent row becomes a `{col: value}` map.  Otherwise every row is a list of values.
///
/// ```yaml
/// - excel_read_sheet:
///     file: "data.xlsx"
///     sheet: "Sheet1"
///     has_header: true
///     save_as: rows
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelReadSheetStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    #[serde(default)]
    pub has_header: bool,
    pub save_as: String,
}

/// Return the dimensions (row count, column count) of a worksheet's used range.
///
/// ```yaml
/// - excel_get_dims:
///     file: "data.xlsx"
///     sheet: "Sheet1"
///     save_as: dims   # { rows: 10, cols: 3 }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelGetDimsStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    pub save_as: String,
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

/// Select an option in a `<select>` element by visible text or value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSelectStep {
    pub selector: String,
    pub item: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Execute JavaScript in the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebExecuteJsStep {
    pub script: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub save_as: Option<String>,
}

/// Action to take on a JS alert/confirm/prompt.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertAction {
    #[default]
    Accept,
    Dismiss,
    GetText,
}

/// Switch the active browsing context to an iframe or back to the top.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSwitchFrameStep {
    /// CSS selector of the iframe element to enter.
    #[serde(default)]
    pub selector: Option<String>,
    /// Zero-based index of the iframe to enter.
    #[serde(default)]
    pub index: Option<u16>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Scroll an element (or the window) by pixel offsets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScrollStep {
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Handle a JS alert/confirm/prompt dialog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAlertStep {
    #[serde(default)]
    pub action: AlertAction,
    pub save_as: Option<String>,
}

/// Wait for an element's text to contain `text`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebWaitTextStep {
    pub selector: String,
    pub text: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Check if a string contains a substring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringContainsStep {
    pub value: String,
    pub search: String,
    #[serde(default)]
    pub ignore_case: bool,
    pub save_as: String,
}

/// Check if a string starts with a prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringStartsWithStep {
    pub value: String,
    pub search: String,
    #[serde(default)]
    pub ignore_case: bool,
    pub save_as: String,
}

/// Check if a string ends with a suffix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringEndsWithStep {
    pub value: String,
    pub search: String,
    #[serde(default)]
    pub ignore_case: bool,
    pub save_as: String,
}

/// Find the first occurrence position of a substring (0-based; -1 if not found).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringIndexOfStep {
    pub value: String,
    pub search: String,
    #[serde(default)]
    pub ignore_case: bool,
    pub save_as: String,
}

/// Parse a string or boolean value into a JSON number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToNumberStep {
    pub value: String,
    pub save_as: String,
    /// Fallback value when parsing fails. If absent, a parse failure is an error.
    #[serde(default)]
    pub default: Option<f64>,
}

/// Convert any value to its string representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToStringStep {
    pub value: String,
    pub save_as: String,
}

/// Return the JSON type name of a variable ("string", "number", "bool", "array", "object", "null").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarTypeStep {
    pub value: String,
    pub save_as: String,
}

/// Get the number of elements in a list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLengthStep {
    pub list: String,
    pub save_as: String,
}

/// Get an element from a list by zero-based index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGetStep {
    pub list: String,
    pub index: String,
    pub save_as: String,
}

/// Append a value to the end of a list (in-place update).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPushStep {
    pub list: String,
    pub value: String,
}

/// Remove the element at a zero-based index from a list (in-place update).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRemoveStep {
    pub list: String,
    pub index: String,
}

/// Check if a list contains a value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContainsStep {
    pub list: String,
    pub value: String,
    pub save_as: String,
}

/// Generate a random number in `[min, max]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberRandomStep {
    #[serde(default)]
    pub min: f64,
    #[serde(default = "default_random_max")]
    pub max: f64,
    #[serde(default)]
    pub integer: bool,
    pub save_as: String,
}

fn default_random_max() -> f64 {
    1.0
}

/// Write a 2-D array of values to an Excel range starting at `cell`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelWriteRangeStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    /// Top-left cell reference, e.g. `"A2"`.
    pub cell: String,
    /// Variable name holding a `list<list<value>>` or `list<map<str,value>>`.
    pub data: String,
}

/// Build a string by substituting `{0}`, `{1}`, … placeholders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringFormatStep {
    pub format: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub save_as: String,
}

/// Base64-encode a string value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64EncodeStep {
    pub value: String,
    pub save_as: String,
}

/// Base64-decode a string value (UTF-8 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64DecodeStep {
    pub value: String,
    pub save_as: String,
}

/// Send an email via SMTP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailSendStep {
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub user: String,
    pub password: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub cc: Option<String>,
    #[serde(default)]
    pub bcc: Option<String>,
}

fn default_smtp_port() -> u16 {
    587
}

/// Get the size of a file in bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSizeStep {
    pub path: String,
    pub save_as: String,
}

/// Get the last-modified timestamp of a file as a formatted string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModifiedAtStep {
    pub path: String,
    #[serde(default = "default_datetime_format")]
    pub format: String,
    pub save_as: String,
}

/// Find the first row in an Excel sheet where a column value matches.
///
/// ```yaml
/// - excel_find_row:
///     file: "data.xlsx"
///     col: "A"
///     value: "{{ search_val }}"
///     save_as: row_num   # 1-based; -1 if not found
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelFindRowStep {
    pub file: String,
    #[serde(default)]
    pub sheet: Option<String>,
    /// Column letter (A, B, C …).
    pub col: String,
    pub value: String,
    /// Skip the header row when searching (default: true).
    #[serde(default = "default_true")]
    pub has_header: bool,
    /// 1-based data row number of the first match, or -1 if not found.
    pub save_as: String,
}

/// Get the current browser URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGetUrlStep {
    pub save_as: String,
}

/// Get the current page title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGetTitleStep {
    pub save_as: String,
}

/// Get text or an attribute from all elements matching a CSS selector.
///
/// ```yaml
/// - web_get_all:
///     selector: ".result-item"
///     attr: "href"   # omit for innerText
///     save_as: links
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGetAllStep {
    pub selector: String,
    #[serde(default)]
    pub attr: Option<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    pub save_as: String,
}

/// Count occurrences of a substring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringCountStep {
    pub value: String,
    pub search: String,
    #[serde(default)]
    pub ignore_case: bool,
    pub save_as: String,
}

// ── DB step types (feature = "db") ───────────────────────────────────────────

#[cfg(feature = "db")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbQueryStep {
    pub url: String,
    pub sql: String,
    #[serde(default)]
    pub params: Vec<serde_json::Value>,
    pub save_as: String,
}

#[cfg(feature = "db")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbQueryOneStep {
    pub url: String,
    pub sql: String,
    #[serde(default)]
    pub params: Vec<serde_json::Value>,
    pub save_as: String,
}

#[cfg(feature = "db")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbExecuteStep {
    pub url: String,
    pub sql: String,
    #[serde(default)]
    pub params: Vec<serde_json::Value>,
    #[serde(default)]
    pub save_as: Option<String>,
}

// ── PDF step types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfExtractTextStep {
    pub file: String,
    pub save_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageCountStep {
    pub file: String,
    pub save_as: String,
}

// ── ZIP step types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipCompressStep {
    pub dest: String,
    pub files: Vec<String>,
    #[serde(default)]
    pub save_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipExtractStep {
    pub src: String,
    pub dest: String,
    #[serde(default)]
    pub save_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipListStep {
    pub src: String,
    pub save_as: String,
}

// ── FTP step types (feature = "ftp") ─────────────────────────────────────────

#[cfg(feature = "ftp")]
fn default_ftp_port() -> u16 {
    21
}

#[cfg(feature = "ftp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpUploadStep {
    pub host: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_ftp_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    pub local: String,
    pub remote: String,
}

#[cfg(feature = "ftp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpDownloadStep {
    pub host: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_ftp_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    pub remote: String,
    pub local: String,
}

#[cfg(feature = "ftp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpListStep {
    pub host: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_ftp_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    pub remote: String,
    pub save_as: String,
}

#[cfg(feature = "ftp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpDeleteStep {
    pub host: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_ftp_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    pub remote: String,
}

#[cfg(feature = "ftp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpMkdirStep {
    pub host: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_ftp_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    pub remote: String,
}

// ── wait_until step type ──────────────────────────────────────────────────────

fn default_wait_until_timeout_ms() -> u64 {
    30_000
}
fn default_wait_until_interval_ms() -> u64 {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitUntilStep {
    pub cond: String,
    #[serde(default = "default_wait_until_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_wait_until_interval_ms")]
    pub interval_ms: u64,
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
fn default_stable_frames() -> u8 {
    1
}
fn default_max_retries() -> u32 {
    3
}
fn default_reconnect_retry_ms() -> u64 {
    1000
}
fn default_wait_change_threshold() -> u8 {
    15
}
fn default_wait_change_ratio() -> f32 {
    0.001
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
    "wait_change",
    "window_control",
    "ocr_match",
    "click_text",
    "move_to_text",
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
    "uia_wait",
    "uia_select",
    "uia_get_children",
    "uia_check",
    "get_pixel_color",
    "wait_color",
    "click_in_window",
    "web_open",
    "web_click",
    "web_type",
    "web_get",
    "web_wait",
    "web_screenshot",
    "web_close",
    "dir_create",
    "dir_delete",
    "dir_exists",
    "wait_process",
    "mouse_hover",
    "excel_read_sheet",
    "excel_get_dims",
    "excel_write_range",
    "string_format",
    "base64_encode",
    "base64_decode",
    "web_select",
    "web_execute_js",
    "web_switch_frame",
    "web_scroll",
    "web_alert",
    "web_navigate_back",
    "web_navigate_forward",
    "web_wait_text",
    "string_contains",
    "string_starts_with",
    "string_ends_with",
    "string_index_of",
    "to_number",
    "to_string",
    "var_type",
    "list_length",
    "list_get",
    "list_push",
    "list_remove",
    "list_contains",
    "number_random",
    "string_count",
    "mail_send",
    "file_size",
    "file_modified_at",
    "excel_find_row",
    "web_get_url",
    "web_get_title",
    "web_get_all",
    "db_query",
    "db_query_one",
    "db_execute",
    "pdf_extract_text",
    "pdf_page_count",
    "zip_compress",
    "zip_extract",
    "zip_list",
    "ftp_upload",
    "ftp_download",
    "ftp_list",
    "ftp_delete",
    "ftp_mkdir",
    "wait_until",
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
            "wait_change" => ScenarioStep::WaitChange(map.next_value()?),
            "window_control" => ScenarioStep::WindowControl(map.next_value()?),
            "ocr_match" => ScenarioStep::OcrMatch(map.next_value()?),
            "click_text" => ScenarioStep::ClickText(map.next_value()?),
            "move_to_text" => ScenarioStep::MoveToText(map.next_value()?),
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
            "file_size" => ScenarioStep::FileSize(map.next_value()?),
            "file_modified_at" => ScenarioStep::FileModifiedAt(map.next_value()?),
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
            "mail_send" => ScenarioStep::MailSend(map.next_value()?),
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
            "uia_wait" => ScenarioStep::UiaWait(map.next_value()?),
            "uia_select" => ScenarioStep::UiaSelect(map.next_value()?),
            "uia_get_children" => ScenarioStep::UiaGetChildren(map.next_value()?),
            "uia_check" => ScenarioStep::UiaCheck(map.next_value()?),
            // --- pixel / colour ---
            "get_pixel_color" => ScenarioStep::GetPixelColor(map.next_value()?),
            "wait_color" => ScenarioStep::WaitColor(map.next_value()?),
            // --- window-relative click ---
            "click_in_window" => ScenarioStep::ClickInWindow(map.next_value()?),
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
            // --- directory operations ---
            "dir_create" => ScenarioStep::DirCreate(map.next_value()?),
            "dir_delete" => ScenarioStep::DirDelete(map.next_value()?),
            "dir_exists" => ScenarioStep::DirExists(map.next_value()?),
            // --- process wait ---
            "wait_process" => ScenarioStep::WaitProcess(map.next_value()?),
            // --- mouse hover ---
            "mouse_hover" => ScenarioStep::MouseHover(map.next_value()?),
            // --- excel sheet-level read ---
            "excel_read_sheet" => ScenarioStep::ExcelReadSheet(map.next_value()?),
            "excel_get_dims" => ScenarioStep::ExcelGetDims(map.next_value()?),
            "excel_find_row" => ScenarioStep::ExcelFindRow(map.next_value()?),
            // --- excel range write ---
            "excel_write_range" => ScenarioStep::ExcelWriteRange(map.next_value()?),
            // --- string format ---
            "string_format" => ScenarioStep::StringFormat(map.next_value()?),
            // --- base64 ---
            "base64_encode" => ScenarioStep::Base64Encode(map.next_value()?),
            "base64_decode" => ScenarioStep::Base64Decode(map.next_value()?),
            // --- web tier 3 ---
            "web_select" => ScenarioStep::WebSelect(map.next_value()?),
            "web_execute_js" => ScenarioStep::WebExecuteJs(map.next_value()?),
            "web_switch_frame" => ScenarioStep::WebSwitchFrame(map.next_value()?),
            "web_scroll" => ScenarioStep::WebScroll(map.next_value()?),
            "web_alert" => ScenarioStep::WebAlert(map.next_value()?),
            // --- web tier 4 ---
            "web_navigate_back" => {
                let _: serde::de::IgnoredAny = map.next_value()?;
                ScenarioStep::WebNavigateBack
            }
            "web_navigate_forward" => {
                let _: serde::de::IgnoredAny = map.next_value()?;
                ScenarioStep::WebNavigateForward
            }
            "web_wait_text" => ScenarioStep::WebWaitText(map.next_value()?),
            // --- string query ---
            "string_contains" => ScenarioStep::StringContains(map.next_value()?),
            "string_starts_with" => ScenarioStep::StringStartsWith(map.next_value()?),
            "string_ends_with" => ScenarioStep::StringEndsWith(map.next_value()?),
            "string_index_of" => ScenarioStep::StringIndexOf(map.next_value()?),
            // --- type conversion ---
            "to_number" => ScenarioStep::ToNumber(map.next_value()?),
            "to_string" => ScenarioStep::ToString(map.next_value()?),
            "var_type" => ScenarioStep::VarType(map.next_value()?),
            // --- list operations ---
            "list_length" => ScenarioStep::ListLength(map.next_value()?),
            "list_get" => ScenarioStep::ListGet(map.next_value()?),
            "list_push" => ScenarioStep::ListPush(map.next_value()?),
            "list_remove" => ScenarioStep::ListRemove(map.next_value()?),
            "list_contains" => ScenarioStep::ListContains(map.next_value()?),
            // --- number ---
            "number_random" => ScenarioStep::NumberRandom(map.next_value()?),
            // --- string count ---
            "string_count" => ScenarioStep::StringCount(map.next_value()?),
            // --- web tier 5 ---
            "web_get_url" => ScenarioStep::WebGetUrl(map.next_value()?),
            "web_get_title" => ScenarioStep::WebGetTitle(map.next_value()?),
            "web_get_all" => ScenarioStep::WebGetAll(map.next_value()?),
            // --- DB ---
            #[cfg(feature = "db")]
            "db_query" => ScenarioStep::DbQuery(map.next_value()?),
            #[cfg(feature = "db")]
            "db_query_one" => ScenarioStep::DbQueryOne(map.next_value()?),
            #[cfg(feature = "db")]
            "db_execute" => ScenarioStep::DbExecute(map.next_value()?),
            // --- PDF ---
            "pdf_extract_text" => ScenarioStep::PdfExtractText(map.next_value()?),
            "pdf_page_count" => ScenarioStep::PdfPageCount(map.next_value()?),
            // --- ZIP ---
            "zip_compress" => ScenarioStep::ZipCompress(map.next_value()?),
            "zip_extract" => ScenarioStep::ZipExtract(map.next_value()?),
            "zip_list" => ScenarioStep::ZipList(map.next_value()?),
            // --- FTP ---
            #[cfg(feature = "ftp")]
            "ftp_upload" => ScenarioStep::FtpUpload(map.next_value()?),
            #[cfg(feature = "ftp")]
            "ftp_download" => ScenarioStep::FtpDownload(map.next_value()?),
            #[cfg(feature = "ftp")]
            "ftp_list" => ScenarioStep::FtpList(map.next_value()?),
            #[cfg(feature = "ftp")]
            "ftp_delete" => ScenarioStep::FtpDelete(map.next_value()?),
            #[cfg(feature = "ftp")]
            "ftp_mkdir" => ScenarioStep::FtpMkdir(map.next_value()?),
            // --- wait_until ---
            "wait_until" => ScenarioStep::WaitUntil(map.next_value()?),
            other => return Err(A::Error::unknown_variant(other, KNOWN_VARIANTS)),
        };

        // Consume remaining top-level metadata entries after the step key-value.
        // `enabled: false` marks the step as disabled; other unknown keys are ignored
        // for forward compatibility.
        let mut enabled = true;
        while let Some(meta_key) = map.next_key::<String>()? {
            match meta_key.as_str() {
                "enabled" => {
                    enabled = map.next_value::<bool>().unwrap_or(true);
                }
                _ => {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        if enabled {
            Ok(step)
        } else {
            Ok(ScenarioStep::Disabled(Box::new(step)))
        }
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
    /// Returns true for composite/flow-control steps that manage their own inner retries.
    pub fn is_flow(&self) -> bool {
        match self {
            Self::Disabled(_) => false,
            _ => matches!(
                self,
                Self::Group(_)
                    | Self::If(_)
                    | Self::Switch(_)
                    | Self::Repeat(_)
                    | Self::While(_)
                    | Self::DoWhile(_)
                    | Self::TryCatch(_)
                    | Self::Foreach(_)
                    | Self::SubScenario(_)
                    | Self::CallScenario(_)
            ),
        }
    }

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
            Self::WaitChange(_) => "wait_change",
            Self::WindowControl(_) => "window_control",
            Self::OcrMatch(_) => "ocr_match",
            Self::ClickText(_) => "click_text",
            Self::MoveToText(_) => "move_to_text",
            Self::MlDetect(_) => "ml_detect",
            Self::ImportVars(_) => "import_vars",
            Self::SaveVars(_) => "save_vars",
            Self::LoadVars(_) => "load_vars",
            Self::FileExists(_) => "file_exists",
            Self::FileCopy(_) => "file_copy",
            Self::FileMove(_) => "file_move",
            Self::FileDelete(_) => "file_delete",
            Self::FileRename(_) => "file_rename",
            Self::FileSize(_) => "file_size",
            Self::FileModifiedAt(_) => "file_modified_at",
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
            Self::MailSend(_) => "mail_send",
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
            Self::UiaWait(_) => "uia_wait",
            Self::UiaSelect(_) => "uia_select",
            Self::UiaGetChildren(_) => "uia_get_children",
            Self::UiaCheck(_) => "uia_check",
            Self::GetPixelColor(_) => "get_pixel_color",
            Self::WaitColor(_) => "wait_color",
            Self::ClickInWindow(_) => "click_in_window",
            Self::WebOpen(_) => "web_open",
            Self::WebClick(_) => "web_click",
            Self::WebType(_) => "web_type",
            Self::WebGet(_) => "web_get",
            Self::WebWait(_) => "web_wait",
            Self::WebScreenshot(_) => "web_screenshot",
            Self::WebClose => "web_close",
            Self::DirCreate(_) => "dir_create",
            Self::DirDelete(_) => "dir_delete",
            Self::DirExists(_) => "dir_exists",
            Self::WaitProcess(_) => "wait_process",
            Self::MouseHover(_) => "mouse_hover",
            Self::ExcelReadSheet(_) => "excel_read_sheet",
            Self::ExcelGetDims(_) => "excel_get_dims",
            Self::ExcelFindRow(_) => "excel_find_row",
            Self::ExcelWriteRange(_) => "excel_write_range",
            Self::StringFormat(_) => "string_format",
            Self::Base64Encode(_) => "base64_encode",
            Self::Base64Decode(_) => "base64_decode",
            Self::WebSelect(_) => "web_select",
            Self::WebExecuteJs(_) => "web_execute_js",
            Self::WebSwitchFrame(_) => "web_switch_frame",
            Self::WebScroll(_) => "web_scroll",
            Self::WebAlert(_) => "web_alert",
            Self::WebNavigateBack => "web_navigate_back",
            Self::WebNavigateForward => "web_navigate_forward",
            Self::WebWaitText(_) => "web_wait_text",
            Self::StringContains(_) => "string_contains",
            Self::StringStartsWith(_) => "string_starts_with",
            Self::StringEndsWith(_) => "string_ends_with",
            Self::StringIndexOf(_) => "string_index_of",
            Self::ToNumber(_) => "to_number",
            Self::ToString(_) => "to_string",
            Self::VarType(_) => "var_type",
            Self::ListLength(_) => "list_length",
            Self::ListGet(_) => "list_get",
            Self::ListPush(_) => "list_push",
            Self::ListRemove(_) => "list_remove",
            Self::ListContains(_) => "list_contains",
            Self::NumberRandom(_) => "number_random",
            Self::StringCount(_) => "string_count",
            Self::WebGetUrl(_) => "web_get_url",
            Self::WebGetTitle(_) => "web_get_title",
            Self::WebGetAll(_) => "web_get_all",
            #[cfg(feature = "db")]
            Self::DbQuery(_) => "db_query",
            #[cfg(feature = "db")]
            Self::DbQueryOne(_) => "db_query_one",
            #[cfg(feature = "db")]
            Self::DbExecute(_) => "db_execute",
            Self::PdfExtractText(_) => "pdf_extract_text",
            Self::PdfPageCount(_) => "pdf_page_count",
            Self::ZipCompress(_) => "zip_compress",
            Self::ZipExtract(_) => "zip_extract",
            Self::ZipList(_) => "zip_list",
            #[cfg(feature = "ftp")]
            Self::FtpUpload(_) => "ftp_upload",
            #[cfg(feature = "ftp")]
            Self::FtpDownload(_) => "ftp_download",
            #[cfg(feature = "ftp")]
            Self::FtpList(_) => "ftp_list",
            #[cfg(feature = "ftp")]
            Self::FtpDelete(_) => "ftp_delete",
            #[cfg(feature = "ftp")]
            Self::FtpMkdir(_) => "ftp_mkdir",
            Self::WaitUntil(_) => "wait_until",
            Self::Disabled(inner) => inner.name(),
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
        assert_eq!(ms.direction, ScrollDirection::Up);
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
        assert_eq!(ms.direction, ScrollDirection::Down);
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

    #[test]
    fn tier2_steps_parse() {
        let yaml = r#"
name: test
steps:
  - dir_create:
      path: "output/2024"
  - dir_delete:
      path: "tmp"
      recursive: true
      ignore_missing: true
  - dir_exists:
      path: "logs"
      save_as: has_logs
  - wait_process:
      name: "notepad.exe"
      state: started
      timeout_ms: 5000
      save_as: np_ready
  - mouse_hover:
      x: "{{ btn_x }}"
      y: "{{ btn_y }}"
      hover_ms: 800
  - excel_read_sheet:
      file: "data.xlsx"
      sheet: "Sheet1"
      has_header: true
      save_as: rows
  - excel_get_dims:
      file: "data.xlsx"
      save_as: dims
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 7);

        let ScenarioStep::DirCreate(dc) = &s.steps[0] else {
            panic!()
        };
        assert_eq!(dc.path, "output/2024");

        let ScenarioStep::DirDelete(dd) = &s.steps[1] else {
            panic!()
        };
        assert!(dd.recursive);
        assert!(dd.ignore_missing);

        let ScenarioStep::DirExists(de) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(de.save_as, "has_logs");

        let ScenarioStep::WaitProcess(wp) = &s.steps[3] else {
            panic!()
        };
        assert_eq!(wp.name, "notepad.exe");
        assert_eq!(wp.state, ProcessWaitState::Started);
        assert_eq!(wp.timeout_ms, 5000);
        assert_eq!(wp.save_as.as_deref(), Some("np_ready"));

        let ScenarioStep::MouseHover(mh) = &s.steps[4] else {
            panic!()
        };
        assert_eq!(mh.x, "{{ btn_x }}");
        assert_eq!(mh.hover_ms, 800);

        let ScenarioStep::ExcelReadSheet(ers) = &s.steps[5] else {
            panic!()
        };
        assert!(ers.has_header);
        assert_eq!(ers.sheet.as_deref(), Some("Sheet1"));

        let ScenarioStep::ExcelGetDims(egd) = &s.steps[6] else {
            panic!()
        };
        assert_eq!(egd.save_as, "dims");
        assert!(egd.sheet.is_none());
    }

    #[test]
    fn tier2_defaults() {
        let yaml = r#"
name: test
steps:
  - dir_delete:
      path: "tmp"
  - wait_process:
      name: "app.exe"
  - mouse_hover:
      x: "100"
      y: "200"
  - excel_read_sheet:
      file: "data.xlsx"
      save_as: rows
"#;
        let s = Scenario::from_yaml(yaml).unwrap();

        let ScenarioStep::DirDelete(dd) = &s.steps[0] else {
            panic!()
        };
        assert!(!dd.recursive);
        assert!(!dd.ignore_missing);

        let ScenarioStep::WaitProcess(wp) = &s.steps[1] else {
            panic!()
        };
        assert_eq!(wp.state, ProcessWaitState::Started);
        assert_eq!(wp.timeout_ms, 5_000);
        assert_eq!(wp.retry_interval_ms, 200);
        assert!(wp.save_as.is_none());

        let ScenarioStep::MouseHover(mh) = &s.steps[2] else {
            panic!()
        };
        assert_eq!(mh.hover_ms, 500);

        let ScenarioStep::ExcelReadSheet(ers) = &s.steps[3] else {
            panic!()
        };
        assert!(!ers.has_header);
        assert!(ers.sheet.is_none());
    }

    #[test]
    fn tier3_steps_parse() {
        let yaml = r##"
name: test
steps:
  - web_select:
      selector: "select#country"
      item: "日本"
  - web_execute_js:
      script: "return arguments[0].innerText;"
      args: ["hello"]
      save_as: result
  - web_switch_frame:
      selector: "iframe#content"
  - web_switch_frame:
      index: 1
  - web_switch_frame: {}
  - web_scroll:
      selector: "#list"
      x: 0
      y: 300
  - web_alert:
      action: get_text
      save_as: msg
  - excel_write_range:
      file: "out.xlsx"
      cell: "A2"
      data: rows
  - string_format:
      format: "Hello, {0}!"
      args: ["world"]
      save_as: greeting
  - base64_encode:
      value: "hello"
      save_as: encoded
  - base64_decode:
      value: "aGVsbG8="
      save_as: decoded
"##;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 11);

        let ScenarioStep::WebSelect(ws) = &s.steps[0] else {
            panic!("0")
        };
        assert_eq!(ws.selector, "select#country");
        assert_eq!(ws.item, "日本");
        assert_eq!(ws.timeout_ms, 5_000);

        let ScenarioStep::WebExecuteJs(wej) = &s.steps[1] else {
            panic!("1")
        };
        assert_eq!(wej.script, "return arguments[0].innerText;");
        assert_eq!(wej.args, vec!["hello"]);
        assert_eq!(wej.save_as.as_deref(), Some("result"));

        let ScenarioStep::WebSwitchFrame(wsf) = &s.steps[2] else {
            panic!("2")
        };
        assert_eq!(wsf.selector.as_deref(), Some("iframe#content"));
        assert!(wsf.index.is_none());

        let ScenarioStep::WebSwitchFrame(wsf_idx) = &s.steps[3] else {
            panic!("3")
        };
        assert_eq!(wsf_idx.index, Some(1u16));
        assert!(wsf_idx.selector.is_none());

        let ScenarioStep::WebSwitchFrame(wsf_main) = &s.steps[4] else {
            panic!("4")
        };
        assert!(wsf_main.selector.is_none());
        assert!(wsf_main.index.is_none());

        let ScenarioStep::WebScroll(wsc) = &s.steps[5] else {
            panic!("5")
        };
        assert_eq!(wsc.selector.as_deref(), Some("#list"));
        assert_eq!(wsc.x, 0);
        assert_eq!(wsc.y, 300);

        let ScenarioStep::WebAlert(wa) = &s.steps[6] else {
            panic!("6")
        };
        assert_eq!(wa.action, AlertAction::GetText);
        assert_eq!(wa.save_as.as_deref(), Some("msg"));

        let ScenarioStep::ExcelWriteRange(ewr) = &s.steps[7] else {
            panic!("7")
        };
        assert_eq!(ewr.cell, "A2");
        assert_eq!(ewr.data, "rows");
        assert!(ewr.sheet.is_none());

        let ScenarioStep::StringFormat(sf) = &s.steps[8] else {
            panic!("8")
        };
        assert_eq!(sf.format, "Hello, {0}!");
        assert_eq!(sf.args, vec!["world"]);
        assert_eq!(sf.save_as, "greeting");

        let ScenarioStep::Base64Encode(be) = &s.steps[9] else {
            panic!("9")
        };
        assert_eq!(be.value, "hello");
        assert_eq!(be.save_as, "encoded");

        let ScenarioStep::Base64Decode(bd) = &s.steps[10] else {
            panic!("10")
        };
        assert_eq!(bd.value, "aGVsbG8=");
        assert_eq!(bd.save_as, "decoded");
    }

    #[test]
    fn tier4_steps_parse() {
        let yaml = r##"
name: test
steps:
  - string_contains:
      value: "hello world"
      search: "world"
      save_as: found
  - string_starts_with:
      value: "hello"
      search: "hel"
      ignore_case: true
      save_as: sw
  - string_ends_with:
      value: "hello"
      search: "LLO"
      ignore_case: true
      save_as: ew
  - string_index_of:
      value: "hello world"
      search: "world"
      save_as: pos
  - to_number:
      value: "3.14"
      save_as: pi
      default: 0.0
  - to_string:
      value: "{{ count }}"
      save_as: count_str
  - var_type:
      value: my_var
      save_as: type_name
  - list_length:
      list: rows
      save_as: n
  - list_get:
      list: rows
      index: "0"
      save_as: first
  - list_push:
      list: items
      value: "new"
  - list_remove:
      list: items
      index: "0"
  - list_contains:
      list: tags
      value: "urgent"
      save_as: is_urgent
  - number_random:
      min: 1
      max: 100
      integer: true
      save_as: rand_val
  - web_navigate_back: ~
  - web_navigate_forward: ~
  - web_wait_text:
      selector: ".status"
      text: "完了"
      timeout_ms: 10000
"##;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 16);

        let ScenarioStep::StringContains(sc) = &s.steps[0] else {
            panic!("0")
        };
        assert_eq!(sc.value, "hello world");
        assert_eq!(sc.search, "world");
        assert!(!sc.ignore_case);
        assert_eq!(sc.save_as, "found");

        let ScenarioStep::StringStartsWith(ssw) = &s.steps[1] else {
            panic!("1")
        };
        assert!(ssw.ignore_case);

        let ScenarioStep::StringEndsWith(sew) = &s.steps[2] else {
            panic!("2")
        };
        assert!(sew.ignore_case);

        let ScenarioStep::StringIndexOf(sio) = &s.steps[3] else {
            panic!("3")
        };
        assert_eq!(sio.search, "world");
        assert_eq!(sio.save_as, "pos");

        let ScenarioStep::ToNumber(tn) = &s.steps[4] else {
            panic!("4")
        };
        assert_eq!(tn.default, Some(0.0));

        let ScenarioStep::ToString(ts) = &s.steps[5] else {
            panic!("5")
        };
        assert_eq!(ts.save_as, "count_str");

        let ScenarioStep::VarType(vt) = &s.steps[6] else {
            panic!("6")
        };
        assert_eq!(vt.value, "my_var");

        let ScenarioStep::ListLength(ll) = &s.steps[7] else {
            panic!("7")
        };
        assert_eq!(ll.list, "rows");

        let ScenarioStep::ListGet(lg) = &s.steps[8] else {
            panic!("8")
        };
        assert_eq!(lg.index, "0");

        let ScenarioStep::ListPush(lp) = &s.steps[9] else {
            panic!("9")
        };
        assert_eq!(lp.value, "new");

        let ScenarioStep::ListRemove(lr) = &s.steps[10] else {
            panic!("10")
        };
        assert_eq!(lr.index, "0");

        let ScenarioStep::ListContains(lc) = &s.steps[11] else {
            panic!("11")
        };
        assert_eq!(lc.value, "urgent");

        let ScenarioStep::NumberRandom(nr) = &s.steps[12] else {
            panic!("12")
        };
        assert_eq!(nr.min, 1.0);
        assert_eq!(nr.max, 100.0);
        assert!(nr.integer);

        assert!(matches!(&s.steps[13], ScenarioStep::WebNavigateBack));
        assert!(matches!(&s.steps[14], ScenarioStep::WebNavigateForward));

        let ScenarioStep::WebWaitText(wwt) = &s.steps[15] else {
            panic!("15")
        };
        assert_eq!(wwt.selector, ".status");
        assert_eq!(wwt.text, "完了");
        assert_eq!(wwt.timeout_ms, 10000);
    }

    #[test]
    fn tier5_steps_parse() {
        let yaml = r#"
name: test
steps:
  - mail_send:
      host: "smtp.example.com"
      port: 587
      user: "bot@example.com"
      password: "secret"
      from: "bot@example.com"
      to: "user@example.com"
      subject: "Report"
      body: "Hello"
      cc: "cc@example.com"
  - file_size:
      path: "data.xlsx"
      save_as: sz
  - file_modified_at:
      path: "data.xlsx"
      format: "%Y-%m-%d"
      save_as: mtime
  - excel_find_row:
      file: "data.xlsx"
      sheet: "Sheet1"
      col: "A"
      value: "target"
      has_header: true
      save_as: row_num
  - string_count:
      value: "hello world hello"
      search: "hello"
      save_as: cnt
  - foreach:
      var: items
      index_var: idx
      do:
        - wait_ms: 1
  - web_get_url:
      save_as: url
  - web_get_title:
      save_as: title
  - web_get_all:
      selector: ".item"
      attr: "href"
      timeout_ms: 3000
      save_as: links
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 9);

        let ScenarioStep::MailSend(ms) = &s.steps[0] else {
            panic!("0")
        };
        assert_eq!(ms.host, "smtp.example.com");
        assert_eq!(ms.port, 587);
        assert_eq!(ms.cc.as_deref(), Some("cc@example.com"));
        assert!(ms.bcc.is_none());

        let ScenarioStep::FileSize(fs) = &s.steps[1] else {
            panic!("1")
        };
        assert_eq!(fs.path, "data.xlsx");
        assert_eq!(fs.save_as, "sz");

        let ScenarioStep::FileModifiedAt(fm) = &s.steps[2] else {
            panic!("2")
        };
        assert_eq!(fm.format, "%Y-%m-%d");
        assert_eq!(fm.save_as, "mtime");

        let ScenarioStep::ExcelFindRow(efr) = &s.steps[3] else {
            panic!("3")
        };
        assert_eq!(efr.col, "A");
        assert_eq!(efr.value, "target");
        assert!(efr.has_header);
        assert_eq!(efr.save_as, "row_num");

        let ScenarioStep::StringCount(sc) = &s.steps[4] else {
            panic!("4")
        };
        assert_eq!(sc.value, "hello world hello");
        assert_eq!(sc.search, "hello");
        assert!(!sc.ignore_case);
        assert_eq!(sc.save_as, "cnt");

        let ScenarioStep::Foreach(fe) = &s.steps[5] else {
            panic!("5")
        };
        assert_eq!(fe.var, "items");
        assert_eq!(fe.index_var.as_deref(), Some("idx"));

        let ScenarioStep::WebGetUrl(wgu) = &s.steps[6] else {
            panic!("6")
        };
        assert_eq!(wgu.save_as, "url");

        let ScenarioStep::WebGetTitle(wgt) = &s.steps[7] else {
            panic!("7")
        };
        assert_eq!(wgt.save_as, "title");

        let ScenarioStep::WebGetAll(wga) = &s.steps[8] else {
            panic!("8")
        };
        assert_eq!(wga.selector, ".item");
        assert_eq!(wga.attr.as_deref(), Some("href"));
        assert_eq!(wga.timeout_ms, 3000);
        assert_eq!(wga.save_as, "links");
    }

    #[test]
    fn enabled_false_wraps_in_disabled() {
        let yaml = r#"
name: test
steps:
  - wait_ms: 500
  - click_image:
      template: "btn.png"
    enabled: false
  - wait_ms: 100
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert_eq!(s.steps.len(), 3);
        assert!(matches!(s.steps[0], ScenarioStep::WaitMs(500)));
        let ScenarioStep::Disabled(inner) = &s.steps[1] else {
            panic!("expected Disabled, got {:?}", s.steps[1].name());
        };
        assert_eq!(inner.name(), "click_image");
        assert!(matches!(s.steps[2], ScenarioStep::WaitMs(100)));
    }

    #[test]
    fn enabled_true_is_not_wrapped() {
        let yaml = r#"
name: test
steps:
  - wait_ms: 200
    enabled: true
"#;
        let s = Scenario::from_yaml(yaml).unwrap();
        assert!(matches!(s.steps[0], ScenarioStep::WaitMs(200)));
    }
}
