use anyhow::{Context, Result};
use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{sse::Event, IntoResponse, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use futures::stream::StreamExt;
use robost_core::ProgressEvent;
use serde::{Deserialize, Serialize};
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;

// ── Shared state for the run-only server ────────────────────────────────────

#[derive(Clone)]
pub struct ServerState {
    progress_broadcast: Arc<broadcast::Sender<ProgressEvent>>,
}

// ── Agent state (includes all run-only state plus file/execution management) ─

// ScenarioEngine's future is !Send, so we can't use tokio::spawn.
// Instead we run the engine on a std::thread with its own single-threaded runtime
// and communicate cancellation via a oneshot channel.
#[derive(Clone)]
pub struct AgentState {
    pub progress_tx: Arc<broadcast::Sender<ProgressEvent>>,
    pub abort_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub is_running: Arc<Mutex<bool>>,
    pub current_scenario: Arc<Mutex<Option<String>>>,
    pub scenarios_dir: PathBuf,
}

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Serialize)]
struct FolderEntry {
    name: String,
    scenarios: Vec<String>,
}

#[derive(Serialize)]
struct ScenarioList {
    scenarios: Vec<String>,
    folders: Vec<FolderEntry>,
}

#[derive(Deserialize)]
struct CreateFolderBody {
    name: String,
}

#[derive(Serialize)]
struct ScenarioContent {
    name: String,
    content: String,
}

#[derive(Deserialize)]
struct SaveBody {
    content: String,
}

#[derive(Deserialize)]
struct RunBody {
    scenario: String,
    #[serde(default)]
    from: usize,
    /// If set, stop after this step index (inclusive). Used for single-step runs.
    #[serde(default)]
    to: Option<usize>,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
    scenario: Option<String>,
}

#[derive(Serialize)]
struct OkResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatBody {
    messages: Vec<ChatMessage>,
    #[serde(default)]
    scenario_yaml: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

// ── Path traversal guard ──────────────────────────────────────────────────────

/// Returns the joined path for a scenario name like `file.yaml` or `folder/file.yaml`.
/// Rejects path traversal, backslashes, hidden files/dirs, and more than one level of nesting.
fn safe_scenario_path(scenarios_dir: &FsPath, name: &str) -> Option<PathBuf> {
    if name.is_empty() || name.contains('\\') {
        return None;
    }
    let parts: Vec<&str> = name.splitn(3, '/').collect();
    if parts.len() > 2 {
        return None; // deeper than one folder level
    }
    for part in &parts {
        if part.is_empty() || part.starts_with('.') || part.contains("..") {
            return None;
        }
    }
    match parts.as_slice() {
        [file] => Some(scenarios_dir.join(file)),
        [folder, file] => Some(scenarios_dir.join(folder).join(file)),
        _ => None,
    }
}

/// Validate a bare folder name (no slashes).
fn safe_folder_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains("..")
        && !name.starts_with('.')
}

// ── Run-only server handlers ─────────────────────────────────────────────────

async fn handle_events(
    State(state): State<ServerState>,
) -> Sse<impl futures::stream::Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.progress_broadcast.subscribe();
    let stream = BroadcastStream::new(rx).map(|result| {
        let json = match result {
            Ok(event) => serde_json::to_string(&event).unwrap_or_default(),
            Err(_) => "{}".to_string(),
        };
        Ok(Event::default().data(json))
    });
    Sse::new(stream)
}

async fn handle_health() -> &'static str {
    "OK"
}

// ── Agent server handlers ────────────────────────────────────────────────────

async fn agent_events(
    State(state): State<AgentState>,
) -> Sse<impl futures::stream::Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.progress_tx.subscribe();
    let stream = BroadcastStream::new(rx).map(|result| {
        let json = match result {
            Ok(event) => serde_json::to_string(&event).unwrap_or_default(),
            Err(_) => "{}".to_string(),
        };
        Ok(Event::default().data(json))
    });
    Sse::new(stream)
}

// Fix: capture_screen is blocking — run on a dedicated thread pool thread.
async fn agent_screenshot() -> impl IntoResponse {
    let result: Result<Result<Vec<u8>, String>, _> = tokio::task::spawn_blocking(|| {
        let img = robost_capture::capture_screen().map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(buf)
    })
    .await;

    let inner = match result {
        Ok(r) => r,
        Err(e) => Err(e.to_string()),
    };
    match inner {
        Ok(buf) => (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], buf),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain")],
            e.into_bytes(),
        ),
    }
}

async fn agent_list_scenarios(State(state): State<AgentState>) -> impl IntoResponse {
    let dir = state.scenarios_dir.clone();
    let result = tokio::task::spawn_blocking(move || -> std::io::Result<ScenarioList> {
        let mut top_scenarios: Vec<String> = Vec::new();
        let mut folders: Vec<FolderEntry> = Vec::new();

        for entry in std::fs::read_dir(&dir)?.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            if file_name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                let mut sub: Vec<String> = std::fs::read_dir(&path)
                    .map(|e| {
                        e.flatten()
                            .filter_map(|s| {
                                let n = s.file_name().to_string_lossy().to_string();
                                (n.ends_with(".yaml") || n.ends_with(".yml")).then_some(n)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                sub.sort();
                folders.push(FolderEntry {
                    name: file_name,
                    scenarios: sub,
                });
            } else if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                top_scenarios.push(file_name);
            }
        }

        top_scenarios.sort();
        folders.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(ScenarioList {
            scenarios: top_scenarios,
            folders,
        })
    })
    .await;

    match result {
        Ok(Ok(list)) => Json(list).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

async fn agent_create_folder(
    State(state): State<AgentState>,
    Json(body): Json<CreateFolderBody>,
) -> impl IntoResponse {
    if !safe_folder_name(&body.name) {
        return (
            StatusCode::BAD_REQUEST,
            Json(OkResponse {
                ok: false,
                message: Some("invalid folder name".into()),
            }),
        )
            .into_response();
    }
    let path = state.scenarios_dir.join(&body.name);
    match tokio::fs::create_dir(&path).await {
        Ok(()) => Json(OkResponse {
            ok: true,
            message: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

async fn agent_delete_folder(
    State(state): State<AgentState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if !safe_folder_name(&name) {
        return (
            StatusCode::BAD_REQUEST,
            Json(OkResponse {
                ok: false,
                message: Some("invalid folder name".into()),
            }),
        )
            .into_response();
    }
    let path = state.scenarios_dir.join(&name);
    match tokio::fs::remove_dir(&path).await {
        Ok(()) => Json(OkResponse {
            ok: true,
            message: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

// Fix: safe_scenario_path guard + tokio::fs.
async fn agent_get_scenario(
    State(state): State<AgentState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let path = match safe_scenario_path(&state.scenarios_dir, &name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OkResponse {
                    ok: false,
                    message: Some("invalid scenario name".into()),
                }),
            )
                .into_response()
        }
    };
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => (StatusCode::OK, Json(ScenarioContent { name, content })).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

// Fix: safe_scenario_path guard + tokio::fs.
async fn agent_save_scenario(
    State(state): State<AgentState>,
    Path(name): Path<String>,
    Json(body): Json<SaveBody>,
) -> impl IntoResponse {
    let path = match safe_scenario_path(&state.scenarios_dir, &name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OkResponse {
                    ok: false,
                    message: Some("invalid scenario name".into()),
                }),
            )
                .into_response()
        }
    };
    match tokio::fs::write(&path, &body.content).await {
        Ok(()) => Json(OkResponse {
            ok: true,
            message: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

// Fix: safe_scenario_path guard + tokio::fs.
async fn agent_delete_scenario(
    State(state): State<AgentState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let path = match safe_scenario_path(&state.scenarios_dir, &name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OkResponse {
                    ok: false,
                    message: Some("invalid scenario name".into()),
                }),
            )
                .into_response()
        }
    };
    match tokio::fs::remove_file(&path).await {
        Ok(()) => Json(OkResponse {
            ok: true,
            message: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

async fn agent_run(
    State(state): State<AgentState>,
    Json(body): Json<RunBody>,
) -> impl IntoResponse {
    let mut is_running = state.is_running.lock().await;
    if *is_running {
        return (
            StatusCode::CONFLICT,
            Json(OkResponse {
                ok: false,
                message: Some("A scenario is already running".into()),
            }),
        )
            .into_response();
    }

    // Fix: guard against path traversal in the JSON body scenario field.
    let scenario_path = match safe_scenario_path(&state.scenarios_dir, &body.scenario) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OkResponse {
                    ok: false,
                    message: Some("invalid scenario name".into()),
                }),
            )
                .into_response()
        }
    };
    if !scenario_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(OkResponse {
                ok: false,
                message: Some(format!("Scenario '{}' not found", body.scenario)),
            }),
        )
            .into_response();
    }

    let tx = state.progress_tx.clone();
    let is_running_flag = state.is_running.clone();
    let current_scenario_slot = state.current_scenario.clone();
    let abort_tx_slot = state.abort_tx.clone();
    let scenario_name = body.scenario.clone();
    let dry_run = body.dry_run;
    let from = body.from;
    let to = body.to;

    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();
    *state.abort_tx.lock().await = Some(abort_tx);

    // ScenarioEngine's future is !Send, so we spawn a std::thread with its own
    // single-threaded Tokio runtime and race against the oneshot abort signal.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build scenario runtime");

        rt.block_on(async move {
            {
                let mut cur = current_scenario_slot.lock().await;
                *cur = Some(scenario_name.clone());
            }

            let engine_fut = async {
                let backend = Arc::new(
                    robost_backend::LocalBackend::new().map_err(|e| anyhow::anyhow!("{e}"))?,
                );
                let base_dir = scenario_path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf();
                let engine = robost_core::ScenarioEngine::new(backend, base_dir)
                    .with_dry_run(dry_run)
                    .with_progress_channel(Some(tx.clone()));
                let mut scenario = robost_core::Scenario::from_file(&scenario_path)?;
                // Limit to a single step when `to` is specified.
                if let Some(end) = to {
                    let end = end.min(scenario.steps.len().saturating_sub(1));
                    scenario.steps = scenario.steps[from..=end].to_vec();
                }
                let effective_from = if to.is_some() { 0 } else { from };
                let mut vars = robost_core::Variables::new();
                engine
                    .run_with_opts(&scenario, &mut vars, effective_from, None)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))
            };

            tokio::select! {
                result = engine_fut => {
                    if let Err(e) = result {
                        let _ = tx.send(ProgressEvent::Finished {
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
                _ = abort_rx => {
                    let _ = tx.send(ProgressEvent::Finished {
                        success: false,
                        error: Some("Stopped by user".into()),
                    });
                }
            }

            *is_running_flag.lock().await = false;
            *current_scenario_slot.lock().await = None;
            *abort_tx_slot.lock().await = None;
        });
    });

    *is_running = true;

    Json(OkResponse {
        ok: true,
        message: None,
    })
    .into_response()
}

async fn agent_stop(State(state): State<AgentState>) -> impl IntoResponse {
    let abort_tx = state.abort_tx.lock().await.take();
    let was_running = abort_tx.is_some();
    if let Some(tx) = abort_tx {
        let _ = tx.send(());
    }
    // Fix: only emit Finished when something was actually running.
    // The spawned thread will also send Finished via the abort_rx path above,
    // so we skip the redundant send here; the thread handles cleanup too.
    if !was_running {
        *state.is_running.lock().await = false;
        *state.current_scenario.lock().await = None;
    }
    Json(OkResponse {
        ok: true,
        message: None,
    })
}

async fn agent_status(State(state): State<AgentState>) -> impl IntoResponse {
    let running = *state.is_running.lock().await;
    let scenario = state.current_scenario.lock().await.clone();
    Json(StatusResponse { running, scenario })
}

async fn agent_upload_file(
    State(state): State<AgentState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(n) if !n.is_empty() => n.to_owned(),
            _ => continue,
        };

        // Only allow .xlsx, .csv, .xls
        let lower = file_name.to_lowercase();
        if !lower.ends_with(".xlsx") && !lower.ends_with(".csv") && !lower.ends_with(".xls") {
            return (
                StatusCode::BAD_REQUEST,
                Json(OkResponse {
                    ok: false,
                    message: Some("xlsx / csv / xls ファイルのみアップロード可能です".into()),
                }),
            )
                .into_response();
        }

        // Sanitise the file name (no path separators)
        let safe_name: String = file_name
            .chars()
            .filter(|&c| c != '/' && c != '\\' && c != '\0')
            .collect();
        let dest = state.scenarios_dir.join(&safe_name);

        match field.bytes().await {
            Ok(data) => {
                if let Err(e) = tokio::fs::write(&dest, &data).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(OkResponse {
                            ok: false,
                            message: Some(e.to_string()),
                        }),
                    )
                        .into_response();
                }
                return Json(serde_json::json!({ "ok": true, "path": safe_name })).into_response();
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(OkResponse {
                        ok: false,
                        message: Some(e.to_string()),
                    }),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Json(OkResponse {
            ok: false,
            message: Some("ファイルが見つかりません".into()),
        }),
    )
        .into_response()
}

const KEYCHAIN_SERVICE: &str = "robost";
const KEYCHAIN_API_KEY_ACCOUNT: &str = "anthropic_api_key";

fn keychain_get_api_key() -> Option<String> {
    keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_API_KEY_ACCOUNT)
        .ok()?
        .get_password()
        .ok()
}

/// Env var wins if set (no I/O); otherwise falls back to the OS keychain, which is
/// blocking I/O and must not run directly on the async executor thread.
async fn resolve_api_key() -> Option<String> {
    if let Some(k) = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
    {
        return Some(k);
    }
    tokio::task::spawn_blocking(keychain_get_api_key)
        .await
        .ok()
        .flatten()
}

async fn agent_chat(Json(body): Json<ChatBody>) -> impl IntoResponse {
    let api_key = match resolve_api_key().await {
        Some(k) => k,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(OkResponse {
                    ok: false,
                    message: Some(
                        "ANTHROPIC_API_KEY is not set. Set it in Settings, or export it in the shell that starts the agent: `export ANTHROPIC_API_KEY=sk-...`"
                            .into(),
                    ),
                }),
            )
                .into_response()
        }
    };

    let model =
        std::env::var("ROBOST_AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string());

    let system = if let Some(ref yaml) = body.scenario_yaml {
        format!(
            "You are an expert RPA scenario creation assistant for robost. \
             Help users build YAML automation scenarios for desktop automation. \
             When generating or modifying a scenario, output the complete YAML as a fenced code block (```yaml ... ```). \
             Current scenario YAML:\n```yaml\n{yaml}\n```"
        )
    } else {
        "You are an expert RPA scenario creation assistant for robost. \
         Help users build YAML automation scenarios for desktop automation. \
         When generating or modifying a scenario, output the complete YAML as a fenced code block (```yaml ... ```)."
            .to_string()
    };

    let messages: Vec<serde_json::Value> = body
        .messages
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();

    let request_body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "system": system,
        "messages": messages,
    });

    let result = tokio::task::spawn_blocking(move || {
        ureq::post("https://api.anthropic.com/v1/messages")
            .set("x-api-key", &api_key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(&request_body)
            .map_err(Box::new)
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
            let body: serde_json::Value = resp.into_json().unwrap_or_default();
            let reply = body["content"][0]["text"]
                .as_str()
                .unwrap_or("(empty response)")
                .to_string();
            Json(ChatResponse { reply }).into_response()
        }
        Ok(Err(e)) => {
            let message = match *e {
                ureq::Error::Status(code, response) => {
                    let detail = response
                        .into_json::<serde_json::Value>()
                        .ok()
                        .and_then(|b| b["error"]["message"].as_str().map(str::to_string));
                    match code {
                        401 => format!(
                            "Anthropic APIキーが無効です(401 Unauthorized)。設定画面でキーを確認・再設定してください。{}",
                            detail.map(|d| format!("詳細: {d}")).unwrap_or_default()
                        ),
                        429 => "Anthropicのレート制限に達しました(429)。しばらく待って再試行してください。".to_string(),
                        _ => format!(
                            "Anthropic APIエラー ({code}){}",
                            detail.map(|d| format!(": {d}")).unwrap_or_default()
                        ),
                    }
                }
                ureq::Error::Transport(t) => format!("Anthropic APIに接続できませんでした: {t}"),
            };
            (
                StatusCode::BAD_GATEWAY,
                Json(OkResponse {
                    ok: false,
                    message: Some(message),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
struct SettingsResponse {
    has_api_key: bool,
}

async fn agent_get_settings() -> impl IntoResponse {
    let has_api_key = resolve_api_key().await.is_some();
    Json(SettingsResponse { has_api_key })
}

#[derive(Deserialize)]
struct SaveSettingsBody {
    api_key: String,
}

/// Runs the actual (blocking) keychain write/delete off the async executor thread.
/// Returns `Ok(Some(message))` on success with an optional user-facing note,
/// `Err(message)` on a real failure (an already-absent entry is not an error).
fn save_settings_blocking(key: String) -> std::result::Result<Option<String>, String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_API_KEY_ACCOUNT)
        .map_err(|e| e.to_string())?;

    if key.is_empty() {
        if let Err(e) = entry.delete_credential() {
            if !matches!(e, keyring::Error::NoEntry) {
                return Err(format!("キーチェーンからの削除に失敗しました: {e}"));
            }
        }
        let env_key_present = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty())
            .is_some();
        let message = env_key_present.then(|| {
            "キーチェーンからは削除しましたが、環境変数 ANTHROPIC_API_KEY が設定されているため引き続きそちらが使用されます。"
                .to_string()
        });
        return Ok(message);
    }

    entry.set_password(&key).map_err(|e| e.to_string())?;
    Ok(None)
}

async fn agent_save_settings(Json(body): Json<SaveSettingsBody>) -> impl IntoResponse {
    let key = body.api_key.trim().to_string();
    match tokio::task::spawn_blocking(move || save_settings_blocking(key)).await {
        Ok(Ok(message)) => Json(OkResponse { ok: true, message }).into_response(),
        Ok(Err(message)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(message),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OkResponse {
                ok: false,
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

#[cfg(feature = "embed-editor")]
#[derive(rust_embed::RustEmbed)]
#[folder = "../../apps/web-editor/dist/"]
struct EditorAssets;

#[cfg(feature = "embed-editor")]
async fn agent_index(uri: axum::http::Uri) -> impl IntoResponse {
    use axum::body::Body;
    use axum::response::Response;
    let path = uri.path().trim_start_matches('/');
    // SPA fallback only for extension-less paths (e.g. /scenarios/123).
    // Paths with extensions that are absent from the bundle get a plain 404
    // so the browser's module loader sees a real error instead of HTML.
    // Track the actual filename being served so MIME is derived from the real
    // extension, not the request path (e.g. "/" and "/scenarios/123" both serve
    // index.html and must get text/html, not application/octet-stream).
    let (file, served_name) = if path.is_empty() {
        (EditorAssets::get("index.html"), "index.html")
    } else if let Some(f) = EditorAssets::get(path) {
        (Some(f), path)
    } else if !path.contains('.') {
        (EditorAssets::get("index.html"), "index.html")
    } else {
        (None, path)
    };
    match file {
        Some(f) => {
            let mime = mime_guess::from_path(served_name).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(f.data.into_owned()))
                .expect("infallible: valid MIME string and owned bytes")
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .expect("infallible: known-valid status and empty body"),
    }
}

// Fallback HTML for when the built web UI is not embedded
#[cfg(not(feature = "embed-editor"))]
async fn agent_index() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="ja">
<head><meta charset="UTF-8"><title>robost agent</title>
<style>body{font-family:monospace;margin:2rem;background:#1e1e1e;color:#ccc}</style>
</head>
<body>
<h2>robost agent is running</h2>
<p>Start the web UI with:</p>
<pre>  cd apps/web-editor && npm run dev</pre>
<p>Then open <a href="http://localhost:5173" style="color:#7cb8ff">http://localhost:5173</a></p>
</body>
</html>"#;
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

// ── Server initialization ─────────────────────────────────────────────────────

/// Start the run-mode SSE server (used by `rpa run --serve`).
/// Returns the broadcast sender and bound port.
pub async fn run_server(bind_addr: &str) -> Result<(Arc<broadcast::Sender<ProgressEvent>>, u16)> {
    let (tx, _rx) = broadcast::channel::<ProgressEvent>(64);
    let tx = Arc::new(tx);

    let state = ServerState {
        progress_broadcast: tx.clone(),
    };

    let app = Router::new()
        .route("/events", get(handle_events))
        .route("/health", get(handle_health))
        .with_state(state);

    eprintln!("[server] binding to {bind_addr}");
    let listener = TcpListener::bind(bind_addr)
        .await
        .context(format!("failed to bind to {bind_addr}"))?;

    let bound_addr = listener
        .local_addr()
        .context("failed to get bound address")?;
    let bound_port = bound_addr.port();

    eprintln!("[server] bound to {bound_addr}, listening on port {bound_port}");
    println!("PORT={}", bound_port);
    std::io::Write::flush(&mut std::io::stdout()).ok();

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[server] HTTP server error: {e}");
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok((tx, bound_port))
}

/// Start the agent HTTP server — long-running, blocks until Ctrl+C.
pub async fn run_agent_server(bind_addr: &str, scenarios_dir: PathBuf) -> Result<()> {
    let (tx, _) = broadcast::channel::<ProgressEvent>(128);
    let state = AgentState {
        progress_tx: Arc::new(tx),
        abort_tx: Arc::new(Mutex::new(None)),
        is_running: Arc::new(Mutex::new(false)),
        current_scenario: Arc::new(Mutex::new(None)),
        scenarios_dir,
    };

    let app = Router::new()
        // SSE + screenshot (same paths used by Electron version)
        .route("/events", get(agent_events))
        .route("/screenshot", get(agent_screenshot))
        .route("/health", get(handle_health))
        // Scenario management
        .route("/api/scenarios", get(agent_list_scenarios))
        .route("/api/scenarios/*name", get(agent_get_scenario))
        .route("/api/scenarios/*name", post(agent_save_scenario))
        .route("/api/scenarios/*name", delete(agent_delete_scenario))
        // File upload (Excel/CSV for data_source)
        .route("/api/upload", post(agent_upload_file))
        // Folder management
        .route("/api/folders", post(agent_create_folder))
        .route("/api/folders/:name", delete(agent_delete_folder))
        // RPA control
        .route("/api/run", post(agent_run))
        .route("/api/stop", post(agent_stop))
        .route("/api/status", get(agent_status))
        // AI chat
        .route("/api/chat", post(agent_chat))
        .route(
            "/api/settings",
            get(agent_get_settings).post(agent_save_settings),
        )
        // Web UI (fallback)
        .fallback(agent_index)
        .with_state(state);

    let listener = TcpListener::bind(bind_addr)
        .await
        .context(format!("failed to bind agent server to {bind_addr}"))?;
    let bound = listener.local_addr()?;

    println!("robost agent listening on http://{bound}");
    println!("Open http://localhost:{} in your browser", bound.port());

    axum::serve(listener, app)
        .await
        .context("agent server error")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_binding() {
        let result = run_server("127.0.0.1:0").await;
        assert!(result.is_ok());
        let (_, port) = result.unwrap();
        assert!(port > 0);
    }

    #[tokio::test]
    async fn test_broadcast_event() {
        let (tx, _port) = run_server("127.0.0.1:0").await.unwrap();
        let _ = tx.send(ProgressEvent::Finished {
            success: true,
            error: None,
        });
    }
}
