use std::io::BufRead;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::report::{ExecutionReport, Outcome, StepOutcome, StepRecord};

use xcap;

use chrono::Local;
use rpa_backend::Backend;
use rpa_vision::TemplateMatcher;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{error, info, instrument, warn};

use crate::scenario::{
    CalcStep, CallScenarioStep, ClickAction, ClickImageStep, ClipboardGetStep, ClipboardSetStep,
    CopyVarStep, CsvReadStep, CsvWriteMode, CsvWriteStep, DateAddStep, DateDiffStep,
    DateFormatStep, DialogInputStep, DialogSelectStep, DialogWaitStep, DiffUnit, DoWhileStep,
    EnvGetStep, ExcelAddSheetStep, ExcelDeleteSheetStep, ExcelReadCellStep, ExcelReadRangeStep,
    ExcelRenameSheetStep, ExcelWriteCellStep, FileAppendStep, FileCopyStep, FileDeleteStep,
    FileExistsStep, FileListStep, FileMoveStep, FileReadStep, FileRenameStep, FileWriteMode,
    FileWriteStep, FindImageStep, ForeachStep, GetDatetimeStep, GetUsernameStep, GroupStep, IfStep,
    ImportVarsStep, IncrementStep, JsonParseStep, JsonStringifyStep, KeyComboStep, LibraryStep,
    LoadVarsStep, LogLevel, LogWriteStep, MailReceiveStep, MatchRectStep, MlDetectStep,
    MouseClickXyStep, MouseDragStep, MouseMoveStep, MouseScrollStep, NotifyStep, OcrMatchStep,
    PathBasenameStep, PathDirnameStep, PathJoinStep, ProcessExistsStep, ProcessKillStep,
    ProcessStartStep, RepeatStep, SaveVarsStep, ScenarioStep, ScreenshotSaveStep, ScriptStep,
    ShellStep, StringJoinStep, StringRegexStep, StringReplaceStep, StringSplitStep,
    StringSubstringStep, StringTrimStep, SubScenarioStep, SwitchStep, TrimSide, TryCatchStep,
    TypeStep, UiaBy, UiaClickStep, UiaFindStep, UiaGetStep, UiaSetStep, UrlOpenStep, WaitImageStep,
    WaitNoImageStep, WaitWindowStep, WhileStep, WidthStep, WindowControlAction, WindowControlStep,
    WindowState,
};
#[cfg(feature = "http")]
use crate::scenario::{
    ContentType, HttpAuth, HttpDeleteStep, HttpGetStep, HttpPatchStep, HttpPostStep, HttpPutStep,
};
#[cfg(feature = "web")]
use crate::scenario::{
    WebClickStep, WebGetStep, WebOpenStep, WebScreenshotStep, WebTypeStep, WebWaitStep,
};
use crate::variables::Variables;
use crate::Scenario;

// ── Error ──────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("template not found within timeout: {0}")]
    Timeout(String),
    #[error("image load error: {path}: {source}")]
    ImageLoad {
        path: PathBuf,
        source: image::ImageError,
    },
    #[error("vision error: {0}")]
    Vision(#[from] rpa_vision::MatchError),
    #[error("backend error: {0}")]
    Backend(#[from] rpa_backend::BackendError),
    #[error("secret env var missing: {0}")]
    MissingSecret(String),
    #[error("script error: {0}")]
    Script(#[from] rpa_script::ScriptError),
    #[error("sub-scenario error: {0}")]
    SubScenario(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("shell timeout: {0}")]
    ShellTimeout(String),
    #[error("blocking task panicked: {0}")]
    TaskPanic(String),
    #[error("scenario execution cancelled")]
    Cancelled,
    #[error("data source error: {0}")]
    DataSource(#[from] crate::data_source::DataSourceError),
    #[error("csv export error: {0}")]
    CsvExport(#[from] csv::Error),
    #[error("xlsx export error: {0}")]
    XlsxExport(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

// ── Control flow signal returned by run_steps ──────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum Flow {
    Done,
    Break,
    Continue,
    Exit,
}

// ── Engine ─────────────────────────────────────────────────────────────────

pub struct ScenarioEngine {
    backend: Arc<dyn Backend>,
    matcher: TemplateMatcher,
    script_engine: rpa_script::ScriptEngine,
    base_dir: PathBuf,
    screenshot_dir: PathBuf,
    /// When true, dialog steps are auto-skipped / answered with defaults.
    silent: bool,
    /// How long (ms) to wait for a lost window to reappear (RDP reconnect).
    /// 0 = disabled. Overridden per-run by `scenario.reconnect_timeout_ms`.
    reconnect_timeout_ms: u64,
    /// Shared flag; set to true to cancel the in-progress run.
    cancelled: Arc<AtomicBool>,
    /// Pause before every step and wait for Enter (step-through debugger).
    debug_step: bool,
    /// Skip all actual input operations (click, type, press, drag, scroll).
    dry_run: bool,
    /// Pause at this specific step index (0-based), then resume normally.
    break_at: Option<usize>,
    /// Print all variables after each step.
    dump_vars: bool,
    /// Active browser session for web_* steps.
    #[cfg(feature = "web")]
    web_session: tokio::sync::Mutex<Option<rpa_web::WebSession>>,
    /// Output path for the execution report (None = disabled).
    report_path: Option<PathBuf>,
    /// Accumulated step records for the current run.
    report_records: tokio::sync::Mutex<Vec<StepRecord>>,
}

impl ScenarioEngine {
    pub fn new(backend: Arc<dyn Backend>, base_dir: PathBuf) -> Self {
        let screenshot_dir = base_dir.join("screenshots");
        Self {
            backend,
            matcher: TemplateMatcher::default(),
            script_engine: rpa_script::ScriptEngine::new(),
            base_dir,
            screenshot_dir,
            silent: false,
            reconnect_timeout_ms: 0,
            cancelled: Arc::new(AtomicBool::new(false)),
            debug_step: false,
            dry_run: false,
            break_at: None,
            dump_vars: false,
            #[cfg(feature = "web")]
            web_session: tokio::sync::Mutex::new(None),
            report_path: None,
            report_records: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// Set the reconnect timeout: how many ms to wait for a window to reappear
    /// after an RDP/VNC session disconnect before failing the step.
    pub fn with_reconnect_timeout(mut self, ms: u64) -> Self {
        self.reconnect_timeout_ms = ms;
        self
    }

    /// Share a cancellation flag with the engine. Setting it to `true` causes
    /// the running scenario to return `Err(EngineError::Cancelled)` at the
    /// next yield/sleep point.
    pub fn with_cancel(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancelled = flag;
        self
    }

    /// Pause before every step; user presses Enter to continue or 'q' to quit.
    pub fn with_debug_step(mut self, enable: bool) -> Self {
        self.debug_step = enable;
        self
    }

    /// Skip all actual input operations (click, type, press, mouse ops).
    pub fn with_dry_run(mut self, enable: bool) -> Self {
        self.dry_run = enable;
        self
    }

    /// Pause execution at the given step index (0-based).
    pub fn with_break_at(mut self, idx: Option<usize>) -> Self {
        self.break_at = idx;
        self
    }

    /// Print all variables to stderr after each step.
    pub fn with_dump_vars(mut self, enable: bool) -> Self {
        self.dump_vars = enable;
        self
    }

    /// Enable execution report output to `path` (.csv or .html detected by extension).
    pub fn with_report(mut self, path: PathBuf) -> Self {
        self.report_path = Some(path);
        self
    }

    fn check_cancelled(&self) -> Result<()> {
        if self.cancelled.load(Ordering::Relaxed) {
            Err(EngineError::Cancelled)
        } else {
            Ok(())
        }
    }

    async fn debug_pause(&self, idx: usize, step: &ScenarioStep, vars: &Variables) -> Result<()> {
        use tokio::io::AsyncBufReadExt;
        eprintln!("\n[DEBUG] step {idx}: {}", step.name());
        eprintln!("  vars: {}", vars.debug_dump());
        eprint!("  Press Enter to continue, 'q' to quit: ");
        let mut line = String::new();
        tokio::io::BufReader::new(tokio::io::stdin())
            .read_line(&mut line)
            .await
            .map_err(|e| EngineError::Other(format!("debug read: {e}")))?;
        if line.trim() == "q" {
            return Err(EngineError::Cancelled);
        }
        Ok(())
    }

    /// Run a full scenario, returning after all steps complete (or on error/exit).
    /// `from_step` skips steps before that index (0-based).
    /// `data_override` replaces `scenario.data_source.file` if set.
    #[instrument(name = "run_scenario", fields(name = %scenario.name), skip(self, scenario, vars))]
    pub async fn run_with_opts(
        &self,
        scenario: &Scenario,
        vars: &mut Variables,
        from_step: usize,
        data_override: Option<&std::path::Path>,
    ) -> Result<()> {
        let run_started = chrono::Local::now();

        // Reset accumulated records for this run.
        if self.report_path.is_some() {
            self.report_records.lock().await.clear();
        }

        for (k, v) in &scenario.variables {
            vars.set(k.clone(), v.clone());
        }

        // Load data_source.
        let ds_file = data_override.map(|p| p.to_path_buf()).or_else(|| {
            scenario
                .data_source
                .as_ref()
                .map(|ds| self.base_dir.join(&ds.file))
        });

        if let Some(path) = ds_file {
            let sheet = scenario
                .data_source
                .as_ref()
                .and_then(|ds| ds.sheet.as_deref());
            let rows = crate::data_source::load(&path, sheet)?;
            let json_rows: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| serde_json::Value::Object(row.into_iter().collect()))
                .collect();
            vars.set("__rows__".to_owned(), serde_json::Value::Array(json_rows));
            info!(file = %path.display(), "__rows__ loaded");
        }

        let steps = if from_step == 0 {
            &scenario.steps[..]
        } else {
            scenario.steps.get(from_step..).unwrap_or(&[])
        };

        let reconnect_ms = scenario
            .reconnect_timeout_ms
            .unwrap_or(self.reconnect_timeout_ms);

        let run_result = self.run_steps(steps, vars, reconnect_ms).await;
        let run_finished = chrono::Local::now();

        if let Some(report_path) = &self.report_path {
            let step_records = self.report_records.lock().await.clone();
            let outcome = match &run_result {
                Ok(_) => Outcome::Success,
                Err(e) => {
                    let failed_step = step_records
                        .iter()
                        .rev()
                        .find(|r| matches!(r.outcome, StepOutcome::Failed(_)))
                        .map(|r| r.index)
                        .unwrap_or(0);
                    Outcome::Failed {
                        step_index: failed_step,
                        message: e.to_string(),
                    }
                }
            };
            let report = ExecutionReport {
                scenario_name: scenario.name.clone(),
                started_at: run_started,
                finished_at: run_finished,
                steps: step_records,
                outcome,
            };
            let ext = report_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let write_result = if ext.eq_ignore_ascii_case("html") {
                report.write_html(report_path)
            } else {
                report.write_csv(report_path)
            };
            if let Err(e) = write_result {
                warn!(path = %report_path.display(), error = %e, "failed to write execution report");
            } else {
                info!(path = %report_path.display(), "execution report written");
            }
        }

        match run_result? {
            Flow::Done | Flow::Exit => Ok(()),
            Flow::Break => {
                warn!("break at top level (no enclosing loop)");
                Ok(())
            }
            Flow::Continue => {
                warn!("continue at top level (no enclosing loop)");
                Ok(())
            }
        }
    }

    /// Convenience wrapper with defaults (from_step=0, no data override).
    pub async fn run(&self, scenario: &Scenario, vars: &mut Variables) -> Result<()> {
        self.run_with_opts(scenario, vars, 0, None).await
    }

    /// Export `__rows__` from vars to a CSV file.
    pub fn export_rows_csv(vars: &Variables, path: &std::path::Path) -> Result<()> {
        let rows = match vars.get("__rows__") {
            Some(serde_json::Value::Array(arr)) => arr,
            _ => return Ok(()),
        };

        if rows.is_empty() {
            return Ok(());
        }

        // Collect headers from first row.
        let headers: Vec<String> = match &rows[0] {
            serde_json::Value::Object(map) => map.keys().cloned().collect(),
            _ => return Ok(()),
        };

        let mut wtr = csv::Writer::from_path(path)?;
        wtr.write_record(&headers)?;

        for row in rows {
            if let serde_json::Value::Object(map) = row {
                let record: Vec<String> = headers
                    .iter()
                    .map(|h| match map.get(h) {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    })
                    .collect();
                wtr.write_record(&record)?;
            }
        }
        wtr.flush()?;
        Ok(())
    }

    /// Export `__rows__` from vars to an XLSX file.
    pub fn export_rows_xlsx(vars: &Variables, path: &std::path::Path) -> Result<()> {
        use rust_xlsxwriter::Workbook;

        let rows = match vars.get("__rows__") {
            Some(serde_json::Value::Array(arr)) => arr,
            _ => return Ok(()),
        };
        if rows.is_empty() {
            return Ok(());
        }
        let headers: Vec<String> = match &rows[0] {
            serde_json::Value::Object(map) => map.keys().cloned().collect(),
            _ => return Ok(()),
        };

        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        for (col, h) in headers.iter().enumerate() {
            sheet
                .write_string(0, col as u16, h)
                .map_err(|e| EngineError::XlsxExport(e.to_string()))?;
        }
        for (row_idx, row) in rows.iter().enumerate() {
            if let serde_json::Value::Object(map) = row {
                for (col, h) in headers.iter().enumerate() {
                    let val = match map.get(h) {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    };
                    sheet
                        .write_string((row_idx + 1) as u32, col as u16, &val)
                        .map_err(|e| EngineError::XlsxExport(e.to_string()))?;
                }
            }
        }
        workbook
            .save(path)
            .map_err(|e| EngineError::XlsxExport(e.to_string()))?;
        Ok(())
    }

    // ── Step execution ──────────────────────────────────────────────────────

    fn run_steps<'a>(
        &'a self,
        steps: &'a [ScenarioStep],
        vars: &'a mut Variables,
        reconnect_ms: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Flow>> + 'a>> {
        Box::pin(async move {
            for (i, step) in steps.iter().enumerate() {
                self.check_cancelled()?;

                let span = tracing::info_span!("step", index = i);
                let _g = span.enter();

                if self.debug_step || self.break_at == Some(i) {
                    self.debug_pause(i, step, vars).await?;
                }

                let step_started = chrono::Local::now();
                let step_timer = Instant::now();

                // Reconnect deadline is initialized lazily on the first WindowNotFound.
                // Only leaf (non-flow) steps are retried here; flow steps propagate
                // errors so their inner run_steps handles retries without re-running
                // already-completed inner steps.
                let mut reconnect_deadline: Option<Instant> = None;

                loop {
                    match self.run_step(step, vars, reconnect_ms).await {
                        Err(e)
                            if is_window_not_found(&e)
                                && reconnect_ms > 0
                                && !is_flow_step(step) =>
                        {
                            let deadline = reconnect_deadline.get_or_insert_with(|| {
                                warn!(
                                    step = i,
                                    reconnect_ms,
                                    "window not found — waiting for RDP/VNC reconnect"
                                );
                                Instant::now() + Duration::from_millis(reconnect_ms)
                            });
                            if Instant::now() >= *deadline {
                                error!(error = %e, step = i, "step failed (reconnect timeout)");
                                let screenshot = self.save_failure_screenshot(i).await;
                                if self.report_path.is_some() {
                                    self.report_records.lock().await.push(StepRecord {
                                        index: i,
                                        name: step.name().to_owned(),
                                        started_at: step_started,
                                        elapsed_ms: step_timer.elapsed().as_millis() as u64,
                                        outcome: StepOutcome::Failed(e.to_string()),
                                        screenshot_path: screenshot,
                                    });
                                }
                                return Err(e);
                            }
                            sleep(Duration::from_millis(1000)).await;
                            self.check_cancelled()?;
                        }
                        Err(e) => {
                            error!(error = %e, step = i, "step failed");
                            let screenshot = self.save_failure_screenshot(i).await;
                            if self.report_path.is_some() {
                                self.report_records.lock().await.push(StepRecord {
                                    index: i,
                                    name: step.name().to_owned(),
                                    started_at: step_started,
                                    elapsed_ms: step_timer.elapsed().as_millis() as u64,
                                    outcome: StepOutcome::Failed(e.to_string()),
                                    screenshot_path: screenshot,
                                });
                            }
                            return Err(e);
                        }
                        Ok(Flow::Done) => {
                            if self.dump_vars {
                                eprintln!("[VARS] step {i}: {}", vars.debug_dump());
                            }
                            if self.report_path.is_some() {
                                self.report_records.lock().await.push(StepRecord {
                                    index: i,
                                    name: step.name().to_owned(),
                                    started_at: step_started,
                                    elapsed_ms: step_timer.elapsed().as_millis() as u64,
                                    outcome: StepOutcome::Ok,
                                    screenshot_path: None,
                                });
                            }
                            break;
                        }
                        Ok(flow) => return Ok(flow),
                    }
                }
            }
            Ok(Flow::Done)
        }) // Box::pin
    }

    async fn run_step(
        &self,
        step: &ScenarioStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        match step {
            // existing
            ScenarioStep::WaitImage(s) => {
                self.wait_image(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::ClickImage(s) => {
                self.click_image(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::Type(s) => {
                self.type_text(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::Press(key) => {
                if !self.dry_run {
                    self.backend.press_key(key)?;
                } else {
                    info!(dry_run = true, key, "press skipped");
                }
                Ok(Flow::Done)
            }
            ScenarioStep::Library(s) => {
                self.call_library(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::Script(s) => {
                self.run_script(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::Foreach(s) => self.run_foreach(s, vars, reconnect_ms).await,
            ScenarioStep::SubScenario(s) => {
                self.run_sub_scenario(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::Set(s) => {
                vars.set(s.name.clone(), s.value.clone());
                Ok(Flow::Done)
            }
            ScenarioStep::WaitMs(ms) => {
                let mut remaining = *ms;
                while remaining > 0 {
                    self.check_cancelled()?;
                    let chunk = remaining.min(100);
                    sleep(Duration::from_millis(chunk)).await;
                    remaining = remaining.saturating_sub(chunk);
                }
                Ok(Flow::Done)
            }

            // flow control
            ScenarioStep::Group(s) => self.run_group(s, vars, reconnect_ms).await,
            ScenarioStep::If(s) => self.run_if(s, vars, reconnect_ms).await,
            ScenarioStep::Switch(s) => self.run_switch(s, vars, reconnect_ms).await,
            ScenarioStep::Repeat(s) => self.run_repeat(s, vars, reconnect_ms).await,
            ScenarioStep::While(s) => self.run_while(s, vars, reconnect_ms).await,
            ScenarioStep::DoWhile(s) => self.run_do_while(s, vars, reconnect_ms).await,
            ScenarioStep::TryCatch(s) => self.run_try_catch(s, vars, reconnect_ms).await,
            ScenarioStep::Break => Ok(Flow::Break),
            ScenarioStep::Continue => Ok(Flow::Continue),
            ScenarioStep::CallScenario(s) => {
                self.run_call_scenario(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::Exit => Ok(Flow::Exit),

            // additional action nodes
            ScenarioStep::FindImage(s) => {
                self.find_image(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::Shell(s) => {
                self.run_shell(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::ClipboardSet(s) => {
                self.clipboard_set(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ClipboardGet(s) => {
                self.clipboard_get(s, vars)?;
                Ok(Flow::Done)
            }

            // variable nodes
            ScenarioStep::CopyVar(s) => {
                self.copy_var(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::GetDatetime(s) => {
                self.get_datetime(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::GetUsername(s) => {
                self.get_username(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::Calc(s) => {
                self.run_calc(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::Increment(s) => {
                self.run_increment(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ToFullwidth(s) => {
                self.run_to_fullwidth(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ToHalfwidth(s) => {
                self.run_to_halfwidth(s, vars)?;
                Ok(Flow::Done)
            }

            // user interaction
            ScenarioStep::DialogWait(s) => {
                self.dialog_wait(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::DialogInput(s) => {
                self.dialog_input(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::DialogSelect(s) => {
                self.dialog_select(s, vars).await?;
                Ok(Flow::Done)
            }

            // window / region nodes
            ScenarioStep::WaitWindow(s) => {
                self.wait_window(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::MatchRect(s) => {
                self.match_rect(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::WindowControl(s) => {
                self.window_control(s, vars).await?;
                Ok(Flow::Done)
            }

            // OCR
            ScenarioStep::OcrMatch(s) => {
                self.ocr_match(s, vars).await?;
                Ok(Flow::Done)
            }

            // ML detection
            ScenarioStep::MlDetect(s) => {
                self.ml_detect(s, vars).await?;
                Ok(Flow::Done)
            }

            // variable persistence
            ScenarioStep::ImportVars(s) => {
                self.import_vars(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::SaveVars(s) => {
                self.save_vars(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::LoadVars(s) => {
                self.load_vars(s, vars)?;
                Ok(Flow::Done)
            }

            // file operations
            ScenarioStep::FileExists(s) => {
                self.file_exists(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileCopy(s) => {
                self.file_copy(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileMove(s) => {
                self.file_move(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileDelete(s) => {
                self.file_delete(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileRename(s) => {
                self.file_rename(s, vars)?;
                Ok(Flow::Done)
            }

            // logging
            ScenarioStep::LogWrite(s) => {
                self.log_write(s, vars)?;
                Ok(Flow::Done)
            }

            // date operations
            ScenarioStep::DateFormat(s) => {
                self.date_format(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::DateAdd(s) => {
                self.date_add(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::DateDiff(s) => {
                self.date_diff(s, vars)?;
                Ok(Flow::Done)
            }

            // string operations
            ScenarioStep::StringReplace(s) => {
                self.string_replace(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::StringTrim(s) => {
                self.string_trim(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::StringUpper(s) => {
                let v = vars.expand(&s.value).to_uppercase();
                vars.set(&s.save_as, serde_json::Value::String(v));
                Ok(Flow::Done)
            }
            ScenarioStep::StringLower(s) => {
                let v = vars.expand(&s.value).to_lowercase();
                vars.set(&s.save_as, serde_json::Value::String(v));
                Ok(Flow::Done)
            }
            ScenarioStep::StringSubstring(s) => {
                self.string_substring(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::StringLength(s) => {
                let len = vars.expand(&s.value).chars().count() as i64;
                vars.set(&s.save_as, serde_json::Value::Number(len.into()));
                Ok(Flow::Done)
            }
            ScenarioStep::StringSplit(s) => {
                self.string_split(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::StringJoin(s) => {
                self.string_join(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::StringRegex(s) => {
                self.string_regex(s, vars)?;
                Ok(Flow::Done)
            }

            // json helpers
            ScenarioStep::JsonParse(s) => {
                self.json_parse(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::JsonStringify(s) => {
                self.json_stringify(s, vars)?;
                Ok(Flow::Done)
            }

            // path helpers
            ScenarioStep::PathJoin(s) => {
                self.path_join(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::PathBasename(s) => {
                self.path_basename(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::PathDirname(s) => {
                self.path_dirname(s, vars)?;
                Ok(Flow::Done)
            }

            // env / misc
            ScenarioStep::EnvGet(s) => {
                self.env_get(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileList(s) => {
                self.file_list(s, vars)?;
                Ok(Flow::Done)
            }

            // mouse coordinate nodes
            ScenarioStep::MouseMove(s) => {
                self.mouse_move(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::MouseClickXy(s) => {
                self.mouse_click_xy(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::MouseDrag(s) => {
                self.mouse_drag(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::MouseScroll(s) => {
                self.mouse_scroll(s, vars)?;
                Ok(Flow::Done)
            }
            // --- HTTP client nodes ---
            #[cfg(feature = "http")]
            ScenarioStep::HttpGet(s) => {
                self.http_get(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "http")]
            ScenarioStep::HttpPost(s) => {
                self.http_post(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "http")]
            ScenarioStep::HttpPut(s) => {
                self.http_put(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "http")]
            ScenarioStep::HttpDelete(s) => {
                self.http_delete(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "http")]
            ScenarioStep::HttpPatch(s) => {
                self.http_patch(s, vars).await?;
                Ok(Flow::Done)
            }
            // mail receive
            ScenarioStep::MailReceive(s) => {
                self.mail_receive(s, vars).await?;
                Ok(Flow::Done)
            }
            // Excel cell nodes
            ScenarioStep::ExcelReadCell(s) => {
                self.excel_read_cell(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ExcelReadRange(s) => {
                self.excel_read_range(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ExcelWriteCell(s) => {
                self.excel_write_cell(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ExcelAddSheet(s) => {
                self.excel_add_sheet(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ExcelDeleteSheet(s) => {
                self.excel_delete_sheet(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ExcelRenameSheet(s) => {
                self.excel_rename_sheet(s, vars)?;
                Ok(Flow::Done)
            }

            // text file read/write
            ScenarioStep::FileRead(s) => {
                self.file_read(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileWrite(s) => {
                self.file_write(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::FileAppend(s) => {
                self.file_append(s, vars)?;
                Ok(Flow::Done)
            }

            // process operations
            ScenarioStep::ProcessStart(s) => {
                self.process_start(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::ProcessKill(s) => {
                self.process_kill(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::ProcessExists(s) => {
                self.process_exists(s, vars)?;
                Ok(Flow::Done)
            }

            // key combination
            ScenarioStep::KeyCombo(s) => {
                self.key_combo(s)?;
                Ok(Flow::Done)
            }

            // CSV read/write
            ScenarioStep::CsvRead(s) => {
                self.csv_read(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::CsvWrite(s) => {
                self.csv_write(s, vars)?;
                Ok(Flow::Done)
            }

            // screenshot / observation
            ScenarioStep::ScreenshotSave(s) => {
                self.screenshot_save(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::WaitNoImage(s) => {
                self.wait_no_image(s, vars).await?;
                Ok(Flow::Done)
            }

            // system integration
            ScenarioStep::UrlOpen(s) => {
                self.url_open(s, vars)?;
                Ok(Flow::Done)
            }
            ScenarioStep::Notify(s) => {
                self.notify_step(s, vars)?;
                Ok(Flow::Done)
            }

            // --- UI Automation ---
            ScenarioStep::UiaGet(s) => {
                self.uia_get(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::UiaSet(s) => {
                self.uia_set(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::UiaClick(s) => {
                self.uia_click(s, vars).await?;
                Ok(Flow::Done)
            }
            ScenarioStep::UiaFind(s) => {
                self.uia_find(s, vars).await?;
                Ok(Flow::Done)
            }

            // --- Web automation ---
            #[cfg(feature = "web")]
            ScenarioStep::WebOpen(s) => {
                self.web_open(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "web")]
            ScenarioStep::WebClick(s) => {
                self.web_click(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "web")]
            ScenarioStep::WebType(s) => {
                self.web_type(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "web")]
            ScenarioStep::WebGet(s) => {
                self.web_get(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "web")]
            ScenarioStep::WebWait(s) => {
                self.web_wait(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "web")]
            ScenarioStep::WebScreenshot(s) => {
                self.web_screenshot(s, vars).await?;
                Ok(Flow::Done)
            }
            #[cfg(feature = "web")]
            ScenarioStep::WebClose => {
                self.web_close().await?;
                Ok(Flow::Done)
            }

            // Non-web build: web steps are unsupported
            #[cfg(not(feature = "web"))]
            ScenarioStep::WebOpen(_)
            | ScenarioStep::WebClick(_)
            | ScenarioStep::WebType(_)
            | ScenarioStep::WebGet(_)
            | ScenarioStep::WebWait(_)
            | ScenarioStep::WebScreenshot(_)
            | ScenarioStep::WebClose => Err(EngineError::Other(
                "web_* steps require the 'web' feature; rebuild with: cargo build --features web"
                    .to_owned(),
            )),
        }
    }

    // ── Image helpers ───────────────────────────────────────────────────────

    fn capture_target(window: &Option<String>) -> rpa_template::Target {
        match window {
            Some(title) => rpa_template::Target::Window {
                title_contains: title.clone(),
            },
            None => rpa_template::Target::Screen,
        }
    }

    async fn wait_image(&self, step: &WaitImageStep, vars: &Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.template));
        let template = Arc::new(self.load_image(&path)?);
        let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);
        let target = Self::capture_target(&step.window);

        loop {
            let backend = Arc::clone(&self.backend);
            let matcher = self.matcher.clone();
            let tmpl = Arc::clone(&template);
            let tgt = target.clone();
            let masks = step.masks.clone();

            let result: Result<rpa_vision::MatchResult> = tokio::task::spawn_blocking(move || {
                let (img, origin) = backend.capture_with_origin(&tgt)?;
                Ok(matcher.find_with_masks(&img, &tmpl, origin, &masks)?)
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

            match result {
                Ok(m) => {
                    info!(score = m.score, "template found");
                    return Ok(());
                }
                Err(EngineError::Vision(_)) if Instant::now() < deadline => {
                    self.check_cancelled()?;
                    sleep(Duration::from_millis(step.retry_interval_ms)).await;
                }
                Err(EngineError::Vision(_)) => {
                    return Err(EngineError::Timeout(step.template.clone()))
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn click_image(&self, step: &ClickImageStep, vars: &Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.template));
        let template = Arc::new(self.load_image(&path)?);
        let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);
        let target = Self::capture_target(&step.window);

        loop {
            let backend = Arc::clone(&self.backend);
            let matcher = self.matcher.clone();
            let tmpl = Arc::clone(&template);
            let tgt = target.clone();
            let masks = step.masks.clone();

            let result: Result<rpa_vision::MatchResult> = tokio::task::spawn_blocking(move || {
                let (img, origin) = backend.capture_with_origin(&tgt)?;
                Ok(matcher.find_with_masks(&img, &tmpl, origin, &masks)?)
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

            match result {
                Ok(m) => {
                    let point = rpa_template::ScreenPoint {
                        x: m.center.x + step.offset_x,
                        y: m.center.y + step.offset_y,
                    };
                    if !self.dry_run {
                        match step.action {
                            ClickAction::Left => self.backend.click(point)?,
                            ClickAction::Right => self.backend.right_click(point)?,
                            ClickAction::Double => self.backend.double_click(point)?,
                        }
                    } else {
                        info!(
                            dry_run = true,
                            x = point.x,
                            y = point.y,
                            "click_image skipped"
                        );
                    }
                    if let Some(ms) = step.post_click_ms {
                        sleep(Duration::from_millis(ms)).await;
                    }
                    return Ok(());
                }
                Err(EngineError::Vision(_)) if Instant::now() < deadline => {
                    self.check_cancelled()?;
                    sleep(Duration::from_millis(200)).await;
                }
                Err(EngineError::Vision(_)) => {
                    return Err(EngineError::Timeout(step.template.clone()))
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Find an image without clicking. Stores position info in `save_as` if specified.
    async fn find_image(&self, step: &FindImageStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.template));
        let template = Arc::new(self.load_image(&path)?);
        let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);
        let target = Self::capture_target(&step.window);

        loop {
            let backend = Arc::clone(&self.backend);
            let matcher = self.matcher.clone();
            let tmpl = Arc::clone(&template);
            let tgt = target.clone();

            let result: Result<rpa_vision::MatchResult> = tokio::task::spawn_blocking(move || {
                let (img, origin) = backend.capture_with_origin(&tgt)?;
                Ok(matcher.find(&img, &tmpl, origin)?)
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

            match result {
                Ok(m) => {
                    info!(
                        score = m.score,
                        x = m.center.x,
                        y = m.center.y,
                        "template found"
                    );
                    if let Some(save_as) = &step.save_as {
                        vars.set(
                            save_as.clone(),
                            serde_json::json!({
                                "found": true,
                                "x": m.center.x,
                                "y": m.center.y,
                                "score": m.score,
                            }),
                        );
                    }
                    return Ok(());
                }
                Err(EngineError::Vision(_)) if Instant::now() < deadline => {
                    self.check_cancelled()?;
                    sleep(Duration::from_millis(step.retry_interval_ms)).await;
                }
                Err(EngineError::Vision(_)) => {
                    if let Some(save_as) = &step.save_as {
                        vars.set(save_as.clone(), serde_json::json!({ "found": false }));
                        return Ok(());
                    }
                    return Err(EngineError::Timeout(step.template.clone()));
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ── Text helpers ────────────────────────────────────────────────────────

    fn type_text(&self, step: &TypeStep, vars: &Variables) -> Result<()> {
        let text = match step {
            TypeStep::Plain(s) => vars.expand(s),
            TypeStep::SecretEnv { secret_env } => std::env::var(secret_env)
                .map_err(|_| EngineError::MissingSecret(secret_env.clone()))?,
        };
        if !self.dry_run {
            self.backend.type_text(&text)?;
        } else {
            info!(dry_run = true, len = text.len(), "type_text skipped");
        }
        Ok(())
    }

    // ── Library call ────────────────────────────────────────────────────────

    async fn call_library(&self, step: &LibraryStep, vars: &mut Variables) -> Result<()> {
        let inputs = step
            .inputs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let name = step.name.clone();
        let outputs = tokio::task::spawn_blocking(move || rpa_stdlib::dispatch(&name, inputs))
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?
            .map_err(|e| EngineError::SubScenario(format!("library '{}': {e}", step.name)))?;

        // Merge each named output into vars.
        for (k, v) in &outputs {
            vars.set(k.clone(), v.clone());
        }
        // Also store the whole output map under save_as if provided.
        if let Some(save_as) = &step.save_as {
            vars.set(
                save_as.clone(),
                serde_json::Value::Object(outputs.into_iter().collect::<serde_json::Map<_, _>>()),
            );
        }

        Ok(())
    }

    // ── Inline script ───────────────────────────────────────────────────────

    fn run_script(&self, step: &ScriptStep, vars: &mut Variables) -> Result<()> {
        let map = vars.as_map().clone();
        let (result, exports) = self.script_engine.run(&step.script, &map)?;

        if let Some(save_as) = &step.save_as {
            vars.set(save_as.clone(), dynamic_to_json(&result));
        }
        for (k, v) in exports {
            vars.set(k, dynamic_to_json(&v));
        }
        Ok(())
    }

    // ── Condition helper ────────────────────────────────────────────────────

    fn eval_cond(&self, cond: &str, vars: &Variables) -> Result<bool> {
        let map = vars.as_map().clone();
        self.script_engine
            .eval_bool(cond, &map)
            .map_err(EngineError::Script)
    }

    // ── Flow control ────────────────────────────────────────────────────────

    async fn run_group(
        &self,
        step: &GroupStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        let name = step.name.as_deref().unwrap_or("<group>");
        let span = tracing::info_span!("group", name = name);
        let _g = span.enter();
        self.run_steps(&step.steps, vars, reconnect_ms).await
    }

    async fn run_if(&self, step: &IfStep, vars: &mut Variables, reconnect_ms: u64) -> Result<Flow> {
        if self.eval_cond(&step.cond, vars)? {
            self.run_steps(&step.then, vars, reconnect_ms).await
        } else {
            self.run_steps(&step.else_steps, vars, reconnect_ms).await
        }
    }

    async fn run_switch(
        &self,
        step: &SwitchStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        let val = vars
            .get(&step.on)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        for case in &step.cases {
            if val == case.when {
                return self.run_steps(&case.steps, vars, reconnect_ms).await;
            }
        }
        self.run_steps(&step.default, vars, reconnect_ms).await
    }

    async fn run_repeat(
        &self,
        step: &RepeatStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        for i in 0..step.count {
            vars.set("_index".to_owned(), serde_json::Value::Number(i.into()));
            match self.run_steps(&step.steps, vars, reconnect_ms).await? {
                Flow::Done | Flow::Continue => {}
                Flow::Break => break,
                Flow::Exit => return Ok(Flow::Exit),
            }
        }
        Ok(Flow::Done)
    }

    async fn run_while(
        &self,
        step: &WhileStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        loop {
            if !self.eval_cond(&step.cond, vars)? {
                break;
            }
            match self.run_steps(&step.steps, vars, reconnect_ms).await? {
                Flow::Done | Flow::Continue => {}
                Flow::Break => break,
                Flow::Exit => return Ok(Flow::Exit),
            }
        }
        Ok(Flow::Done)
    }

    async fn run_do_while(
        &self,
        step: &DoWhileStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        loop {
            match self.run_steps(&step.steps, vars, reconnect_ms).await? {
                Flow::Done | Flow::Continue => {}
                Flow::Break => break,
                Flow::Exit => return Ok(Flow::Exit),
            }
            if !self.eval_cond(&step.cond, vars)? {
                break;
            }
        }
        Ok(Flow::Done)
    }

    async fn run_try_catch(
        &self,
        step: &TryCatchStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        let result = match self.run_steps(&step.try_steps, vars, reconnect_ms).await {
            Ok(f) => Ok(f),
            Err(e) => {
                vars.set(
                    "_error".to_owned(),
                    serde_json::Value::String(e.to_string()),
                );
                self.run_steps(&step.catch, vars, reconnect_ms).await
            }
        };
        if !step.finally.is_empty() {
            self.run_steps(&step.finally, vars, reconnect_ms).await?;
        }
        result
    }

    async fn run_foreach(
        &self,
        step: &ForeachStep,
        vars: &mut Variables,
        reconnect_ms: u64,
    ) -> Result<Flow> {
        let list = match vars.get(&step.var) {
            Some(serde_json::Value::Array(arr)) => arr.clone(),
            _ => {
                warn!(
                    var = step.var,
                    "foreach: variable not found or not an array"
                );
                return Ok(Flow::Done);
            }
        };

        for item in list {
            vars.set("item".to_owned(), item);
            match self.run_steps(&step.steps, vars, reconnect_ms).await? {
                Flow::Done | Flow::Continue => {}
                Flow::Break => break,
                Flow::Exit => return Ok(Flow::Exit),
            }
        }
        Ok(Flow::Done)
    }

    async fn run_sub_scenario(&self, step: &SubScenarioStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(&step.path);
        let sub =
            Scenario::from_file(&path).map_err(|e| EngineError::SubScenario(e.to_string()))?;

        let mut sub_vars = Variables::new();
        for (k, v) in &step.inputs {
            sub_vars.set(k.clone(), v.clone());
        }
        for (k, v) in vars.as_map() {
            if !sub_vars.as_map().contains_key(k) {
                sub_vars.set(k.clone(), v.clone());
            }
        }

        let sub_engine = ScenarioEngine::new(
            Arc::clone(&self.backend),
            path.parent().unwrap_or(&self.base_dir).to_path_buf(),
        )
        .with_silent(self.silent)
        .with_reconnect_timeout(self.reconnect_timeout_ms)
        .with_cancel(Arc::clone(&self.cancelled));
        sub_engine.run(&sub, &mut sub_vars).await?;

        if let Some(save_as) = &step.save_as {
            let map: serde_json::Map<String, serde_json::Value> = sub_vars
                .as_map()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            vars.set(save_as.clone(), serde_json::Value::Object(map));
        }
        Ok(())
    }

    async fn run_call_scenario(&self, step: &CallScenarioStep, vars: &mut Variables) -> Result<()> {
        let raw_path = vars.expand(&step.path);
        let path = self.base_dir.join(&raw_path);
        let sub =
            Scenario::from_file(&path).map_err(|e| EngineError::SubScenario(e.to_string()))?;

        let mut sub_vars = Variables::new();
        for (k, v) in &step.inputs {
            sub_vars.set(k.clone(), v.clone());
        }
        for (k, v) in vars.as_map() {
            if !sub_vars.as_map().contains_key(k) {
                sub_vars.set(k.clone(), v.clone());
            }
        }

        let sub_engine = ScenarioEngine::new(
            Arc::clone(&self.backend),
            path.parent().unwrap_or(&self.base_dir).to_path_buf(),
        )
        .with_silent(self.silent)
        .with_reconnect_timeout(self.reconnect_timeout_ms)
        .with_cancel(Arc::clone(&self.cancelled));
        sub_engine.run(&sub, &mut sub_vars).await?;

        if let Some(save_as) = &step.save_as {
            let map: serde_json::Map<String, serde_json::Value> = sub_vars
                .as_map()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            vars.set(save_as.clone(), serde_json::Value::Object(map));
        }
        Ok(())
    }

    // ── Shell ───────────────────────────────────────────────────────────────

    async fn run_shell(&self, step: &ShellStep, vars: &mut Variables) -> Result<()> {
        let cmd = vars.expand(&step.cmd);
        let args: Vec<String> = step.args.iter().map(|a| vars.expand(a)).collect();

        let output = tokio::time::timeout(
            Duration::from_millis(step.timeout_ms),
            tokio::process::Command::new(&cmd).args(&args).output(),
        )
        .await
        .map_err(|_| EngineError::ShellTimeout(cmd.clone()))?
        .map_err(EngineError::Io)?;

        let code = output.status.code().unwrap_or(-1);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(cmd = %cmd, code, stderr = %stderr, "shell command exited with error");
        } else {
            info!(cmd = %cmd, code, "shell command succeeded");
        }

        if let Some(save_as) = &step.save_as {
            let stdout = String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_owned();
            vars.set(save_as.clone(), serde_json::Value::String(stdout));
        }
        Ok(())
    }

    // ── Clipboard ───────────────────────────────────────────────────────────

    fn clipboard_set(&self, step: &ClipboardSetStep, vars: &Variables) -> Result<()> {
        let text = vars.expand(&step.value);
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(&text))
            .map_err(|e| EngineError::Other(format!("clipboard_set: {e}")))?;
        info!(value = %text, "clipboard_set");
        Ok(())
    }

    fn clipboard_get(&self, step: &ClipboardGetStep, vars: &mut Variables) -> Result<()> {
        let text = arboard::Clipboard::new()
            .and_then(|mut cb| cb.get_text())
            .map_err(|e| EngineError::Other(format!("clipboard_get: {e}")))?;
        info!(save_as = %step.save_as, len = text.len(), "clipboard_get");
        vars.set(step.save_as.clone(), serde_json::Value::String(text));
        Ok(())
    }

    // ── Variable nodes ──────────────────────────────────────────────────────

    fn copy_var(&self, step: &CopyVarStep, vars: &mut Variables) -> Result<()> {
        let val = vars
            .get(&step.from)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        vars.set(step.to.clone(), val);
        Ok(())
    }

    fn get_datetime(&self, step: &GetDatetimeStep, vars: &mut Variables) -> Result<()> {
        let formatted = Local::now().format(&step.format).to_string();
        vars.set(step.save_as.clone(), serde_json::Value::String(formatted));
        Ok(())
    }

    fn get_username(&self, step: &GetUsernameStep, vars: &mut Variables) -> Result<()> {
        let name = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_owned());
        vars.set(step.save_as.clone(), serde_json::Value::String(name));
        Ok(())
    }

    fn run_calc(&self, step: &CalcStep, vars: &mut Variables) -> Result<()> {
        let map = vars.as_map().clone();
        let (result, _) = self.script_engine.run(&step.expr, &map)?;
        vars.set(step.save_as.clone(), dynamic_to_json(&result));
        Ok(())
    }

    fn run_increment(&self, step: &IncrementStep, vars: &mut Variables) -> Result<()> {
        let current = vars.get(&step.name).and_then(|v| v.as_i64()).unwrap_or(0);
        vars.set(
            step.name.clone(),
            serde_json::Value::Number((current + step.by).into()),
        );
        Ok(())
    }

    fn run_to_fullwidth(&self, step: &WidthStep, vars: &mut Variables) -> Result<()> {
        let input = vars.expand(&step.value);
        let output = to_fullwidth(&input);
        vars.set(step.save_as.clone(), serde_json::Value::String(output));
        Ok(())
    }

    fn run_to_halfwidth(&self, step: &WidthStep, vars: &mut Variables) -> Result<()> {
        let input = vars.expand(&step.value);
        let output = to_halfwidth(&input);
        vars.set(step.save_as.clone(), serde_json::Value::String(output));
        Ok(())
    }

    // ── Dialog nodes ────────────────────────────────────────────────────────

    async fn dialog_wait(&self, step: &DialogWaitStep, vars: &Variables) -> Result<()> {
        let message = vars.expand(&step.message);
        let title = step.title.as_deref().unwrap_or("Waiting");
        if self.silent {
            info!(title, message, "dialog_wait skipped (silent)");
            return Ok(());
        }
        eprintln!("[{}] {}", title, message);
        eprintln!("Press Enter to continue...");
        let mut buf = String::new();
        std::io::BufReader::new(std::io::stdin()).read_line(&mut buf)?;
        Ok(())
    }

    async fn dialog_input(&self, step: &DialogInputStep, vars: &mut Variables) -> Result<()> {
        let message = vars.expand(&step.message);
        let title = step.title.as_deref().unwrap_or("Input");
        let default = step.default.as_deref().unwrap_or("");

        if self.silent {
            info!(
                title,
                message, default, "dialog_input skipped (silent), using default"
            );
            vars.set(
                step.save_as.clone(),
                serde_json::Value::String(default.to_owned()),
            );
            return Ok(());
        }

        eprintln!("[{}] {} (default: {})", title, message, default);
        let mut buf = String::new();
        std::io::BufReader::new(std::io::stdin()).read_line(&mut buf)?;
        let input = buf.trim_end_matches(['\n', '\r']).to_owned();
        let value = if input.is_empty() {
            default.to_owned()
        } else {
            input
        };
        vars.set(step.save_as.clone(), serde_json::Value::String(value));
        Ok(())
    }

    async fn dialog_select(&self, step: &DialogSelectStep, vars: &mut Variables) -> Result<()> {
        let message = vars.expand(&step.message);
        let title = step.title.as_deref().unwrap_or("Select");

        if step.options.is_empty() {
            warn!("dialog_select: no options provided");
            vars.set(step.save_as.clone(), serde_json::Value::Null);
            return Ok(());
        }

        if self.silent {
            let first = step.options[0].clone();
            info!(title, message, choice = %first, "dialog_select skipped (silent), using first option");
            vars.set(step.save_as.clone(), serde_json::Value::String(first));
            return Ok(());
        }

        eprintln!("[{}] {}", title, message);
        for (i, opt) in step.options.iter().enumerate() {
            eprintln!("  {}: {}", i + 1, opt);
        }
        eprintln!("Enter number (1-{}):", step.options.len());

        let mut buf = String::new();
        std::io::BufReader::new(std::io::stdin()).read_line(&mut buf)?;
        let idx: usize = buf.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
        let idx = idx.min(step.options.len() - 1);
        vars.set(
            step.save_as.clone(),
            serde_json::Value::String(step.options[idx].clone()),
        );
        Ok(())
    }

    // ── Window / region nodes ───────────────────────────────────────────────

    async fn wait_window(&self, step: &WaitWindowStep, vars: &mut Variables) -> Result<()> {
        let title = vars.expand(&step.title_contains);
        let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);

        loop {
            let title_clone = title.clone();
            let found = tokio::task::spawn_blocking(move || {
                xcap::Window::all()
                    .unwrap_or_default()
                    .iter()
                    .any(|w| w.title().map(|t| t.contains(&title_clone)).unwrap_or(false))
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

            let matched = match step.state {
                WindowState::Exists => found,
                WindowState::Closed => !found,
            };

            if matched {
                info!(title_contains = %title, state = ?step.state, "wait_window: condition met");
                if let Some(save_as) = &step.save_as {
                    vars.set(save_as.clone(), serde_json::Value::Bool(true));
                }
                return Ok(());
            }

            if Instant::now() >= deadline {
                if let Some(save_as) = &step.save_as {
                    vars.set(save_as.clone(), serde_json::Value::Bool(false));
                    return Ok(());
                }
                return Err(EngineError::Timeout(format!("wait_window: {title}")));
            }

            self.check_cancelled()?;
            sleep(Duration::from_millis(step.retry_interval_ms)).await;
        }
    }

    async fn match_rect(&self, step: &MatchRectStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.template));
        let template = Arc::new(self.load_image(&path)?);
        let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);
        let target = rpa_template::Target::Region(step.rect);

        loop {
            let backend = Arc::clone(&self.backend);
            let matcher = self.matcher.clone();
            let tmpl = Arc::clone(&template);
            let tgt = target.clone();

            let result: Result<rpa_vision::MatchResult> = tokio::task::spawn_blocking(move || {
                let (img, origin) = backend.capture_with_origin(&tgt)?;
                Ok(matcher.find(&img, &tmpl, origin)?)
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

            match result {
                Ok(m) => {
                    info!(
                        score = m.score,
                        x = m.center.x,
                        y = m.center.y,
                        "match_rect: found"
                    );
                    if let Some(save_as) = &step.save_as {
                        vars.set(
                            save_as.clone(),
                            serde_json::json!({
                                "found": true,
                                "x": m.center.x,
                                "y": m.center.y,
                                "score": m.score,
                            }),
                        );
                    }
                    return Ok(());
                }
                Err(EngineError::Vision(_)) if Instant::now() < deadline => {
                    self.check_cancelled()?;
                    sleep(Duration::from_millis(step.retry_interval_ms)).await;
                }
                Err(EngineError::Vision(_)) => {
                    if let Some(save_as) = &step.save_as {
                        vars.set(save_as.clone(), serde_json::json!({ "found": false }));
                        return Ok(());
                    }
                    return Err(EngineError::Timeout(step.template.clone()));
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ── Window control node ─────────────────────────────────────────────────

    async fn window_control(&self, step: &WindowControlStep, vars: &Variables) -> Result<()> {
        let title = vars.expand(&step.title_contains);
        let action = match step.action {
            WindowControlAction::Focus => "focus",
            WindowControlAction::Maximize => "maximize",
            WindowControlAction::Minimize => "minimize",
            WindowControlAction::Close => "close",
        };
        let action_str = action.to_owned();
        let backend = Arc::clone(&self.backend);
        tokio::task::spawn_blocking(move || backend.control_window(&title, &action_str))
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))??;
        info!(title = step.title_contains, action, "window_control");
        Ok(())
    }

    // ── OCR node ────────────────────────────────────────────────────────────

    async fn ocr_match(&self, step: &OcrMatchStep, vars: &mut Variables) -> Result<()> {
        #[cfg(not(feature = "ocr"))]
        {
            let _ = step;
            let _ = vars;
            Err(EngineError::Other(
                "ocr_match requires the 'ocr' feature; rebuild with: cargo build --features ocr"
                    .to_owned(),
            ))
        }

        #[cfg(feature = "ocr")]
        {
            let target = Self::capture_target(&step.window);
            let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);
            let lang = step.lang.clone();
            let contains = step.contains.clone();
            let region = step.region;

            loop {
                let backend = Arc::clone(&self.backend);
                let tgt = target.clone();
                let lang2 = lang.clone();

                let text_result: Result<String> = tokio::task::spawn_blocking(move || {
                    let (img, _origin) = backend.capture_with_origin(&tgt)?;

                    let ocr_img = if let Some(r) = region {
                        let x0 = r.x.max(0) as u32;
                        let y0 = r.y.max(0) as u32;
                        let x1 = (r.x + r.width as i32).min(img.width() as i32) as u32;
                        let y1 = (r.y + r.height as i32).min(img.height() as i32) as u32;
                        image::imageops::crop_imm(&img, x0, y0, x1 - x0, y1 - y0).to_image()
                    } else {
                        img
                    };

                    rpa_vision::ocr::OcrEngine::new(lang2)
                        .extract_text(&ocr_img)
                        .map_err(|e| EngineError::Other(format!("ocr_match: {e}")))
                })
                .await
                .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

                match text_result {
                    Ok(text) => {
                        let found = contains
                            .as_ref()
                            .map_or(true, |pat| text.contains(pat.as_str()));
                        info!(found, text_len = text.len(), "ocr_match");
                        if found {
                            if let Some(save_as) = &step.save_as {
                                vars.set(
                                    save_as.clone(),
                                    serde_json::json!({ "found": true, "text": text }),
                                );
                            }
                            return Ok(());
                        }
                        if Instant::now() >= deadline {
                            if let Some(save_as) = &step.save_as {
                                vars.set(
                                    save_as.clone(),
                                    serde_json::json!({ "found": false, "text": text }),
                                );
                                return Ok(());
                            }
                            return Err(EngineError::Timeout(format!("ocr_match: {:?}", contains)));
                        }
                        self.check_cancelled()?;
                        sleep(Duration::from_millis(step.retry_interval_ms)).await;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }

    // ── ML detection node ───────────────────────────────────────────────────

    async fn ml_detect(&self, step: &MlDetectStep, vars: &mut Variables) -> Result<()> {
        #[cfg(not(feature = "ml"))]
        {
            let _ = step;
            let _ = vars;
            Err(EngineError::Other(
                "ml_detect requires the 'ml' feature; rebuild with: cargo build --features ml"
                    .to_owned(),
            ))
        }

        #[cfg(feature = "ml")]
        {
            use rpa_vision::ml::MlDetector;
            let target = Self::capture_target(&step.window);
            let model_path = self.base_dir.join(&step.model);
            let threshold = step.threshold;
            let backend = Arc::clone(&self.backend);

            let detections = tokio::task::spawn_blocking(move || {
                let (img, _origin) = backend.capture_with_origin(&target)?;
                let detector = MlDetector::new(model_path.to_string_lossy().as_ref(), threshold);
                detector
                    .detect(&img)
                    .map_err(|e| EngineError::Other(format!("ml_detect: {e}")))
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))??;

            info!(count = detections.len(), "ml_detect");
            if let Some(save_as) = &step.save_as {
                let json: Vec<serde_json::Value> = detections
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "label": d.label,
                            "score": d.score,
                            "x": d.bbox.x,
                            "y": d.bbox.y,
                            "width": d.bbox.width,
                            "height": d.bbox.height,
                        })
                    })
                    .collect();
                vars.set(save_as.clone(), serde_json::Value::Array(json));
            }
            Ok(())
        }
    }

    // ── Variable persistence ────────────────────────────────────────────────

    fn import_vars(&self, step: &ImportVarsStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.file));
        let sheet = step.sheet.as_deref();
        let rows = crate::data_source::load(&path, sheet)?;

        let row = rows.get(step.row).ok_or_else(|| {
            EngineError::Other(format!(
                "import_vars: file '{}' has no data row at index {}",
                path.display(),
                step.row
            ))
        })?;

        for (k, v) in row {
            let var_name = if step.prefix.is_empty() {
                k.clone()
            } else {
                format!("{}{k}", step.prefix)
            };
            vars.set(var_name, v.clone());
        }

        info!(file = %path.display(), count = row.len(), "import_vars");
        Ok(())
    }

    fn save_vars(&self, step: &SaveVarsStep, vars: &Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.file));

        let map: serde_json::Map<String, serde_json::Value> = if step.vars.is_empty() {
            vars.as_map()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            step.vars
                .iter()
                .filter_map(|name| vars.get(name).map(|v| (name.clone(), v.clone())))
                .collect()
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
        let json = serde_json::to_string_pretty(&map)
            .map_err(|e| EngineError::Other(format!("save_vars serialize: {e}")))?;
        std::fs::write(&path, json).map_err(EngineError::Io)?;

        info!(file = %path.display(), count = map.len(), "save_vars");
        Ok(())
    }

    fn load_vars(&self, step: &LoadVarsStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.file));
        let data = std::fs::read_to_string(&path).map_err(EngineError::Io)?;
        let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&data)
            .map_err(|e| EngineError::Other(format!("load_vars parse: {e}")))?;

        let count = map.len();
        for (k, v) in map {
            let var_name = if step.prefix.is_empty() {
                k
            } else {
                format!("{}{k}", step.prefix)
            };
            vars.set(var_name, v);
        }

        info!(file = %path.display(), count, "load_vars");
        Ok(())
    }

    // ── File operation methods ──────────────────────────────────────────────

    fn file_exists(&self, step: &FileExistsStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));
        let exists = path.exists();
        vars.set(&step.save_as, serde_json::Value::Bool(exists));
        info!(path = %path.display(), exists, "file_exists");
        Ok(())
    }

    fn file_copy(&self, step: &FileCopyStep, vars: &mut Variables) -> Result<()> {
        let src = self.base_dir.join(vars.expand(&step.src));
        let dst = self.base_dir.join(vars.expand(&step.dst));
        if !step.overwrite && dst.exists() {
            return Err(EngineError::Other(format!(
                "file_copy: destination already exists: {}",
                dst.display()
            )));
        }
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
        std::fs::copy(&src, &dst).map_err(EngineError::Io)?;
        info!(src = %src.display(), dst = %dst.display(), "file_copy");
        Ok(())
    }

    fn file_move(&self, step: &FileMoveStep, vars: &mut Variables) -> Result<()> {
        let src = self.base_dir.join(vars.expand(&step.src));
        let dst = self.base_dir.join(vars.expand(&step.dst));
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
        std::fs::rename(&src, &dst).map_err(EngineError::Io)?;
        info!(src = %src.display(), dst = %dst.display(), "file_move");
        Ok(())
    }

    fn file_delete(&self, step: &FileDeleteStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));
        match std::fs::remove_file(&path) {
            Ok(()) => {
                info!(path = %path.display(), "file_delete");
                Ok(())
            }
            Err(e) if step.ignore_missing && e.kind() == std::io::ErrorKind::NotFound => {
                info!(path = %path.display(), "file_delete: not found, ignored");
                Ok(())
            }
            Err(e) => Err(EngineError::Io(e)),
        }
    }

    fn file_rename(&self, step: &FileRenameStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));
        let new_name = vars.expand(&step.name);
        let dst = path
            .parent()
            .ok_or_else(|| EngineError::Other("file_rename: path has no parent".into()))?
            .join(&new_name);
        std::fs::rename(&path, &dst).map_err(EngineError::Io)?;
        info!(from = %path.display(), to = %dst.display(), "file_rename");
        Ok(())
    }

    // ── Log write method ────────────────────────────────────────────────────

    fn log_write(&self, step: &LogWriteStep, vars: &mut Variables) -> Result<()> {
        use std::io::Write;
        let path = self.base_dir.join(vars.expand(&step.file));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
        let message = vars.expand(&step.message);
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let level = match step.level {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        };
        let line = format!("[{ts}] [{level}] {message}\n");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(EngineError::Io)?;
        file.write_all(line.as_bytes()).map_err(EngineError::Io)?;
        info!(file = %path.display(), level, "log_write");
        Ok(())
    }

    // ── Date operation methods ──────────────────────────────────────────────

    fn date_format(&self, step: &DateFormatStep, vars: &mut Variables) -> Result<()> {
        let value = vars.expand(&step.value);
        let dt = chrono::NaiveDate::parse_from_str(&value, &step.from_format)
            .map_err(|e| EngineError::Other(format!("date_format parse error: {e}")))?;
        let result = dt.format(&step.to_format).to_string();
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn date_add(&self, step: &DateAddStep, vars: &mut Variables) -> Result<()> {
        use chrono::Datelike;
        let value = vars.expand(&step.value);
        let dt = chrono::NaiveDate::parse_from_str(&value, &step.format)
            .map_err(|e| EngineError::Other(format!("date_add parse error: {e}")))?;
        let dt = dt
            .checked_add_signed(chrono::Duration::days(step.days))
            .ok_or_else(|| EngineError::Other("date_add: days overflow".into()))?;
        // Apply months and years via calendar arithmetic.
        let total_months = step.months + step.years * 12;
        let dt = if total_months != 0 {
            let new_month0 = (dt.month0() as i64) + total_months;
            let year_delta = new_month0.div_euclid(12);
            let month = (new_month0.rem_euclid(12) as u32) + 1;
            let year = dt.year() + year_delta as i32;
            // Clamp day to last valid day of resulting month.
            let max_day = days_in_month(year, month);
            let day = dt.day().min(max_day);
            chrono::NaiveDate::from_ymd_opt(year, month, day)
                .ok_or_else(|| EngineError::Other("date_add: invalid result date".into()))?
        } else {
            dt
        };
        let result = dt.format(&step.format).to_string();
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn date_diff(&self, step: &DateDiffStep, vars: &mut Variables) -> Result<()> {
        use chrono::Datelike;
        let from_str = vars.expand(&step.from);
        let to_str = vars.expand(&step.to);
        let from = chrono::NaiveDate::parse_from_str(&from_str, &step.format)
            .map_err(|e| EngineError::Other(format!("date_diff parse 'from': {e}")))?;
        let to = chrono::NaiveDate::parse_from_str(&to_str, &step.format)
            .map_err(|e| EngineError::Other(format!("date_diff parse 'to': {e}")))?;
        let diff = match step.unit {
            DiffUnit::Days => (to - from).num_days(),
            DiffUnit::Months => {
                let months =
                    (to.year() - from.year()) * 12 + (to.month() as i32 - from.month() as i32);
                months as i64
            }
        };
        vars.set(&step.save_as, serde_json::Value::Number(diff.into()));
        Ok(())
    }

    // ── Mouse coordinate methods ────────────────────────────────────────────

    fn mouse_move(&self, step: &MouseMoveStep, vars: &Variables) -> Result<()> {
        let x = self.resolve_coord(&step.x, vars);
        let y = self.resolve_coord(&step.y, vars);
        if self.dry_run {
            info!(dry_run = true, x, y, "mouse_move skipped");
            return Ok(());
        }
        self.backend
            .move_mouse(rpa_template::ScreenPoint { x, y })
            .map_err(EngineError::Backend)
    }

    fn mouse_click_xy(&self, step: &MouseClickXyStep, vars: &Variables) -> Result<()> {
        let x = self.resolve_coord(&step.x, vars);
        let y = self.resolve_coord(&step.y, vars);
        if self.dry_run {
            info!(dry_run = true, x, y, "mouse_click_xy skipped");
            return Ok(());
        }
        let pt = rpa_template::ScreenPoint { x, y };
        match step.action {
            ClickAction::Left => self.backend.click(pt),
            ClickAction::Right => self.backend.right_click(pt),
            ClickAction::Double => self.backend.double_click(pt),
        }
        .map_err(EngineError::Backend)
    }

    fn mouse_drag(&self, step: &MouseDragStep, vars: &Variables) -> Result<()> {
        let from = rpa_template::ScreenPoint {
            x: self.resolve_coord(&step.from_x, vars),
            y: self.resolve_coord(&step.from_y, vars),
        };
        let to = rpa_template::ScreenPoint {
            x: self.resolve_coord(&step.to_x, vars),
            y: self.resolve_coord(&step.to_y, vars),
        };
        if self.dry_run {
            info!(dry_run = true, "mouse_drag skipped");
            return Ok(());
        }
        self.backend
            .drag(from, to, step.hold_ms)
            .map_err(EngineError::Backend)
    }

    fn mouse_scroll(&self, step: &MouseScrollStep, vars: &Variables) -> Result<()> {
        let direction = vars.expand(&step.direction);
        if self.dry_run {
            info!(dry_run = true, direction, "mouse_scroll skipped");
            return Ok(());
        }
        self.backend
            .scroll(&direction, step.amount)
            .map_err(EngineError::Backend)
    }

    /// Expand template variables in `s`, then parse as i32.
    /// Returns 0 on parse failure.
    fn resolve_coord(&self, s: &str, vars: &Variables) -> i32 {
        vars.expand(s).trim().parse::<i32>().unwrap_or(0)
    }

    // ── HTTP client methods ─────────────────────────────────────────────────

    /// Apply optional HTTP authentication to a request.
    #[cfg(feature = "http")]
    fn apply_auth(req: ureq::Request, auth: Option<&HttpAuth>) -> ureq::Request {
        match auth {
            None => req,
            Some(HttpAuth::Basic { user, password }) => {
                use base64::prelude::{Engine as _, BASE64_STANDARD};
                let encoded = BASE64_STANDARD.encode(format!("{user}:{password}"));
                req.set("Authorization", &format!("Basic {encoded}"))
            }
            Some(HttpAuth::Bearer { token }) => {
                req.set("Authorization", &format!("Bearer {token}"))
            }
        }
    }

    #[cfg(feature = "http")]
    async fn http_get(&self, step: &HttpGetStep, vars: &mut Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        let timeout = std::time::Duration::from_millis(step.timeout_ms);
        let headers: Vec<(String, String)> = step
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let auth = step.auth.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut req = ureq::get(&url).timeout(timeout);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            req = Self::apply_auth(req, auth.as_ref());
            Self::http_call(req)
        })
        .await
        .map_err(|e| EngineError::Other(format!("http_get join error: {e}")))?
        .map_err(|e| EngineError::Other(format!("http_get: {e}")))?;
        let status = result["status"].as_u64().unwrap_or(0);
        info!(url = %vars.expand(&step.url), status, "http_get");
        vars.set(&step.save_as, result);
        Ok(())
    }

    #[cfg(feature = "http")]
    async fn http_post(&self, step: &HttpPostStep, vars: &mut Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        let timeout = std::time::Duration::from_millis(step.timeout_ms);
        let body = step.body.clone();
        let content_type = step.content_type.clone();
        let headers: Vec<(String, String)> = step
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let auth = step.auth.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut req = ureq::post(&url).timeout(timeout);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            req = Self::apply_auth(req, auth.as_ref());
            Self::http_send_body(req, &content_type, body.as_ref())
        })
        .await
        .map_err(|e| EngineError::Other(format!("http_post join error: {e}")))?
        .map_err(|e| EngineError::Other(format!("http_post: {e}")))?;
        let status = result["status"].as_u64().unwrap_or(0);
        info!(url = %vars.expand(&step.url), status, "http_post");
        vars.set(&step.save_as, result);
        Ok(())
    }

    #[cfg(feature = "http")]
    async fn http_put(&self, step: &HttpPutStep, vars: &mut Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        let timeout = std::time::Duration::from_millis(step.timeout_ms);
        let body = step.body.clone();
        let content_type = step.content_type.clone();
        let headers: Vec<(String, String)> = step
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let auth = step.auth.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut req = ureq::put(&url).timeout(timeout);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            req = Self::apply_auth(req, auth.as_ref());
            Self::http_send_body(req, &content_type, body.as_ref())
        })
        .await
        .map_err(|e| EngineError::Other(format!("http_put join error: {e}")))?
        .map_err(|e| EngineError::Other(format!("http_put: {e}")))?;
        let status = result["status"].as_u64().unwrap_or(0);
        info!(url = %vars.expand(&step.url), status, "http_put");
        vars.set(&step.save_as, result);
        Ok(())
    }

    #[cfg(feature = "http")]
    async fn http_delete(&self, step: &HttpDeleteStep, vars: &mut Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        let timeout = std::time::Duration::from_millis(step.timeout_ms);
        let headers: Vec<(String, String)> = step
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let auth = step.auth.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut req = ureq::delete(&url).timeout(timeout);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            req = Self::apply_auth(req, auth.as_ref());
            Self::http_call(req)
        })
        .await
        .map_err(|e| EngineError::Other(format!("http_delete join error: {e}")))?
        .map_err(|e| EngineError::Other(format!("http_delete: {e}")))?;
        let status = result["status"].as_u64().unwrap_or(0);
        info!(url = %vars.expand(&step.url), status, "http_delete");
        vars.set(&step.save_as, result);
        Ok(())
    }

    #[cfg(feature = "http")]
    async fn http_patch(&self, step: &HttpPatchStep, vars: &mut Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        let timeout = std::time::Duration::from_millis(step.timeout_ms);
        let body = step.body.clone();
        let content_type = step.content_type.clone();
        let headers: Vec<(String, String)> = step
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let auth = step.auth.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut req = ureq::patch(&url).timeout(timeout);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            req = Self::apply_auth(req, auth.as_ref());
            Self::http_send_body(req, &content_type, body.as_ref())
        })
        .await
        .map_err(|e| EngineError::Other(format!("http_patch join error: {e}")))?
        .map_err(|e| EngineError::Other(format!("http_patch: {e}")))?;
        let status = result["status"].as_u64().unwrap_or(0);
        info!(url = %vars.expand(&step.url), status, "http_patch");
        vars.set(&step.save_as, result);
        Ok(())
    }

    /// Execute a GET-style request (no body) and return a `{status, body, body_json}` value.
    /// 4xx/5xx are treated as data, not errors.
    #[cfg(feature = "http")]
    fn http_call(req: ureq::Request) -> anyhow::Result<serde_json::Value> {
        let (status, body) = match req.call() {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.into_string()?;
                (status, body)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                (code, body)
            }
            Err(e) => return Err(anyhow::anyhow!("http error: {e}")),
        };
        let body_json = serde_json::from_str::<serde_json::Value>(&body).ok();
        Ok(serde_json::json!({
            "status": status,
            "body": body,
            "body_json": body_json,
        }))
    }

    /// Execute a POST/PUT request with the given body and return `{status, body, body_json}`.
    #[cfg(feature = "http")]
    fn http_send_body(
        req: ureq::Request,
        content_type: &ContentType,
        body: Option<&serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        let (status, body_str) = match content_type {
            ContentType::Json => {
                let payload = body.cloned().unwrap_or(serde_json::Value::Null);
                match req.send_json(payload) {
                    Ok(resp) => (resp.status(), resp.into_string()?),
                    Err(ureq::Error::Status(code, resp)) => {
                        (code, resp.into_string().unwrap_or_default())
                    }
                    Err(e) => return Err(anyhow::anyhow!("http error: {e}")),
                }
            }
            ContentType::Form => {
                // body should be a JSON object; convert to form pairs
                let pairs: Vec<(String, String)> = match body {
                    Some(serde_json::Value::Object(map)) => map
                        .iter()
                        .map(|(k, v)| {
                            let vs = match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            (k.clone(), vs)
                        })
                        .collect(),
                    _ => vec![],
                };
                let pairs_ref: Vec<(&str, &str)> = pairs
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                match req.send_form(&pairs_ref) {
                    Ok(resp) => (resp.status(), resp.into_string()?),
                    Err(ureq::Error::Status(code, resp)) => {
                        (code, resp.into_string().unwrap_or_default())
                    }
                    Err(e) => return Err(anyhow::anyhow!("http error: {e}")),
                }
            }
            ContentType::Text => {
                let text = match body {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => String::new(),
                };
                match req.send_string(&text) {
                    Ok(resp) => (resp.status(), resp.into_string()?),
                    Err(ureq::Error::Status(code, resp)) => {
                        (code, resp.into_string().unwrap_or_default())
                    }
                    Err(e) => return Err(anyhow::anyhow!("http error: {e}")),
                }
            }
        };
        let body_json = serde_json::from_str::<serde_json::Value>(&body_str).ok();
        Ok(serde_json::json!({
            "status": status,
            "body": body_str,
            "body_json": body_json,
        }))
    }

    // ── Excel cell methods ──────────────────────────────────────────────────

    fn excel_read_cell(&self, step: &ExcelReadCellStep, vars: &mut Variables) -> Result<()> {
        use calamine::{open_workbook_auto, Reader};

        let path = self.base_dir.join(vars.expand(&step.file));
        let cell_ref = vars.expand(&step.cell);

        let mut wb = open_workbook_auto(&path)
            .map_err(|e| EngineError::Other(format!("excel_read_cell open: {e}")))?;

        let sheet_name = step
            .sheet
            .clone()
            .unwrap_or_else(|| wb.sheet_names().first().cloned().unwrap_or_default());

        let range = wb.worksheet_range(&sheet_name).map_err(|e| {
            EngineError::Other(format!("excel_read_cell sheet '{sheet_name}': {e}"))
        })?;

        let (col, row) = parse_cell_ref(&cell_ref)
            .ok_or_else(|| EngineError::Other(format!("invalid cell ref: {cell_ref}")))?;

        let value = range
            .get_value((row, col))
            .map(excel_datatype_to_json)
            .unwrap_or(serde_json::Value::Null);

        vars.set(&step.save_as, value);
        info!(file = %path.display(), cell = %cell_ref, "excel_read_cell");
        Ok(())
    }

    fn excel_read_range(&self, step: &ExcelReadRangeStep, vars: &mut Variables) -> Result<()> {
        use calamine::{open_workbook_auto, Reader};

        let path = self.base_dir.join(vars.expand(&step.file));
        let range_ref = vars.expand(&step.range);

        let mut wb = open_workbook_auto(&path)
            .map_err(|e| EngineError::Other(format!("excel_read_range open: {e}")))?;

        let sheet_name = step
            .sheet
            .clone()
            .unwrap_or_else(|| wb.sheet_names().first().cloned().unwrap_or_default());

        let sheet = wb.worksheet_range(&sheet_name).map_err(|e| {
            EngineError::Other(format!("excel_read_range sheet '{sheet_name}': {e}"))
        })?;

        let (start_str, end_str) = range_ref
            .split_once(':')
            .ok_or_else(|| EngineError::Other(format!("invalid range: {range_ref}")))?;

        let (start_col, start_row) = parse_cell_ref(start_str)
            .ok_or_else(|| EngineError::Other(format!("invalid range start: {start_str}")))?;
        let (end_col, end_row) = parse_cell_ref(end_str)
            .ok_or_else(|| EngineError::Other(format!("invalid range end: {end_str}")))?;

        let mut rows: Vec<serde_json::Value> = Vec::new();
        for r in start_row..=end_row {
            let mut row: Vec<serde_json::Value> = Vec::new();
            for c in start_col..=end_col {
                let v = sheet
                    .get_value((r, c))
                    .map(excel_datatype_to_json)
                    .unwrap_or(serde_json::Value::Null);
                row.push(v);
            }
            rows.push(serde_json::Value::Array(row));
        }

        info!(file = %path.display(), range = %range_ref, rows = rows.len(), "excel_read_range");
        vars.set(&step.save_as, serde_json::Value::Array(rows));
        Ok(())
    }

    fn excel_write_cell(&self, step: &ExcelWriteCellStep, vars: &mut Variables) -> Result<()> {
        #[cfg(not(feature = "excel-write"))]
        {
            let _ = step;
            let _ = vars;
            Err(EngineError::Other(
                "excel_write_cell requires the 'excel-write' feature; rebuild with: cargo build --features excel-write"
                    .to_owned(),
            ))
        }

        #[cfg(feature = "excel-write")]
        {
            let path = self.base_dir.join(vars.expand(&step.file));
            let cell_ref = vars.expand(&step.cell);
            let value = vars.expand(&step.value);

            let sheet_name = step.sheet.clone().unwrap_or_else(|| "Sheet1".to_owned());

            let (col, row) = parse_cell_ref(&cell_ref)
                .ok_or_else(|| EngineError::Other(format!("invalid cell ref: {cell_ref}")))?;

            // umya-spreadsheet uses 1-based (col, row) tuple
            let col1 = col + 1;
            let row1 = row + 1;

            let mut wb = if path.exists() {
                umya_spreadsheet::reader::xlsx::read(&path)
                    .map_err(|e| EngineError::Other(format!("excel_write_cell read: {e}")))?
            } else {
                umya_spreadsheet::new_file_empty_worksheet()
            };

            // Get or create the sheet.
            if wb.get_sheet_by_name(&sheet_name).is_none() {
                wb.new_sheet(&sheet_name)
                    .map_err(|e| EngineError::Other(format!("excel_write_cell new_sheet: {e}")))?;
            }

            let sheet = wb
                .get_sheet_by_name_mut(&sheet_name)
                .ok_or_else(|| EngineError::Other(format!("sheet not found: {sheet_name}")))?;

            sheet
                .get_cell_mut((col1, row1))
                .set_value_string(value.clone());

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
            }
            umya_spreadsheet::writer::xlsx::write(&wb, &path)
                .map_err(|e| EngineError::Other(format!("excel_write_cell write: {e}")))?;

            info!(file = %path.display(), cell = %cell_ref, value = %value, "excel_write_cell");
            Ok(())
        }
    }

    // ── Mail receive method ─────────────────────────────────────────────────

    async fn mail_receive(&self, step: &MailReceiveStep, vars: &mut Variables) -> Result<()> {
        let host = vars.expand(&step.host);
        let user = vars.expand(&step.user);
        let password = vars.expand(&step.password);
        let folder = vars.expand(&step.folder);
        let port = step.port;
        let count = step.count;
        let only_unseen = step.only_unseen;

        let result = tokio::task::spawn_blocking(move || {
            let mut inputs = std::collections::HashMap::new();
            inputs.insert("host".to_owned(), serde_json::Value::String(host));
            inputs.insert("user".to_owned(), serde_json::Value::String(user));
            inputs.insert("password".to_owned(), serde_json::Value::String(password));
            inputs.insert("folder".to_owned(), serde_json::Value::String(folder));
            inputs.insert("port".to_owned(), serde_json::Value::Number(port.into()));
            inputs.insert("count".to_owned(), serde_json::Value::Number(count.into()));
            inputs.insert(
                "only_unseen".to_owned(),
                serde_json::Value::Bool(only_unseen),
            );
            rpa_stdlib::dispatch("mail.imap_receive", inputs)
        })
        .await
        .map_err(|e| EngineError::Other(format!("mail_receive join: {e}")))?
        .map_err(|e| EngineError::Other(format!("mail_receive: {e}")))?;

        let messages = result
            .get("messages")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));
        let count = messages.as_array().map(|a| a.len()).unwrap_or(0);
        info!(count, "mail_receive");
        vars.set(&step.save_as, messages);
        Ok(())
    }

    // ── Excel sheet management methods ─────────────────────────────────────

    fn excel_add_sheet(&self, step: &ExcelAddSheetStep, vars: &Variables) -> Result<()> {
        let file = self.base_dir.join(vars.expand(&step.file));
        let name = vars.expand(&step.name);
        let mut inputs = std::collections::HashMap::new();
        inputs.insert(
            "file".to_owned(),
            serde_json::Value::String(file.to_string_lossy().into_owned()),
        );
        inputs.insert("name".to_owned(), serde_json::Value::String(name.clone()));
        rpa_stdlib::dispatch("excel.add_sheet", inputs)
            .map_err(|e| EngineError::Other(format!("excel_add_sheet: {e}")))?;
        info!(file = %file.display(), name = %name, "excel_add_sheet");
        Ok(())
    }

    fn excel_delete_sheet(&self, step: &ExcelDeleteSheetStep, vars: &Variables) -> Result<()> {
        let file = self.base_dir.join(vars.expand(&step.file));
        let name = vars.expand(&step.name);
        let mut inputs = std::collections::HashMap::new();
        inputs.insert(
            "file".to_owned(),
            serde_json::Value::String(file.to_string_lossy().into_owned()),
        );
        inputs.insert("name".to_owned(), serde_json::Value::String(name.clone()));
        rpa_stdlib::dispatch("excel.delete_sheet", inputs)
            .map_err(|e| EngineError::Other(format!("excel_delete_sheet: {e}")))?;
        info!(file = %file.display(), name = %name, "excel_delete_sheet");
        Ok(())
    }

    fn excel_rename_sheet(&self, step: &ExcelRenameSheetStep, vars: &Variables) -> Result<()> {
        let file = self.base_dir.join(vars.expand(&step.file));
        let from_name = vars.expand(&step.from_name);
        let to_name = vars.expand(&step.to_name);
        let mut inputs = std::collections::HashMap::new();
        inputs.insert(
            "file".to_owned(),
            serde_json::Value::String(file.to_string_lossy().into_owned()),
        );
        inputs.insert(
            "from_name".to_owned(),
            serde_json::Value::String(from_name.clone()),
        );
        inputs.insert(
            "to_name".to_owned(),
            serde_json::Value::String(to_name.clone()),
        );
        rpa_stdlib::dispatch("excel.rename_sheet", inputs)
            .map_err(|e| EngineError::Other(format!("excel_rename_sheet: {e}")))?;
        info!(file = %file.display(), from = %from_name, to = %to_name, "excel_rename_sheet");
        Ok(())
    }

    // ── Text file read/write methods ────────────────────────────────────────

    fn file_read(&self, step: &FileReadStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));
        let content = std::fs::read_to_string(&path).map_err(EngineError::Io)?;
        info!(path = %path.display(), bytes = content.len(), "file_read");
        vars.set(&step.save_as, serde_json::Value::String(content));
        Ok(())
    }

    fn file_write(&self, step: &FileWriteStep, vars: &mut Variables) -> Result<()> {
        use std::io::Write;
        let path = self.base_dir.join(vars.expand(&step.path));
        let content = vars.expand(&step.content);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
        let append = step.mode == FileWriteMode::Append;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(!append)
            .truncate(!append)
            .append(append)
            .open(&path)
            .map_err(EngineError::Io)?;
        file.write_all(content.as_bytes())
            .map_err(EngineError::Io)?;
        info!(path = %path.display(), mode = ?step.mode, "file_write");
        Ok(())
    }

    fn file_append(&self, step: &FileAppendStep, vars: &mut Variables) -> Result<()> {
        use std::io::Write;
        let path = self.base_dir.join(vars.expand(&step.path));
        let content = vars.expand(&step.content);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(EngineError::Io)?;
        file.write_all(content.as_bytes())
            .map_err(EngineError::Io)?;
        info!(path = %path.display(), "file_append");
        Ok(())
    }

    // ── Process operation methods ───────────────────────────────────────────

    async fn process_start(&self, step: &ProcessStartStep, vars: &mut Variables) -> Result<()> {
        let command = vars.expand(&step.command);
        let args: Vec<String> = step.args.iter().map(|a| vars.expand(a)).collect();
        let child = std::process::Command::new(&command)
            .args(&args)
            .spawn()
            .map_err(EngineError::Io)?;
        let pid = child.id();
        if step.wait_ms > 0 {
            sleep(Duration::from_millis(step.wait_ms)).await;
        }
        if let Some(save_as) = &step.save_pid_as {
            vars.set(save_as.clone(), serde_json::Value::Number(pid.into()));
        }
        info!(command = %command, pid, "process_start");
        Ok(())
    }

    fn process_kill(&self, step: &ProcessKillStep, vars: &mut Variables) -> Result<()> {
        if let Some(pid_tpl) = &step.pid {
            let pid_str = vars.expand(pid_tpl);
            let pid = pid_str
                .trim()
                .parse::<u32>()
                .map_err(|_| EngineError::Other(format!("process_kill: invalid pid: {pid_str}")))?;
            Self::kill_by_pid(pid)?;
            info!(pid, "process_kill");
        } else if let Some(name_tpl) = &step.name {
            let name = vars.expand(name_tpl);
            Self::kill_by_name(&name)?;
            info!(name = %name, "process_kill");
        } else {
            return Err(EngineError::Other(
                "process_kill: must specify pid or name".into(),
            ));
        }
        Ok(())
    }

    fn process_exists(&self, step: &ProcessExistsStep, vars: &mut Variables) -> Result<()> {
        let name = vars.expand(&step.name);
        let exists = Self::check_process_exists(&name)?;
        vars.set(&step.save_as, serde_json::Value::Bool(exists));
        info!(name = %name, exists, "process_exists");
        Ok(())
    }

    #[cfg(unix)]
    fn kill_by_pid(pid: u32) -> Result<()> {
        let status = std::process::Command::new("kill")
            .args(["-15", &pid.to_string()])
            .status()
            .map_err(EngineError::Io)?;
        if !status.success() {
            return Err(EngineError::Other(format!(
                "process_kill: kill -15 {pid} failed"
            )));
        }
        Ok(())
    }

    #[cfg(windows)]
    fn kill_by_pid(pid: u32) -> Result<()> {
        std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map_err(EngineError::Io)?;
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    fn kill_by_pid(_pid: u32) -> Result<()> {
        Err(EngineError::Other(
            "process_kill by pid not supported on this platform".into(),
        ))
    }

    #[cfg(unix)]
    fn kill_by_name(name: &str) -> Result<()> {
        std::process::Command::new("pkill")
            .args(["-f", name])
            .status()
            .map_err(EngineError::Io)?;
        Ok(())
    }

    #[cfg(windows)]
    fn kill_by_name(name: &str) -> Result<()> {
        std::process::Command::new("taskkill")
            .args(["/IM", name, "/F"])
            .status()
            .map_err(EngineError::Io)?;
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    fn kill_by_name(_name: &str) -> Result<()> {
        Err(EngineError::Other(
            "process_kill by name not supported on this platform".into(),
        ))
    }

    #[cfg(unix)]
    fn check_process_exists(name: &str) -> Result<bool> {
        let output = std::process::Command::new("pgrep")
            .args(["-x", name])
            .output()
            .map_err(EngineError::Io)?;
        Ok(output.status.success())
    }

    #[cfg(windows)]
    fn check_process_exists(name: &str) -> Result<bool> {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {name}"), "/NH"])
            .output()
            .map_err(EngineError::Io)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_lowercase().contains(&name.to_lowercase()))
    }

    #[cfg(not(any(unix, windows)))]
    fn check_process_exists(_name: &str) -> Result<bool> {
        Err(EngineError::Other(
            "process_exists not supported on this platform".into(),
        ))
    }

    // ── Key combination method ──────────────────────────────────────────────

    fn key_combo(&self, step: &KeyComboStep) -> Result<()> {
        if self.dry_run {
            info!(dry_run = true, keys = ?step.keys, "key_combo skipped");
            return Ok(());
        }
        let key_refs: Vec<&str> = step.keys.iter().map(|k| k.as_str()).collect();
        self.backend
            .key_combo(&key_refs)
            .map_err(EngineError::Backend)
    }

    // ── CSV read/write methods ──────────────────────────────────────────────

    fn csv_read(&self, step: &CsvReadStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));

        let value = if step.has_header {
            // Reuse data_source::load — returns Vec<HashMap<String, Value>>
            let rows = crate::data_source::load(&path, None)
                .map_err(|e| EngineError::Other(format!("csv_read: {e}")))?;
            let json_rows: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| serde_json::Value::Object(row.into_iter().collect()))
                .collect();
            serde_json::Value::Array(json_rows)
        } else {
            // Raw rows without header: each row is a list of strings
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_path(&path)
                .map_err(|e| EngineError::Other(format!("csv_read: {e}")))?;
            let mut rows: Vec<serde_json::Value> = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| EngineError::Other(format!("csv_read: {e}")))?;
                let row: Vec<serde_json::Value> = record
                    .iter()
                    .map(|s| serde_json::Value::String(s.to_owned()))
                    .collect();
                rows.push(serde_json::Value::Array(row));
            }
            serde_json::Value::Array(rows)
        };

        info!(path = %path.display(), "csv_read");
        vars.set(&step.save_as, value);
        Ok(())
    }

    fn csv_write(&self, step: &CsvWriteStep, vars: &mut Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }

        let rows_val = vars
            .get(&step.rows)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let rows = match &rows_val {
            serde_json::Value::Array(arr) => arr,
            _ => {
                return Err(EngineError::Other(format!(
                    "csv_write: '{}' is not an array",
                    step.rows
                )))
            }
        };

        let append = step.mode == CsvWriteMode::Append;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(!append)
            .truncate(!append)
            .append(append)
            .open(&path)
            .map_err(EngineError::Io)?;
        let mut wtr = csv::Writer::from_writer(file);

        // Determine headers and row-writing strategy.
        if let Some(serde_json::Value::Object(_)) = rows.first() {
            // list<map>: infer headers from the first row (or step.headers override)
            let headers: Vec<String> = if !step.headers.is_empty() {
                step.headers.clone()
            } else {
                match rows.first() {
                    Some(serde_json::Value::Object(map)) => map.keys().cloned().collect(),
                    _ => vec![],
                }
            };
            if !append {
                wtr.write_record(&headers)
                    .map_err(|e| EngineError::Other(format!("csv_write header: {e}")))?;
            }
            for row in rows {
                if let serde_json::Value::Object(map) = row {
                    let record: Vec<String> = headers
                        .iter()
                        .map(|h| match map.get(h) {
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(v) => v.to_string(),
                            None => String::new(),
                        })
                        .collect();
                    wtr.write_record(&record)
                        .map_err(|e| EngineError::Other(format!("csv_write row: {e}")))?;
                }
            }
        } else {
            // list<list>: write headers if provided and not appending
            if !append && !step.headers.is_empty() {
                wtr.write_record(&step.headers)
                    .map_err(|e| EngineError::Other(format!("csv_write header: {e}")))?;
            }
            for row in rows {
                let record: Vec<String> = match row {
                    serde_json::Value::Array(items) => items
                        .iter()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect(),
                    other => vec![other.to_string()],
                };
                wtr.write_record(&record)
                    .map_err(|e| EngineError::Other(format!("csv_write row: {e}")))?;
            }
        }

        wtr.flush()
            .map_err(|e| EngineError::Other(format!("csv_write flush: {e}")))?;
        info!(path = %path.display(), rows = rows.len(), "csv_write");
        Ok(())
    }

    // ── String operation methods ────────────────────────────────────────────

    fn string_replace(&self, step: &StringReplaceStep, vars: &mut Variables) -> Result<()> {
        let value = vars.expand(&step.value);
        let from = vars.expand(&step.from);
        let to = vars.expand(&step.to);
        let result = if step.all {
            value.replace(&*from, &to)
        } else {
            value.replacen(&*from, &to, 1)
        };
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn string_trim(&self, step: &StringTrimStep, vars: &mut Variables) -> Result<()> {
        let value = vars.expand(&step.value);
        let result = match step.side {
            TrimSide::Both => value.trim().to_owned(),
            TrimSide::Left => value.trim_start().to_owned(),
            TrimSide::Right => value.trim_end().to_owned(),
        };
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn string_substring(&self, step: &StringSubstringStep, vars: &mut Variables) -> Result<()> {
        let value = vars.expand(&step.value);
        let chars: Vec<char> = value.chars().collect();
        let start = step.start.min(chars.len());
        let result: String = match step.length {
            Some(len) => chars[start..].iter().take(len).collect(),
            None => chars[start..].iter().collect(),
        };
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn string_split(&self, step: &StringSplitStep, vars: &mut Variables) -> Result<()> {
        let value = vars.expand(&step.value);
        let delim = vars.expand(&step.delimiter);
        let parts: Vec<serde_json::Value> = value
            .split(&*delim)
            .map(|s| serde_json::Value::String(s.to_owned()))
            .collect();
        vars.set(&step.save_as, serde_json::Value::Array(parts));
        Ok(())
    }

    fn string_join(&self, step: &StringJoinStep, vars: &mut Variables) -> Result<()> {
        let separator = vars.expand(&step.separator);
        let arr = vars
            .get(&step.value)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let result = match &arr {
            serde_json::Value::Array(items) => items
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(&separator),
            other => other.to_string(),
        };
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn string_regex(&self, step: &StringRegexStep, vars: &mut Variables) -> Result<()> {
        let value = vars.expand(&step.value);
        let pattern = vars.expand(&step.pattern);
        let re = regex::Regex::new(&pattern)
            .map_err(|e| EngineError::Other(format!("string_regex: invalid pattern: {e}")))?;
        let result = if let Some(caps) = re.captures(&value) {
            let full = caps.get(0).map(|m| m.as_str()).unwrap_or("").to_owned();
            let groups: Vec<serde_json::Value> = (1..caps.len())
                .map(|i| {
                    serde_json::Value::String(
                        caps.get(i).map(|m| m.as_str()).unwrap_or("").to_owned(),
                    )
                })
                .collect();
            serde_json::json!({ "found": true, "full": full, "groups": groups })
        } else {
            serde_json::json!({ "found": false, "full": "", "groups": [] })
        };
        vars.set(&step.save_as, result);
        Ok(())
    }

    // ── JSON helper methods ─────────────────────────────────────────────────

    fn json_parse(&self, step: &JsonParseStep, vars: &mut Variables) -> Result<()> {
        let text = vars.expand(&step.value);
        let val: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| EngineError::Other(format!("json_parse: {e}")))?;
        vars.set(&step.save_as, val);
        Ok(())
    }

    fn json_stringify(&self, step: &JsonStringifyStep, vars: &mut Variables) -> Result<()> {
        let val = vars
            .get(&step.value)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let text = serde_json::to_string(&val)
            .map_err(|e| EngineError::Other(format!("json_stringify: {e}")))?;
        vars.set(&step.save_as, serde_json::Value::String(text));
        Ok(())
    }

    // ── Path helper methods ─────────────────────────────────────────────────

    fn path_join(&self, step: &PathJoinStep, vars: &mut Variables) -> Result<()> {
        let mut path = std::path::PathBuf::new();
        for part in &step.parts {
            path.push(vars.expand(part));
        }
        let result = path.to_string_lossy().into_owned();
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn path_basename(&self, step: &PathBasenameStep, vars: &mut Variables) -> Result<()> {
        let path = std::path::Path::new(&vars.expand(&step.path)).to_path_buf();
        let result = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    fn path_dirname(&self, step: &PathDirnameStep, vars: &mut Variables) -> Result<()> {
        let path = std::path::Path::new(&vars.expand(&step.path)).to_path_buf();
        let result = path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        vars.set(&step.save_as, serde_json::Value::String(result));
        Ok(())
    }

    // ── Env / file-list methods ─────────────────────────────────────────────

    fn env_get(&self, step: &EnvGetStep, vars: &mut Variables) -> Result<()> {
        let val = match std::env::var(&step.name) {
            Ok(v) => v,
            Err(_) => step
                .default
                .clone()
                .ok_or_else(|| EngineError::Other(format!("env var not set: {}", step.name)))?,
        };
        vars.set(&step.save_as, serde_json::Value::String(val));
        Ok(())
    }

    fn file_list(&self, step: &FileListStep, vars: &mut Variables) -> Result<()> {
        let dir = self.base_dir.join(vars.expand(&step.dir));
        let pattern = vars.expand(&step.pattern);
        let glob_pat = dir.join(&pattern);
        let glob_str = glob_pat.to_string_lossy();

        let mut entries: Vec<serde_json::Value> = Vec::new();
        for path in glob::glob(&glob_str)
            .map_err(|e| EngineError::Other(format!("file_list: invalid glob: {e}")))?
            .flatten()
        {
            let meta = std::fs::metadata(&path).ok();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            entries.push(serde_json::json!({
                "name": name,
                "path": path.to_string_lossy().as_ref(),
                "is_dir": is_dir,
                "size_bytes": size,
            }));
        }
        info!(dir = %dir.display(), count = entries.len(), "file_list");
        vars.set(&step.save_as, serde_json::Value::Array(entries));
        Ok(())
    }

    // ── Utilities ───────────────────────────────────────────────────────────

    fn load_image(&self, path: &PathBuf) -> Result<image::RgbaImage> {
        image::open(path)
            .map(|i| i.into_rgba8())
            .map_err(|source| EngineError::ImageLoad {
                path: path.clone(),
                source,
            })
    }

    async fn save_failure_screenshot(&self, step_index: usize) -> Option<PathBuf> {
        let _ = tokio::fs::create_dir_all(&self.screenshot_dir).await;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = self
            .screenshot_dir
            .join(format!("fail_step{step_index}_{ts}.png"));

        if let Ok(img) = self.backend.capture(&rpa_template::Target::Screen) {
            if let Err(e) = img.save(&path) {
                warn!(error = %e, "failed to save failure screenshot");
                None
            } else {
                info!(path = %path.display(), "failure screenshot saved");
                Some(path)
            }
        } else {
            None
        }
    }
}

// ── Reconnect helpers ──────────────────────────────────────────────────────

fn is_window_not_found(e: &EngineError) -> bool {
    matches!(e, EngineError::Backend(be) if be.is_window_not_found())
}

fn uia_selector_from_by(by: &UiaBy, vars: &Variables) -> rpa_uia::UiaSelector {
    match by {
        UiaBy::Name(s) => rpa_uia::UiaSelector::from_name(vars.expand(s)),
        UiaBy::Id(s) => rpa_uia::UiaSelector::from_id(vars.expand(s)),
        UiaBy::Class(s) => rpa_uia::UiaSelector::from_class(vars.expand(s)),
    }
}

/// Returns true for composite/flow steps whose inner run_steps already handles
/// reconnect retries. These must NOT be retried at the outer level to avoid
/// re-executing successfully-completed inner steps.
fn is_flow_step(step: &ScenarioStep) -> bool {
    matches!(
        step,
        ScenarioStep::Group(_)
            | ScenarioStep::If(_)
            | ScenarioStep::Switch(_)
            | ScenarioStep::Repeat(_)
            | ScenarioStep::While(_)
            | ScenarioStep::DoWhile(_)
            | ScenarioStep::TryCatch(_)
            | ScenarioStep::Foreach(_)
            | ScenarioStep::SubScenario(_)
            | ScenarioStep::CallScenario(_)
    )
}

// ── Excel helpers ─────────────────────────────────────────────────────────

/// Parse A1-notation cell reference into (col_index, row_index), 0-based.
/// "A1" → (0, 0), "B5" → (1, 4), "AA10" → (26, 9)
fn parse_cell_ref(s: &str) -> Option<(u32, u32)> {
    let s = s.trim().to_uppercase();
    let col_part: String = s.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let row_part: String = s.chars().skip_while(|c| c.is_ascii_alphabetic()).collect();
    if col_part.is_empty() || row_part.is_empty() {
        return None;
    }
    let col = col_part
        .chars()
        .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));
    let row: u32 = row_part.parse().ok()?;
    Some((col - 1, row - 1))
}

fn excel_datatype_to_json(d: &calamine::Data) -> serde_json::Value {
    use calamine::Data;
    match d {
        Data::Int(n) => serde_json::Value::Number((*n).into()),
        Data::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::String(f.to_string())),
        Data::String(s) => serde_json::Value::String(s.clone()),
        Data::Bool(b) => serde_json::Value::Bool(*b),
        Data::Empty => serde_json::Value::Null,
        other => serde_json::Value::String(other.to_string()),
    }
}

// ── Date helpers ──────────────────────────────────────────────────────────

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

// ── Pure string transformations ────────────────────────────────────────────

/// ASCII printable (0x21-0x7E) → full-width (U+FF01-U+FF5E); space → U+3000.
fn to_fullwidth(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => '\u{3000}',
            c if ('!'..='~').contains(&c) => char::from_u32(c as u32 + 0xFEE0).unwrap_or(c),
            c => c,
        })
        .collect()
}

/// Full-width (U+FF01-U+FF5E) → ASCII; U+3000 → space.
fn to_halfwidth(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{3000}' => ' ',
            c if ('\u{FF01}'..='\u{FF5E}').contains(&c) => {
                char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
            }
            c => c,
        })
        .collect()
}

// ── Rhai Dynamic → serde_json::Value ───────────────────────────────────────

fn dynamic_to_json(d: &rhai::Dynamic) -> serde_json::Value {
    if d.is_unit() {
        serde_json::Value::Null
    } else if let Ok(b) = d.as_bool() {
        serde_json::Value::Bool(b)
    } else if let Ok(i) = d.as_int() {
        serde_json::Value::Number(i.into())
    } else if let Ok(f) = d.as_float() {
        serde_json::json!(f)
    } else if d.is_string() {
        serde_json::Value::String(d.to_string())
    } else if let Some(arr) = d.clone().try_cast::<rhai::Array>() {
        serde_json::Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if let Some(map) = d.clone().try_cast::<rhai::Map>() {
        let obj: serde_json::Map<String, serde_json::Value> = map
            .iter()
            .map(|(k, v)| (k.to_string(), dynamic_to_json(v)))
            .collect();
        serde_json::Value::Object(obj)
    } else {
        serde_json::Value::String(d.to_string())
    }
}

// ── Screenshot save ─────────────────────────────────────────────────────────

impl ScenarioEngine {
    async fn screenshot_save(&self, step: &ScreenshotSaveStep, vars: &Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.path));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let target = match &step.window {
            Some(w) => rpa_template::Target::Window {
                title_contains: w.clone(),
            },
            None => rpa_template::Target::Screen,
        };
        let backend = Arc::clone(&self.backend);
        let img = tokio::task::spawn_blocking(move || backend.capture(&target))
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))??;
        img.save(&path)
            .map_err(|e| EngineError::Other(format!("screenshot_save: {e}")))?;
        info!(path = %path.display(), "screenshot saved");
        Ok(())
    }

    // ── Wait no image ───────────────────────────────────────────────────────

    async fn wait_no_image(&self, step: &WaitNoImageStep, vars: &Variables) -> Result<()> {
        let path = self.base_dir.join(vars.expand(&step.template));
        let template = Arc::new(self.load_image(&path)?);
        let deadline = Instant::now() + Duration::from_millis(step.timeout_ms);
        let target = Self::capture_target(&step.window);

        loop {
            let backend = Arc::clone(&self.backend);
            let matcher = self.matcher.clone();
            let tmpl = Arc::clone(&template);
            let tgt = target.clone();

            let result: Result<rpa_vision::MatchResult> = tokio::task::spawn_blocking(move || {
                let (img, origin) = backend.capture_with_origin(&tgt)?;
                Ok(matcher.find_with_masks(&img, &tmpl, origin, &[])?)
            })
            .await
            .map_err(|e| EngineError::TaskPanic(e.to_string()))?;

            match result {
                Ok(_) => {
                    if Instant::now() >= deadline {
                        return Err(EngineError::Timeout(format!(
                            "wait_no_image: '{}' still visible after {}ms",
                            step.template, step.timeout_ms
                        )));
                    }
                    self.check_cancelled()?;
                    sleep(Duration::from_millis(step.interval_ms)).await;
                }
                Err(EngineError::Vision(_)) => {
                    info!("template disappeared");
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
    }

    // ── URL open ────────────────────────────────────────────────────────────

    fn url_open(&self, step: &UrlOpenStep, vars: &Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        open::that(&url)?;
        info!(url = %url, "url opened");
        Ok(())
    }

    // ── Notify ──────────────────────────────────────────────────────────────

    fn notify_step(&self, step: &NotifyStep, vars: &Variables) -> Result<()> {
        let title = vars.expand(&step.title);
        let message = vars.expand(&step.message);
        notify_rust::Notification::new()
            .summary(&title)
            .body(&message)
            .show()
            .map_err(|e| EngineError::Other(format!("notify: {e}")))?;
        info!(title = %title, "notification sent");
        Ok(())
    }

    // ── UI Automation methods ───────────────────────────────────────────────

    async fn uia_get(&self, step: &UiaGetStep, vars: &mut Variables) -> Result<()> {
        let selector = uia_selector_from_by(&step.by, vars);
        let property = step.property.clone();
        let timeout_ms = step.timeout_ms;
        let save_as = step.save_as.clone();

        let result = tokio::task::spawn_blocking(move || {
            let finder = rpa_uia::UiaFinder::new()
                .map_err(|e| EngineError::Other(format!("uia_get: {e}")))?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            loop {
                match finder.find(&selector) {
                    Ok(el) => {
                        let value = match property.as_str() {
                            "name" => el.get_name(),
                            "value" => el.get_value(),
                            _ => el.get_value(),
                        };
                        return value.map_err(|e| EngineError::Other(format!("uia_get: {e}")));
                    }
                    Err(_) if std::time::Instant::now() < deadline => {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Err(e) => return Err(EngineError::Other(format!("uia_get: {e}"))),
                }
            }
        })
        .await
        .map_err(|e| EngineError::TaskPanic(e.to_string()))??;

        vars.set(save_as, serde_json::Value::String(result));
        Ok(())
    }

    async fn uia_set(&self, step: &UiaSetStep, vars: &Variables) -> Result<()> {
        let selector = uia_selector_from_by(&step.by, vars);
        let value = vars.expand(&step.value);
        let timeout_ms = step.timeout_ms;

        tokio::task::spawn_blocking(move || {
            let finder = rpa_uia::UiaFinder::new()
                .map_err(|e| EngineError::Other(format!("uia_set: {e}")))?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            loop {
                match finder.find(&selector) {
                    Ok(el) => {
                        return el
                            .set_value(&value)
                            .map_err(|e| EngineError::Other(format!("uia_set: {e}")));
                    }
                    Err(_) if std::time::Instant::now() < deadline => {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Err(e) => return Err(EngineError::Other(format!("uia_set: {e}"))),
                }
            }
        })
        .await
        .map_err(|e| EngineError::TaskPanic(e.to_string()))?
    }

    async fn uia_click(&self, step: &UiaClickStep, vars: &Variables) -> Result<()> {
        let selector = uia_selector_from_by(&step.by, vars);
        let timeout_ms = step.timeout_ms;

        if self.dry_run {
            info!(dry_run = true, "uia_click skipped");
            return Ok(());
        }

        tokio::task::spawn_blocking(move || {
            let finder = rpa_uia::UiaFinder::new()
                .map_err(|e| EngineError::Other(format!("uia_click: {e}")))?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            loop {
                match finder.find(&selector) {
                    Ok(el) => {
                        return el
                            .invoke()
                            .map_err(|e| EngineError::Other(format!("uia_click: {e}")));
                    }
                    Err(_) if std::time::Instant::now() < deadline => {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Err(e) => return Err(EngineError::Other(format!("uia_click: {e}"))),
                }
            }
        })
        .await
        .map_err(|e| EngineError::TaskPanic(e.to_string()))?
    }

    async fn uia_find(&self, step: &UiaFindStep, vars: &mut Variables) -> Result<()> {
        let selector = uia_selector_from_by(&step.by, vars);
        let timeout_ms = step.timeout_ms;
        let save_as = step.save_as.clone();

        let result = tokio::task::spawn_blocking(move || {
            let finder = rpa_uia::UiaFinder::new()
                .map_err(|e| EngineError::Other(format!("uia_find: {e}")))?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            loop {
                match finder.find(&selector) {
                    Ok(el) => {
                        let name = el.get_name().unwrap_or_default();
                        let rect = el.bounding_rect().unwrap_or((0, 0, 0, 0));
                        return Ok((name, rect));
                    }
                    Err(_) if std::time::Instant::now() < deadline => {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Err(e) => return Err(EngineError::Other(format!("uia_find: {e}"))),
                }
            }
        })
        .await
        .map_err(|e| EngineError::TaskPanic(e.to_string()))??;

        let (name, (x, y, w, h)) = result;
        vars.set(
            save_as,
            serde_json::json!({ "name": name, "x": x, "y": y, "width": w, "height": h }),
        );
        Ok(())
    }

    // ── Web automation methods ──────────────────────────────────────────────

    #[cfg(feature = "web")]
    async fn web_open(&self, step: &WebOpenStep, vars: &Variables) -> Result<()> {
        let url = vars.expand(&step.url);
        let driver = vars.expand(&step.driver);
        let session = rpa_web::WebSession::new(&driver)
            .await
            .map_err(|e| EngineError::Other(format!("web_open: {e}")))?;
        session
            .open(&url)
            .await
            .map_err(|e| EngineError::Other(format!("web_open navigate: {e}")))?;
        *self.web_session.lock().await = Some(session);
        info!(url, "web_open");
        Ok(())
    }

    #[cfg(feature = "web")]
    async fn web_click(&self, step: &WebClickStep, vars: &Variables) -> Result<()> {
        let selector = vars.expand(&step.selector);
        let guard = self.web_session.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| EngineError::Other("web_click: no active session".to_owned()))?;
        if !self.dry_run {
            session
                .click(&selector, step.timeout_ms)
                .await
                .map_err(|e| EngineError::Other(format!("web_click: {e}")))?;
        } else {
            info!(dry_run = true, selector, "web_click skipped");
        }
        Ok(())
    }

    #[cfg(feature = "web")]
    async fn web_type(&self, step: &WebTypeStep, vars: &Variables) -> Result<()> {
        let selector = vars.expand(&step.selector);
        let text = vars.expand(&step.text);
        let guard = self.web_session.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| EngineError::Other("web_type: no active session".to_owned()))?;
        if !self.dry_run {
            session
                .type_text(&selector, &text, step.clear, step.timeout_ms)
                .await
                .map_err(|e| EngineError::Other(format!("web_type: {e}")))?;
        } else {
            info!(dry_run = true, selector, "web_type skipped");
        }
        Ok(())
    }

    #[cfg(feature = "web")]
    async fn web_get(&self, step: &WebGetStep, vars: &mut Variables) -> Result<()> {
        let selector = vars.expand(&step.selector);
        let guard = self.web_session.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| EngineError::Other("web_get: no active session".to_owned()))?;
        let value = if let Some(attr) = &step.attr {
            session
                .get_attr(&selector, attr, step.timeout_ms)
                .await
                .map_err(|e| EngineError::Other(format!("web_get: {e}")))?
                .unwrap_or_default()
        } else {
            session
                .get_text(&selector, step.timeout_ms)
                .await
                .map_err(|e| EngineError::Other(format!("web_get: {e}")))?
        };
        vars.set(step.save_as.clone(), serde_json::Value::String(value));
        Ok(())
    }

    #[cfg(feature = "web")]
    async fn web_wait(&self, step: &WebWaitStep, vars: &Variables) -> Result<()> {
        let selector = vars.expand(&step.selector);
        let guard = self.web_session.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| EngineError::Other("web_wait: no active session".to_owned()))?;
        session
            .find(&selector, step.timeout_ms)
            .await
            .map_err(|e| EngineError::Other(format!("web_wait: {e}")))?;
        Ok(())
    }

    #[cfg(feature = "web")]
    async fn web_screenshot(&self, step: &WebScreenshotStep, vars: &Variables) -> Result<()> {
        let path = vars.expand(&step.path);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let guard = self.web_session.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| EngineError::Other("web_screenshot: no active session".to_owned()))?;
        session
            .screenshot(&path)
            .await
            .map_err(|e| EngineError::Other(format!("web_screenshot: {e}")))?;
        info!(path, "web_screenshot saved");
        Ok(())
    }

    #[cfg(feature = "web")]
    async fn web_close(&self) -> Result<()> {
        let mut guard = self.web_session.lock().await;
        if let Some(session) = guard.take() {
            session
                .close()
                .await
                .map_err(|e| EngineError::Other(format!("web_close: {e}")))?;
            info!("web_close: browser session closed");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{ScenarioStep, ScreenshotSaveStep};

    #[test]
    fn ocr_match_step_yaml_roundtrip() {
        let yaml = r#"
name: test
steps:
  - ocr_match:
      contains: "ログイン"
      lang: "jpn+eng"
      timeout_ms: 3000
      save_as: result
"#;
        let scenario = crate::Scenario::from_yaml(yaml).expect("parse failed");
        assert_eq!(scenario.steps.len(), 1);
        match &scenario.steps[0] {
            ScenarioStep::OcrMatch(s) => {
                assert_eq!(s.contains.as_deref(), Some("ログイン"));
                assert_eq!(s.lang, "jpn+eng");
                assert_eq!(s.timeout_ms, 3000);
                assert_eq!(s.save_as.as_deref(), Some("result"));
            }
            _ => panic!("expected OcrMatch"),
        }
    }

    #[test]
    fn ocr_match_step_defaults() {
        let yaml = r#"
name: test
steps:
  - ocr_match: {}
"#;
        let scenario = crate::Scenario::from_yaml(yaml).expect("parse failed");
        match &scenario.steps[0] {
            ScenarioStep::OcrMatch(s) => {
                assert_eq!(s.lang, "jpn+eng");
                assert_eq!(s.timeout_ms, 5000);
                assert_eq!(s.retry_interval_ms, 200);
                assert!(s.contains.is_none());
                assert!(s.save_as.is_none());
            }
            _ => panic!("expected OcrMatch"),
        }
    }

    #[test]
    fn export_rows_csv_roundtrip() {
        let mut vars = Variables::new();
        vars.set(
            "__rows__",
            serde_json::json!([
                {"name": "Alice", "score": "90"},
                {"name": "Bob",   "score": "75"},
            ]),
        );

        let f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        ScenarioEngine::export_rows_csv(&vars, f.path()).unwrap();

        let content = std::fs::read_to_string(f.path()).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("90"));
    }

    #[test]
    fn export_rows_csv_no_rows() {
        let vars = Variables::new();
        let f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        ScenarioEngine::export_rows_csv(&vars, f.path()).unwrap();
        // No __rows__ → file stays at 0 bytes (NamedTempFile creates an empty file)
    }

    #[test]
    fn reconnect_timeout_ms_parsed_from_yaml() {
        let yaml = r#"
name: rdp_scenario
reconnect_timeout_ms: 30000
steps:
  - wait_ms: 1
"#;
        let scenario = crate::Scenario::from_yaml(yaml).expect("parse failed");
        assert_eq!(scenario.reconnect_timeout_ms, Some(30000));
    }

    #[test]
    fn reconnect_timeout_ms_defaults_to_none() {
        let yaml = r#"
name: simple
steps:
  - wait_ms: 1
"#;
        let scenario = crate::Scenario::from_yaml(yaml).expect("parse failed");
        assert_eq!(scenario.reconnect_timeout_ms, None);
    }

    #[test]
    fn parse_cell_ref_works() {
        assert_eq!(parse_cell_ref("A1"), Some((0, 0)));
        assert_eq!(parse_cell_ref("B5"), Some((1, 4)));
        assert_eq!(parse_cell_ref("Z10"), Some((25, 9)));
        assert_eq!(parse_cell_ref("AA1"), Some((26, 0)));
        assert_eq!(parse_cell_ref("bad"), None);
        assert_eq!(parse_cell_ref(""), None);
    }

    // ── csv_read / csv_write hermetic tests ─────────────────────────────────

    use crate::scenario::{CsvReadStep, CsvWriteMode, CsvWriteStep};
    use crate::variables::Variables;

    fn make_engine_for_dir(dir: &std::path::Path) -> ScenarioEngine {
        use rpa_backend::LocalBackend;
        use std::sync::Arc;
        ScenarioEngine::new(Arc::new(LocalBackend::new().unwrap()), dir.to_path_buf())
            .with_silent(true)
    }

    #[test]
    fn csv_read_with_header() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("data.csv");
        std::fs::write(&csv_path, "name,age\nAlice,30\nBob,25\n").unwrap();

        let engine = make_engine_for_dir(dir.path());
        let mut vars = Variables::new();
        let step = CsvReadStep {
            path: "data.csv".into(),
            has_header: true,
            save_as: "rows".into(),
        };
        engine.csv_read(&step, &mut vars).unwrap();

        let rows = vars.get("rows").unwrap();
        let arr = rows.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], serde_json::json!("Alice"));
        assert_eq!(arr[1]["age"], serde_json::json!("25"));
    }

    #[test]
    fn csv_read_no_header() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("raw.csv");
        std::fs::write(&csv_path, "a,b\nc,d\n").unwrap();

        let engine = make_engine_for_dir(dir.path());
        let mut vars = Variables::new();
        let step = CsvReadStep {
            path: "raw.csv".into(),
            has_header: false,
            save_as: "rows".into(),
        };
        engine.csv_read(&step, &mut vars).unwrap();

        let rows = vars.get("rows").unwrap().as_array().unwrap().clone();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], serde_json::json!("a"));
        assert_eq!(rows[1][1], serde_json::json!("d"));
    }

    #[test]
    fn csv_write_and_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let engine = make_engine_for_dir(dir.path());
        let mut vars = Variables::new();

        // Set up list<map> data
        vars.set(
            "rows",
            serde_json::json!([
                {"name": "Alice", "score": "95"},
                {"name": "Bob", "score": "87"},
            ]),
        );

        let write_step = CsvWriteStep {
            path: "out.csv".into(),
            rows: "rows".into(),
            headers: vec![],
            mode: CsvWriteMode::Overwrite,
        };
        engine.csv_write(&write_step, &mut vars).unwrap();

        // Read it back
        let read_step = CsvReadStep {
            path: "out.csv".into(),
            has_header: true,
            save_as: "loaded".into(),
        };
        engine.csv_read(&read_step, &mut vars).unwrap();

        let loaded = vars.get("loaded").unwrap().as_array().unwrap().clone();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0]["name"], serde_json::json!("Alice"));
        assert_eq!(loaded[1]["score"], serde_json::json!("87"));
    }

    #[test]
    #[ignore = "requires a real display with screen content"]
    fn screenshot_save_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let engine = make_engine_for_dir(dir.path());
        let vars = Variables::new();

        let step = ScreenshotSaveStep {
            path: "caps/shot.png".into(),
            window: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(engine.screenshot_save(&step, &vars)).unwrap();
        assert!(dir.path().join("caps/shot.png").exists());
    }

    #[test]
    fn csv_write_append() {
        let dir = tempfile::tempdir().unwrap();
        let engine = make_engine_for_dir(dir.path());
        let mut vars = Variables::new();

        vars.set("r1", serde_json::json!([{"x": "1"}]));
        vars.set("r2", serde_json::json!([{"x": "2"}]));

        engine
            .csv_write(
                &CsvWriteStep {
                    path: "a.csv".into(),
                    rows: "r1".into(),
                    headers: vec![],
                    mode: CsvWriteMode::Overwrite,
                },
                &mut vars,
            )
            .unwrap();
        engine
            .csv_write(
                &CsvWriteStep {
                    path: "a.csv".into(),
                    rows: "r2".into(),
                    headers: vec![],
                    mode: CsvWriteMode::Append,
                },
                &mut vars,
            )
            .unwrap();

        let content = std::fs::read_to_string(dir.path().join("a.csv")).unwrap();
        // Should have header line + 2 data lines
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "1");
        assert_eq!(lines[2], "2");
    }
}
