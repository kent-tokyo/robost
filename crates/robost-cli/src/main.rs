mod config;
mod scheduler;
mod tray;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "rpa", about = "RPA automation tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Install a .wasm plugin
    Install {
        path: PathBuf,
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

    match cli.command {
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
        } => {
            if tray {
                return tray::run_tray(scenario, silent, reconnect_timeout);
            }

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                if wait_ms > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                }

                let backend = Arc::new(robost_backend::LocalBackend::new()?);
                let base_dir = scenario
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf();
                let mut engine = robost_core::ScenarioEngine::new(backend, base_dir)
                    .with_silent(silent)
                    .with_reconnect_timeout(reconnect_timeout)
                    .with_debug_step(step)
                    .with_dry_run(dry_run)
                    .with_break_at(break_at)
                    .with_dump_vars(dump_vars);
                if let Some(report_path) = report {
                    engine = engine.with_report(report_path);
                }
                engine = engine.with_progress(progress);
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
        Commands::Plugin { action } => match action {
            PluginCommands::Install { path, yes } => {
                let plugin = robost_plugin_host::PluginInstance::load(&path)?;
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

                let dest = config::install_plugin(&path, m)?;
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
