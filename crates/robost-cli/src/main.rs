mod config;
mod scheduler;
mod server;
mod tray;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::io::Read as _;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "rpa", about = "RPA automation tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a scenario YAML file
    Run {
        scenario: PathBuf,
        /// Exit with non-zero status code on error
        #[arg(long, short = 'c')]
        exit_code: bool,
        /// Explicitly exit process when done (useful in non-tray mode)
        #[arg(long, short = 'e')]
        exit: bool,
        /// Start execution at this step index (0-based)
        #[arg(long, default_value = "0")]
        from: usize,
        /// Run only the given step range, e.g. "2..5" (0-based, exclusive end)
        #[arg(long)]
        steps: Option<String>,
        /// Override the scenario's data_source file
        #[arg(long, short = 'd')]
        data: Option<PathBuf>,
        /// Export __rows__ after run (.csv or .xlsx by extension)
        #[arg(long, short = 'x')]
        export: Option<PathBuf>,
        /// Suppress dialog nodes (auto-answer with defaults)
        #[arg(long, short = 's')]
        silent: bool,
        /// Wait this many ms before starting
        #[arg(long, short = 'w', default_value = "0")]
        wait_ms: u64,
        /// Milliseconds to wait for target window to reappear after RDP/VNC reconnect (0 = disabled)
        #[arg(long, default_value = "0")]
        reconnect_timeout: u64,
        /// Stay resident in the system tray after the scenario completes (--exit and --exit-code are ignored)
        #[arg(long)]
        tray: bool,
        /// Pause before every step and wait for Enter (step-through debugger)
        #[arg(long)]
        step: bool,
        /// Skip all actual input operations (clicks, typing, key presses)
        #[arg(long)]
        dry_run: bool,
        /// Pause at this specific step index (0-based)
        #[arg(long)]
        break_at: Option<usize>,
        /// Print all variables to stderr after each step
        #[arg(long)]
        dump_vars: bool,
        /// Write an execution report to this path (.html or .csv detected by extension)
        #[arg(long)]
        report: Option<PathBuf>,
        /// Write step progress JSON {"step":N,"name":"..."} to this file during execution
        #[arg(long)]
        progress: Option<PathBuf>,
        /// Start HTTP server and stream progress via SSE (e.g., "127.0.0.1:0" for dynamic port)
        #[arg(long)]
        serve: Option<String>,
        /// Write per-step JSONL trace to this path (one JSON line per step)
        #[arg(long)]
        trace: Option<PathBuf>,
        /// Save screenshots: never (default) | on-failure | always
        #[arg(long, value_name = "MODE", default_value = "never")]
        screenshots: String,
    },
    /// Plugin management
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },
    /// Schedule management
    Schedule {
        #[command(subcommand)]
        action: ScheduleCommands,
    },
    /// Run the scheduler daemon (blocks until Ctrl+C)
    Daemon,
    /// Check that all robost subsystems are working correctly
    Doctor,
    /// Benchmark a template image against a directory of screenshots
    VisionBench {
        /// Path to the template PNG
        template: PathBuf,
        /// Directory containing screenshot images to test against
        screenshots: PathBuf,
    },
    /// Inspect UIA elements in a window or at a screen point (Windows only)
    Inspect {
        /// Window title substring — lists all UIA elements in that window
        #[arg(long)]
        window: Option<String>,
        /// Screen point — prints the element at x y (two values)
        #[arg(long, num_args = 2, value_names = ["X", "Y"])]
        point: Option<Vec<i32>>,
        /// Wait N seconds then capture element under cursor (default: 5)
        #[arg(long, value_name = "SECS")]
        hover: Option<u64>,
    },
    /// Start the local RPA agent with a built-in HTTP server and web UI
    Agent {
        /// Port to listen on
        #[arg(long, short = 'p', default_value = "9921")]
        port: u16,
        /// Scenarios directory (default: ~/robost/scenarios)
        #[arg(long)]
        scenarios_dir: Option<std::path::PathBuf>,
        /// Do not open browser automatically
        #[arg(long)]
        no_browser: bool,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Install a .wasm plugin from a local path, HTTPS URL, or owner/repo@tag
    Install {
        /// Local .wasm path, https://…/plugin.wasm URL, or owner/repo@tag
        source: String,
        /// Skip interactive permission confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// List installed plugins
    List,
}

#[derive(Subcommand)]
enum ScheduleCommands {
    /// Register a new schedule
    Add {
        /// Human-readable name
        #[arg(long)]
        name: String,
        /// Cron expression (5-field: "minute hour day month weekday")
        #[arg(long, conflicts_with = "at")]
        cron: Option<String>,
        /// One-shot datetime in RFC 3339 format, e.g. "2025-06-01T09:00:00+09:00"
        #[arg(long, conflicts_with = "cron")]
        at: Option<String>,
        /// Scenario YAML file to run
        scenario: PathBuf,
        /// Extra arguments passed to `rpa run`
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },
    /// List registered schedules
    List,
    /// Remove a schedule by id or name
    Remove { id: String },
    /// Enable a disabled schedule
    Enable { id: String },
    /// Disable a schedule without removing it
    Disable { id: String },
}

/// Parse "start..end" or "start.." into (from, to).
fn parse_steps_range(s: &str) -> Result<(usize, Option<usize>)> {
    match s.split_once("..") {
        Some((start, end)) => {
            let from = start.parse::<usize>()?;
            let to = if end.is_empty() {
                None
            } else {
                Some(end.parse::<usize>()?)
            };
            Ok((from, to))
        }
        None => bail!("--steps expects 'start..end' format, e.g. '2..5'"),
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    robost_capture::init_dpi();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Agent {
        port: 9921,
        scenarios_dir: None,
        no_browser: false,
    }) {
        Commands::Run {
            scenario,
            wait_ms,
            silent,
            from,
            steps,
            data,
            export,
            exit_code,
            exit,
            reconnect_timeout,
            tray,
            step,
            dry_run,
            break_at,
            dump_vars,
            report,
            progress,
            serve,
            trace,
            screenshots,
        } => {
            if tray {
                return tray::run_tray(scenario, silent, reconnect_timeout);
            }

            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            rt.block_on(async {
                if wait_ms > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                }

                // Start HTTP server if --serve is specified.
                let progress_tx = if let Some(ref bind_addr) = serve {
                    let (tx, _port) = server::run_server(bind_addr)
                        .await
                        .context("failed to start HTTP server")?;
                    Some(tx)
                } else {
                    None
                };

                let backend = Arc::new(robost_backend::LocalBackend::new()?);
                let base_dir = scenario
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf();
                let screenshots_mode = match screenshots.as_str() {
                    "on-failure" => robost_core::ScreenshotsMode::OnFailure,
                    "always" => robost_core::ScreenshotsMode::Always,
                    _ => robost_core::ScreenshotsMode::Never,
                };
                let mut engine = robost_core::ScenarioEngine::new(backend, base_dir)
                    .with_silent(silent)
                    .with_reconnect_timeout(reconnect_timeout)
                    .with_debug_step(step)
                    .with_dry_run(dry_run)
                    .with_break_at(break_at)
                    .with_dump_vars(dump_vars)
                    .with_trace(trace)
                    .with_screenshots_mode(screenshots_mode);
                if let Some(report_path) = report {
                    engine = engine.with_report(report_path);
                }
                engine = engine.with_progress(progress);

                // Set progress channel if server is running.
                if let Some(tx) = progress_tx {
                    engine = engine.with_progress_channel(Some(tx));
                }
                let mut s = robost_core::Scenario::from_file(&scenario)?;

                // Resolve step range: --steps takes priority over --from.
                let (from_step, to_step) = if let Some(ref range_str) = steps {
                    if from != 0 {
                        bail!("--steps and --from cannot both be specified");
                    }
                    parse_steps_range(range_str)?
                } else {
                    (from, None)
                };

                // Trim scenario steps for --steps to-bound.
                if let Some(to) = to_step {
                    s.steps.truncate(to);
                }

                let mut vars = robost_core::Variables::new();
                let result = engine
                    .run_with_opts(&s, &mut vars, from_step, data.as_deref())
                    .await;

                if let Some(export_path) = export {
                    let ext = export_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if ext.eq_ignore_ascii_case("xlsx") {
                        robost_core::ScenarioEngine::export_rows_xlsx(&vars, &export_path)?;
                    } else {
                        robost_core::ScenarioEngine::export_rows_csv(&vars, &export_path)?;
                    }
                    println!("exported __rows__ to {}", export_path.display());
                }

                match result {
                    Ok(()) => {
                        if exit {
                            std::process::exit(0);
                        }
                    }
                    Err(e) => {
                        eprintln!("error: {e:#}");
                        if exit_code {
                            std::process::exit(1);
                        }
                    }
                }

                Ok(())
            })
        }
        Commands::Schedule { action } => match action {
            ScheduleCommands::Add {
                name,
                cron,
                at,
                scenario,
                extra_args,
            } => scheduler::cmd_add(name, cron, at, scenario, extra_args),
            ScheduleCommands::List => scheduler::cmd_list(),
            ScheduleCommands::Remove { id } => scheduler::cmd_remove(&id),
            ScheduleCommands::Enable { id } => scheduler::cmd_enable(&id, true),
            ScheduleCommands::Disable { id } => scheduler::cmd_enable(&id, false),
        },
        Commands::Daemon => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(scheduler::run_daemon())
        }
        Commands::Doctor => run_doctor(),
        Commands::VisionBench {
            template,
            screenshots,
        } => run_vision_bench(&template, &screenshots),
        Commands::Agent {
            port,
            scenarios_dir,
            no_browser,
        } => {
            let dir = scenarios_dir.unwrap_or_else(|| {
                #[cfg(target_os = "windows")]
                let docs = std::env::var("USERPROFILE")
                    .ok()
                    .map(|h| std::path::PathBuf::from(h).join("Documents"));
                #[cfg(not(target_os = "windows"))]
                let docs = std::env::var("HOME")
                    .ok()
                    .map(|h| std::path::PathBuf::from(h).join("Documents"));
                docs.unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("robost")
                    .join("scenarios")
            });
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create scenarios dir: {}", dir.display()))?;

            let bind_addr = format!("127.0.0.1:{port}");
            if !no_browser {
                let url = format!("http://localhost:{port}");
                // Open browser after a short delay so the server can start
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    #[cfg(target_os = "windows")]
                    let _ = std::process::Command::new("explorer.exe").arg(&url).spawn();
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open").arg(&url).spawn();
                    #[cfg(target_os = "linux")]
                    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                });
            }

            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            rt.block_on(server::run_agent_server(&bind_addr, dir))
        }
        Commands::Inspect {
            window,
            point,
            hover,
        } => run_inspect(window, point, hover),
        Commands::Plugin { action } => match action {
            PluginCommands::Install { source, yes } => {
                let (_tmpdir, wasm_path) = resolve_plugin_source(&source)?;
                let plugin = robost_plugin_host::PluginInstance::load(&wasm_path)?;
                let m = &plugin.manifest;

                if !yes {
                    println!("Plugin: {} v{}", m.plugin.name, m.plugin.version);
                    if let Some(desc) = &m.plugin.description {
                        println!("Description: {desc}");
                    }
                    println!(
                        "Functions: {}",
                        m.function
                            .iter()
                            .map(|f| f.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    println!("Permissions:");
                    if m.permissions.filesystem.is_empty() {
                        println!("  filesystem : (none)");
                    } else {
                        println!("  filesystem : {:?}", m.permissions.filesystem);
                    }
                    println!("  network    : {}", m.permissions.network);
                    println!("  screen     : {}", m.permissions.screen);
                    print!("Allow this plugin? [y/N] ");
                    use std::io::{BufRead, Write};
                    std::io::stdout().flush()?;
                    let mut buf = String::new();
                    std::io::BufReader::new(std::io::stdin()).read_line(&mut buf)?;
                    if !matches!(buf.trim(), "y" | "Y" | "yes") {
                        println!("cancelled.");
                        return Ok(());
                    }
                }

                let dest = config::install_plugin(&wasm_path, m)?;
                println!(
                    "installed: {} v{} → {}",
                    m.plugin.name,
                    m.plugin.version,
                    dest.display()
                );
                Ok(())
            }
            PluginCommands::List => {
                let entries = config::load_registry();
                if entries.is_empty() {
                    println!("(no plugins installed)");
                } else {
                    println!("{:<24} {:<12} PATH", "NAME", "VERSION");
                    for e in &entries {
                        println!("{:<24} {:<12} {}", e.name, e.version, e.wasm_path.display());
                    }
                }
                Ok(())
            }
        },
    }
}

// ── rpa inspect ──────────────────────────────────────────────────────────────

fn print_element_info(info: &robost_uia::ElementInfo) {
    println!("- name: {:?}", info.name);
    println!("  automation_id: {:?}", info.automation_id);
    println!("  class_name: {:?}", info.class_name);
    println!("  control_type: {:?}", info.control_type);
    let (x, y, w, h) = info.rect;
    println!("  rect: [{x}, {y}, {w}, {h}]");
    println!("  enabled: {}", info.enabled);
    if !info.automation_id.is_empty() {
        println!("  # suggested: by: {{ id: {:?} }}", info.automation_id);
    } else if !info.name.is_empty() {
        println!("  # suggested: by: {{ name: {:?} }}", info.name);
    } else if !info.class_name.is_empty() {
        println!("  # suggested: by: {{ class: {:?} }}", info.class_name);
    }
}

fn run_inspect(window: Option<String>, point: Option<Vec<i32>>, hover: Option<u64>) -> Result<()> {
    // --hover: wait N seconds then capture element under cursor
    if let Some(secs) = hover {
        let secs = if secs == 0 { 5 } else { secs };
        eprintln!("カーソルを要素に合わせてください... ({secs} 秒後に取得)");
        std::thread::sleep(std::time::Duration::from_secs(secs));
        let (x, y) = robost_uia::UiaFinder::cursor_pos().context("cursor_pos: Windows only")?;
        let finder = robost_uia::UiaFinder::new().context("UIA init failed — Windows only")?;
        let el = finder
            .element_at_point(x, y)
            .context("no element at cursor position")?;
        print_element_info(&finder.element_info(&el));
        return Ok(());
    }

    let finder = robost_uia::UiaFinder::new().context("UIA init failed — Windows only")?;

    let elements: Vec<robost_uia::ElementInfo> = if let Some(coords) = point {
        let el = finder
            .element_at_point(coords[0], coords[1])
            .context("no element at that point")?;
        vec![finder.element_info(&el)]
    } else if let Some(ref title) = window {
        let win_el = finder
            .find_window_element(title)
            .with_context(|| format!("window not found: {title}"))?;
        finder
            .list_descendants(&win_el)
            .context("failed to enumerate descendants")?
    } else {
        anyhow::bail!("specify --window <TITLE>, --point <X> <Y>, or --hover [SECS]");
    };

    for info in &elements {
        print_element_info(info);
    }
    Ok(())
}

// ── Plugin source resolution ──────────────────────────────────────────────────

/// Returns a (optional temp-dir guard, wasm path) pair.
/// The temp dir is kept alive by the caller; dropping it removes the temp files.
fn resolve_plugin_source(source: &str) -> Result<(Option<tempfile::TempDir>, PathBuf)> {
    if source.starts_with("https://") || source.starts_with("http://") {
        let (dir, path) = download_wasm_url(source)?;
        Ok((Some(dir), path))
    } else if let Some((repo, tag)) = source.split_once('@') {
        if repo.contains('/') {
            let (dir, path) = download_github_release(repo, tag)?;
            Ok((Some(dir), path))
        } else {
            bail!(
                "invalid source '{}': expected a file path, https:// URL, or owner/repo@tag",
                source
            )
        }
    } else {
        Ok((None, PathBuf::from(source)))
    }
}

/// Download a .wasm file (and adjacent plugin.toml) from an HTTPS URL into a temp dir.
fn download_wasm_url(wasm_url: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let dir = tempfile::tempdir()?;
    let filename = wasm_url.rsplit('/').next().unwrap_or("plugin.wasm");
    let wasm_dest = dir.path().join(filename);

    let toml_base = wasm_url
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or(wasm_url);
    let toml_url = format!("{toml_base}/plugin.toml");

    eprintln!("Downloading {wasm_url}");
    fetch_to_file(wasm_url, &wasm_dest)?;
    eprintln!("Downloading {toml_url}");
    fetch_to_file(&toml_url, &dir.path().join("plugin.toml"))?;

    Ok((dir, wasm_dest))
}

/// Resolve `owner/repo@tag` via the GitHub Releases API, then download the .wasm asset.
fn download_github_release(repo: &str, tag: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let api_url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    eprintln!("Fetching release metadata: {api_url}");

    let body: serde_json::Value = ureq::get(&api_url)
        .set("User-Agent", "robost-cli")
        .set("Accept", "application/vnd.github+json")
        .call()
        .context("GitHub API request failed")?
        .into_json()
        .context("failed to parse GitHub API response")?;

    let assets = body["assets"]
        .as_array()
        .context("no 'assets' field in GitHub release")?;

    let wasm_url = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.ends_with(".wasm"))
                .unwrap_or(false)
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .context("no .wasm asset found in GitHub release")?;

    download_wasm_url(wasm_url)
}

fn fetch_to_file(url: &str, dest: &std::path::Path) -> Result<()> {
    let resp = ureq::get(url)
        .call()
        .with_context(|| format!("GET {url} failed"))?;
    let mut buf = Vec::new();
    resp.into_reader().read_to_end(&mut buf)?;
    std::fs::write(dest, buf).with_context(|| format!("write to {} failed", dest.display()))
}

// ── rpa doctor ───────────────────────────────────────────────────────────────

fn run_doctor() -> Result<()> {
    let mut ok = true;

    // 1. Screen capture
    match robost_capture::capture_screen() {
        Ok(img) => println!(
            "[OK]   screen capture     ({}x{})",
            img.width(),
            img.height()
        ),
        Err(e) => {
            println!("[FAIL] screen capture     ({e})");
            ok = false;
        }
    }

    // 2. Input emulation
    match robost_input::InputController::new() {
        Ok(_) => println!("[OK]   input emulation    (enigo initialized)"),
        Err(e) => {
            println!("[FAIL] input emulation    ({e})");
            ok = false;
        }
    }

    // 3. OCR backend (compile-time feature detection)
    #[cfg(feature = "windows-ocr")]
    println!("[OK]   OCR backend        (windows-ocr built-in)");
    #[cfg(all(feature = "ocr", not(feature = "windows-ocr")))]
    println!("[OK]   OCR backend        (Tesseract / leptess)");
    #[cfg(not(any(feature = "ocr", feature = "windows-ocr")))]
    println!("[WARN] OCR backend        (no OCR feature enabled; rebuild with --features ocr)");

    // ocrs-cjk OCR backend
    #[cfg(feature = "ocrs-cjk-ocr")]
    {
        match std::env::var("ROBOST_OCR_MODEL_DIR") {
            Ok(dir) => {
                let has_det = ["detection.rten", "detection.onnx"]
                    .iter()
                    .any(|f| std::path::Path::new(&dir).join(f).exists());
                let has_rec = ["recognition.rten", "recognition.onnx"]
                    .iter()
                    .any(|f| std::path::Path::new(&dir).join(f).exists());
                if has_det && has_rec {
                    println!("[OK]   ocrs-cjk OCR       (models found: {dir})");
                } else {
                    println!("[WARN] ocrs-cjk OCR       (models missing in {dir}: need detection.* + recognition.*)");
                }
            }
            Err(_) => {
                // Check default location
                let default_dir = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .map(|h| format!("{h}/.robost/models"))
                    .unwrap_or_default();
                let has_default = std::path::Path::new(&default_dir)
                    .join("detection.rten")
                    .exists()
                    || std::path::Path::new(&default_dir)
                        .join("detection.onnx")
                        .exists();
                if has_default {
                    println!("[OK]   ocrs-cjk OCR       (models found: {default_dir})");
                } else {
                    println!("[WARN] ocrs-cjk OCR       (set ROBOST_OCR_MODEL_DIR or place models in ~/.robost/models)");
                }
            }
        }
    }
    #[cfg(not(feature = "ocrs-cjk-ocr"))]
    println!("[SKIP] ocrs-cjk OCR       (rebuild with --features ocrs-cjk-ocr to enable)");

    // 4. Keychain access
    #[cfg(feature = "keychain")]
    {
        let entry = keyring::Entry::new("robost-doctor", "test")
            .context("keychain: failed to create entry")?;
        match entry.set_password("robost-doctor-probe") {
            Ok(_) => {
                let _ = entry.delete_credential();
                println!("[OK]   keychain access    (read/write successful)");
            }
            Err(e) => {
                println!("[WARN] keychain access    ({e})");
            }
        }
    }
    #[cfg(not(feature = "keychain"))]
    println!("[SKIP] keychain access    (keychain feature not enabled)");

    // 5. UIA availability
    #[cfg(target_os = "windows")]
    match robost_uia::UiaFinder::new() {
        Ok(_) => println!("[OK]   UIA automation    (IUIAutomation available)"),
        Err(e) => {
            println!("[FAIL] UIA automation    ({e})");
            ok = false;
        }
    }
    #[cfg(not(target_os = "windows"))]
    println!("[SKIP] UIA automation    (Windows only)");

    // 6. Admin / elevated app detection
    #[cfg(target_os = "windows")]
    {
        if check_is_admin() {
            println!(
                "[OK]   Admin mode        (running as administrator — can automate elevated apps)"
            );
        } else {
            println!("[WARN] Admin mode        (not admin — elevated apps cannot be automated; re-run as administrator if needed)");
        }
    }
    #[cfg(not(target_os = "windows"))]
    println!("[SKIP] Admin mode        (Windows only)");

    // 7. DPI awareness
    println!("[OK]   DPI awareness     (SetProcessDpiAwareness called at startup)");

    if ok {
        println!("\nAll checks passed.");
    } else {
        println!("\nOne or more checks failed.");
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn check_is_admin() -> bool {
    use windows::Win32::UI::Shell::IsUserAnAdmin;
    unsafe { IsUserAnAdmin().as_bool() }
}

// ── rpa vision-bench ─────────────────────────────────────────────────────────

fn run_vision_bench(
    template_path: &std::path::Path,
    screenshots_dir: &std::path::Path,
) -> Result<()> {
    use robost_template::ScreenPoint;
    use robost_vision::TemplateMatcher;

    let template_img = image::open(template_path)
        .with_context(|| format!("failed to load template: {}", template_path.display()))?
        .to_rgba8();

    let entries: Vec<_> = std::fs::read_dir(screenshots_dir)
        .with_context(|| format!("cannot read directory: {}", screenshots_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| matches!(x.to_lowercase().as_str(), "png" | "jpg" | "jpeg"))
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        println!("No PNG/JPG files found in {}", screenshots_dir.display());
        return Ok(());
    }

    // Check for too many matches (template too generic)
    let matcher = TemplateMatcher::default();
    let origin = ScreenPoint { x: 0, y: 0 };

    println!("{:<40} {:>10}  FOUND", "FILE", "CONFIDENCE");
    println!("{}", "-".repeat(60));

    let mut scores: Vec<f32> = Vec::new();
    let mut generic_warn = false;

    for entry in &entries {
        let path = entry.path();
        let hay = match image::open(&path) {
            Ok(i) => i.to_rgba8(),
            Err(e) => {
                println!("{:<40}  load error: {e}", path.display().to_string());
                continue;
            }
        };

        let display = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match matcher.find(&hay, &template_img, origin) {
            Ok(m) => {
                scores.push(m.score);
                // Check for too-generic template using find_all
                let all = matcher.find_all(&hay, &template_img, origin);
                if all.len() > 3 {
                    generic_warn = true;
                }
                println!("{:<40} {:>10.3}  yes", display, m.score);
            }
            Err(robost_vision::MatchError::BelowThreshold { score, .. }) => {
                scores.push(score);
                println!("{:<40} {:>10.3}  no   ← LOW", display, score);
            }
            Err(e) => {
                println!("{:<40}  error: {e}", display);
            }
        }
    }

    if !scores.is_empty() {
        let avg = scores.iter().sum::<f32>() / scores.len() as f32;
        println!("\nAverage confidence: {avg:.3}");
        if avg < 0.80 {
            println!("[WARN] Low average confidence — consider capturing a cleaner template.");
        }
    }
    if generic_warn {
        println!(
            "[WARN] Template matched >3 regions in at least one screenshot — may be too generic."
        );
    }

    Ok(())
}
