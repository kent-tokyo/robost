use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// RFC 3339 datetime for one-shot execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    pub scenario: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_status: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ScheduleStore {
    #[serde(default)]
    schedules: Vec<Schedule>,
}

// ── Paths ─────────────────────────────────────────────────────────────────────

pub fn schedules_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".rpa").join("schedules.json")
}

fn logs_base(schedules_path: &Path) -> PathBuf {
    schedules_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("logs")
}

// ── Storage ───────────────────────────────────────────────────────────────────

pub fn load_schedules(path: &Path) -> Result<Vec<Schedule>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let store: ScheduleStore = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    Ok(store.schedules)
}

pub fn save_schedules(path: &Path, schedules: &[Schedule]) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let store = ScheduleStore {
        schedules: schedules.to_vec(),
    };
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&store)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn make_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{:012x}{:04x}", t.as_nanos() & 0xffffffffffff, seq & 0xffff)
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn resolve_schedule_idx(ss: &[Schedule], id: &str) -> Result<usize> {
    if let Some(i) = ss.iter().position(|s| s.id == id) {
        return Ok(i);
    }
    let matches: Vec<usize> = ss
        .iter()
        .enumerate()
        .filter(|(_, s)| s.name == id)
        .map(|(i, _)| i)
        .collect();
    match matches.len() {
        0 => anyhow::bail!("no schedule found: {id}"),
        1 => Ok(matches[0]),
        n => anyhow::bail!(
            "ambiguous: {n} schedules named '{id}' — use the ID instead (see `rpa schedule list`)"
        ),
    }
}

pub fn cmd_add(
    name: String,
    cron: Option<String>,
    at: Option<String>,
    scenario: PathBuf,
    extra_args: Vec<String>,
) -> Result<()> {
    if cron.is_none() && at.is_none() {
        anyhow::bail!("Either --cron or --at is required");
    }
    if cron.is_some() && at.is_some() {
        anyhow::bail!("--cron and --at are mutually exclusive");
    }

    if let Some(ref expr) = cron {
        croner::Cron::new(expr)
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid cron expression '{expr}': {e}"))?;
    }
    if let Some(ref ts) = at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("invalid datetime '{ts}' (expected RFC 3339): {e}"))?;
    }

    let path = schedules_path();
    let mut ss = load_schedules(&path)?;
    let id = make_id();
    let scenario_abs = scenario.canonicalize().unwrap_or(scenario);
    ss.push(Schedule {
        id: id.clone(),
        name: name.clone(),
        cron,
        at,
        scenario: scenario_abs,
        extra_args,
        enabled: true,
        last_run: None,
        last_status: None,
    });
    save_schedules(&path, &ss)?;
    println!("Added schedule '{name}' (id: {id})");
    Ok(())
}

pub fn cmd_list() -> Result<()> {
    let ss = load_schedules(&schedules_path())?;
    if ss.is_empty() {
        println!("(no schedules registered)");
        return Ok(());
    }
    println!(
        "{:<10} {:<20} {:<30} {:<8} LAST STATUS",
        "ID", "NAME", "SCHEDULE", "ENABLED"
    );
    println!("{}", "─".repeat(82));
    for s in &ss {
        let sched = s.cron.as_deref().or(s.at.as_deref()).unwrap_or("?");
        let status = s.last_status.as_deref().unwrap_or("─");
        println!(
            "{:<10} {:<20} {:<30} {:<8} {}",
            s.id, s.name, sched, s.enabled, status
        );
    }
    Ok(())
}

pub fn cmd_remove(id: &str) -> Result<()> {
    let path = schedules_path();
    let mut ss = load_schedules(&path)?;
    let idx = resolve_schedule_idx(&ss, id)?;
    ss.remove(idx);
    save_schedules(&path, &ss)?;
    println!("Removed '{id}'");
    Ok(())
}

pub fn cmd_enable(id: &str, enable: bool) -> Result<()> {
    let path = schedules_path();
    let mut ss = load_schedules(&path)?;
    let idx = resolve_schedule_idx(&ss, id)?;
    ss[idx].enabled = enable;
    let verb = if enable { "Enabled" } else { "Disabled" };
    let name = ss[idx].name.clone();
    save_schedules(&path, &ss)?;
    println!("{verb} '{name}'");
    Ok(())
}

// ── Firing logic ──────────────────────────────────────────────────────────────

fn should_fire(s: &Schedule, now: DateTime<Local>) -> bool {
    if let Some(ref expr) = s.cron {
        let Ok(cron) = croner::Cron::new(expr).parse() else {
            eprintln!(
                "[WARN] schedule '{}' has invalid cron '{}': skipped",
                s.name, expr
            );
            return false;
        };
        // Fire if the next cron occurrence in the past 60-second window is ≤ now.
        let window_start = now - chrono::Duration::minutes(1);
        let Some(t) = cron.iter_from(window_start).next() else {
            return false;
        };
        if t > now {
            return false;
        }
        // Prevent double-fire on daemon restart: skip if we already ran after this occurrence.
        if let Some(ref lr) = s.last_run {
            if let Ok(last) = chrono::DateTime::parse_from_rfc3339(lr) {
                if last.with_timezone(&Local) >= t {
                    return false;
                }
            }
        }
        true
    } else if let Some(ref at_str) = s.at {
        s.last_run.is_none()
            && chrono::DateTime::parse_from_rfc3339(at_str)
                .map(|t| t.with_timezone(&Local) <= now)
                .unwrap_or(false)
    } else {
        false
    }
}

// ── Daemon ────────────────────────────────────────────────────────────────────

async fn sleep_until_next_minute() {
    let secs_past = (chrono::Local::now().timestamp().rem_euclid(60)) as u64;
    let wait = std::time::Duration::from_secs(60u64.saturating_sub(secs_past).max(1));
    tokio::time::sleep(wait).await;
}

async fn fire_due(path: &Path) {
    let now = Local::now();
    let Ok(mut ss) = load_schedules(path) else {
        return;
    };
    let log_base = logs_base(path);
    let rpa = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("rpa"));

    let write_lock = Arc::new(tokio::sync::Mutex::new(()));
    let mut changed = false;
    for s in &mut ss {
        if !s.enabled || !should_fire(s, now) {
            continue;
        }

        let id = s.id.clone();
        let name = s.name.clone();
        let scenario = s.scenario.clone();
        let args = s.extra_args.clone();
        let log_dir = log_base.join(&id);
        let rpa2 = rpa.clone();
        let spath = path.to_owned();

        s.last_run = Some(now.to_rfc3339());
        if s.at.is_some() {
            s.enabled = false; // one-shot: disable after first fire
        }
        changed = true;

        println!(
            "[{}] Firing '{name}' → {}",
            now.format("%H:%M:%S"),
            scenario.display()
        );

        let write_lock2 = Arc::clone(&write_lock);
        tokio::spawn(async move {
            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let logf = log_dir.join(format!("{ts}.log"));
            let _ = std::fs::create_dir_all(&log_dir);

            let mut cmd = tokio::process::Command::new(&rpa2);
            cmd.arg("run").arg(&scenario);
            for a in &args {
                cmd.arg(a);
            }

            if let Ok(f) = std::fs::File::create(&logf) {
                if let Ok(f2) = f.try_clone() {
                    cmd.stdout(f).stderr(f2);
                }
            }

            let result = match cmd.status().await {
                Ok(st) if st.success() => {
                    println!(
                        "[{}] '{name}' OK → log: {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        logf.display()
                    );
                    "ok".to_string()
                }
                Ok(st) => format!("error: exit {}", st.code().unwrap_or(-1)),
                Err(e) => format!("error: {e}"),
            };

            let _guard = write_lock2.lock().await;
            if let Ok(mut ss2) = load_schedules(&spath) {
                if let Some(s2) = ss2.iter_mut().find(|s2| s2.id == id) {
                    s2.last_status = Some(result);
                }
                let _ = save_schedules(&spath, &ss2);
            }
        });
    }

    if changed {
        let _ = save_schedules(path, &ss);
    }
}

pub async fn run_daemon() -> Result<()> {
    let path = schedules_path();
    println!("rpa daemon — schedules: {}", path.display());
    println!("Press Ctrl+C to stop.\n");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nDaemon stopped.");
                return Ok(());
            }
            _ = sleep_until_next_minute() => {
                fire_due(&path).await;
            }
        }
    }
}
