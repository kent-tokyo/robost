use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use eframe::egui;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu},
    TrayIcon, TrayIconBuilder,
};

// ── Recent-scenario persistence ────────────────────────────────────────────

const MAX_RECENT: usize = 5;

fn recent_path() -> PathBuf {
    crate::config::robost_dir().join("recent.json")
}

fn load_recent() -> Vec<PathBuf> {
    let path = recent_path();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<String>>(&data)
        .unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

fn save_recent(recent: &[PathBuf]) {
    let path = recent_path();
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    let strings: Vec<&str> = recent.iter().filter_map(|p| p.to_str()).collect();
    if let Ok(json) = serde_json::to_string(&strings) {
        let _ = std::fs::write(&path, json);
    }
}

fn push_recent(recent: &mut Vec<PathBuf>, new: PathBuf) {
    recent.retain(|p| p != &new);
    recent.insert(0, new);
    recent.truncate(MAX_RECENT);
    save_recent(recent);
}

// ── Tray icon image ────────────────────────────────────────────────────────

fn make_icon() -> tray_icon::Icon {
    const S: u32 = 16;
    let mut rgba = vec![0u8; (S * S * 4) as usize];
    for chunk in rgba.chunks_mut(4) {
        chunk[0] = 30;
        chunk[1] = 120;
        chunk[2] = 220;
        chunk[3] = 255;
    }
    tray_icon::Icon::from_rgba(rgba, S, S).expect("tray icon")
}

// ── Shared run status ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RunStatus {
    Idle,
    Running { name: String },
    Done { name: String },
    Failed { name: String, error: String },
}

impl RunStatus {
    fn label(&self) -> String {
        match self {
            RunStatus::Idle => "待機中".to_owned(),
            RunStatus::Running { name } => format!("実行中: {name}"),
            RunStatus::Done { name } => format!("完了: {name}"),
            RunStatus::Failed { name, error } => format!("失敗: {name} — {error}"),
        }
    }

    fn is_running(&self) -> bool {
        matches!(self, RunStatus::Running { .. })
    }
}

// Commands sent from eframe thread to the command dispatcher thread.
enum TrayCmd {
    Run(PathBuf),
    Cancel,
    Quit,
}

// ── Scenario runner (one thread per run) ───────────────────────────────────

fn run_scenario_thread(
    path: PathBuf,
    cancel_flag: Arc<AtomicBool>,
    status: Arc<Mutex<RunStatus>>,
    silent: bool,
    reconnect_timeout: u64,
) {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    *status.lock().unwrap() = RunStatus::Running { name: name.clone() };

    // Use a current-thread runtime so the !Send engine works fine.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio rt");

    let result = rt.block_on(async move {
        let backend = Arc::new(
            robost_backend::LocalBackend::new()
                .map_err(|e| robost_core::EngineError::Other(e.to_string()))?,
        );
        let base_dir = path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
        let engine = robost_core::ScenarioEngine::new(backend, base_dir)
            .with_silent(silent)
            .with_reconnect_timeout(reconnect_timeout)
            .with_cancel(cancel_flag);
        let scenario = robost_core::Scenario::from_file(&path)
            .map_err(|e| robost_core::EngineError::Other(e.to_string()))?;
        let mut vars = robost_core::Variables::new();
        engine.run(&scenario, &mut vars).await
    });

    let new_status = match result {
        Ok(()) => RunStatus::Done { name },
        Err(robost_core::EngineError::Cancelled) => RunStatus::Idle,
        Err(e) => RunStatus::Failed {
            name,
            error: e.to_string(),
        },
    };
    *status.lock().unwrap() = new_status;
}

// ── Command dispatcher thread ──────────────────────────────────────────────

fn start_dispatcher(
    initial: Option<PathBuf>,
    silent: bool,
    reconnect_timeout: u64,
    status: Arc<Mutex<RunStatus>>,
    cmd_rx: std::sync::mpsc::Receiver<TrayCmd>,
) {
    std::thread::spawn(move || {
        let mut cancel_flag = Arc::new(AtomicBool::new(false));

        let spawn_run = |path: PathBuf, cf: Arc<AtomicBool>, s: Arc<Mutex<RunStatus>>| {
            std::thread::spawn(move || run_scenario_thread(path, cf, s, silent, reconnect_timeout));
        };

        if let Some(path) = initial {
            let cf = Arc::new(AtomicBool::new(false));
            cancel_flag = Arc::clone(&cf);
            spawn_run(path, cf, Arc::clone(&status));
        }

        for cmd in cmd_rx {
            match cmd {
                TrayCmd::Run(path) => {
                    // Signal the current run to stop, then start a fresh one.
                    cancel_flag.store(true, Ordering::Relaxed);
                    let cf = Arc::new(AtomicBool::new(false));
                    cancel_flag = Arc::clone(&cf);
                    spawn_run(path, cf, Arc::clone(&status));
                }
                TrayCmd::Cancel => {
                    cancel_flag.store(true, Ordering::Relaxed);
                }
                TrayCmd::Quit => {
                    cancel_flag.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    });
}

// ── eframe App ─────────────────────────────────────────────────────────────

struct TrayApp {
    _tray: TrayIcon,
    rerun_id: MenuId,
    open_id: MenuId,
    cancel_id: MenuId,
    quit_id: MenuId,
    recent_ids: Vec<(MenuId, PathBuf)>,
    // Items kept alive to support set_text / set_enabled.
    rerun_item: MenuItem,
    cancel_item: MenuItem,
    status_item: MenuItem,
    shared_status: Arc<Mutex<RunStatus>>,
    last_status: RunStatus,
    last_path: Option<PathBuf>,
    recent: Vec<PathBuf>,
    cmd_tx: std::sync::mpsc::SyncSender<TrayCmd>,
}

impl TrayApp {
    fn new(initial: Option<PathBuf>, silent: bool, reconnect_timeout: u64) -> Self {
        let recent = load_recent();
        let shared_status = Arc::new(Mutex::new(RunStatus::Idle));
        let (cmd_tx, cmd_rx) = std::sync::mpsc::sync_channel::<TrayCmd>(8);

        start_dispatcher(
            initial.clone(),
            silent,
            reconnect_timeout,
            Arc::clone(&shared_status),
            cmd_rx,
        );

        // ── Menu construction ───────────────────────────────────────────────
        let menu = Menu::new();

        let has_initial = initial.is_some();
        let initial_name = initial
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        let rerun_label = if has_initial {
            format!("再実行: {initial_name}")
        } else {
            "再実行".to_owned()
        };
        let rerun_item = MenuItem::new(&rerun_label, has_initial, None);
        let rerun_id = rerun_item.id().clone();

        let open_item = MenuItem::new("別のシナリオを開く...", true, None);
        let open_id = open_item.id().clone();

        // Build recent-scenarios submenu.
        let mut recent_ids: Vec<(MenuId, PathBuf)> = Vec::new();
        let recent_sub = Submenu::new("最近のシナリオ", !recent.is_empty());
        for path in recent.iter().take(MAX_RECENT) {
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            let item = MenuItem::new(&label, true, None);
            let id = item.id().clone();
            recent_sub.append_items(&[&item]).expect("submenu append");
            recent_ids.push((id, path.clone()));
        }

        let cancel_item = MenuItem::new("実行中止", false, None);
        let cancel_id = cancel_item.id().clone();

        let status_item = MenuItem::new("状態: 待機中", false, None);

        let quit_item = MenuItem::new("終了", true, None);
        let quit_id = quit_item.id().clone();

        menu.append_items(&[
            &rerun_item,
            &open_item,
            &recent_sub,
            &PredefinedMenuItem::separator(),
            &cancel_item,
            &PredefinedMenuItem::separator(),
            &status_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .expect("menu append");

        let tray = TrayIconBuilder::new()
            .with_icon(make_icon())
            .with_menu(Box::new(menu))
            .with_tooltip("rpa — シナリオ実行")
            .build()
            .expect("tray build");

        Self {
            _tray: tray,
            rerun_id,
            open_id,
            cancel_id,
            quit_id,
            recent_ids,
            rerun_item,
            cancel_item,
            status_item,
            shared_status,
            last_status: RunStatus::Idle,
            last_path: initial,
            recent,
            cmd_tx,
        }
    }

    fn send(&self, cmd: TrayCmd) {
        let _ = self.cmd_tx.try_send(cmd);
    }

    fn launch(&mut self, path: PathBuf) {
        push_recent(&mut self.recent, path.clone());
        self.last_path = Some(path.clone());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.rerun_item.set_text(format!("再実行: {name}"));
        self.rerun_item.set_enabled(true);
        self.send(TrayCmd::Run(path));
    }

    fn sync_menu_to_status(&mut self, status: &RunStatus) {
        let running = status.is_running();
        self.cancel_item.set_enabled(running);
        self.status_item
            .set_text(format!("状態: {}", status.label()));
        if self.last_path.is_some() {
            self.rerun_item.set_enabled(!running);
        }
    }
}

impl eframe::App for TrayApp {
    // Tray app paints nothing; all work runs in the non-rendering `logic` hook.
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync status from background thread.
        let current = self.shared_status.lock().unwrap().clone();
        if current != self.last_status {
            self.sync_menu_to_status(&current);
            self.last_status = current;
        }

        // Drain all pending menu events.
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_id {
                self.send(TrayCmd::Quit);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if event.id == self.cancel_id {
                self.send(TrayCmd::Cancel);
            } else if event.id == self.rerun_id {
                if let Some(path) = self.last_path.clone() {
                    self.launch(path);
                }
            } else if event.id == self.open_id {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("YAML", &["yaml", "yml"])
                    .pick_file()
                {
                    self.launch(path);
                }
            } else {
                for (id, path) in self.recent_ids.clone() {
                    if event.id == id {
                        self.launch(path);
                        break;
                    }
                }
            }
        }

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

// ── Public entry point ─────────────────────────────────────────────────────

pub fn run_tray(scenario: PathBuf, silent: bool, reconnect_timeout: u64) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_visible(false)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "rpa",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(TrayApp::new(
                Some(scenario),
                silent,
                reconnect_timeout,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("tray error: {e}"))?;

    Ok(())
}
