use anyhow::{Context, Result};
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use robost_core::ProgressEvent;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

// ── Server State ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ServerState {
    progress_broadcast: Arc<broadcast::Sender<ProgressEvent>>,
}

// ── HTTP Handlers ───────────────────────────────────────────────────────────

/// Stream scenario progress as Server-Sent Events (SSE).
async fn handle_events(
    State(state): State<ServerState>,
) -> Sse<impl futures::stream::Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.progress_broadcast.subscribe();
    let stream = BroadcastStream::new(rx).map(|result| {
        match result {
            Ok(event) => {
                let json = serde_json::to_string(&event).unwrap_or_default();
                let sse_event = Event::default().data(json);
                Ok(sse_event)
            }
            Err(_) => {
                // Broadcast stream ended, close SSE connection.
                let sse_event = Event::default().data("{}");
                Ok(sse_event)
            }
        }
    });

    Sse::new(stream)
}

/// Health check endpoint.
async fn handle_health() -> &'static str {
    "OK"
}

// ── Server Initialization ───────────────────────────────────────────────────

/// Run an HTTP server on the given bind address.
/// Returns the broadcast sender and bound port number.
pub async fn run_server(bind_addr: &str) -> Result<(Arc<broadcast::Sender<ProgressEvent>>, u16)> {
    // Create broadcast channel (64 queued messages).
    let (tx, _rx) = broadcast::channel::<ProgressEvent>(64);
    let tx = Arc::new(tx);

    // Build router.
    let state = ServerState {
        progress_broadcast: tx.clone(),
    };

    let app = Router::new()
        .route("/events", get(handle_events))
        .route("/health", get(handle_health))
        .with_state(state);

    // Bind to address. Port 0 = dynamic allocation.
    eprintln!("[server] binding to {bind_addr}");
    let listener = TcpListener::bind(bind_addr)
        .await
        .context(format!("failed to bind to {bind_addr}"))?;

    let bound_addr = listener
        .local_addr()
        .context("failed to get bound address")?;
    let bound_port = bound_addr.port();

    eprintln!("[server] bound to {bound_addr}, listening on port {bound_port}");

    // Print port to stdout for parent process (must be before spawning).
    println!("PORT={}", bound_port);
    std::io::Write::flush(&mut std::io::stdout()).ok();

    // Run server in background.
    let app_clone = app;
    eprintln!("[server] spawning HTTP server for port {bound_port}");
    tokio::spawn(async move {
        eprintln!(
            "[server] HTTP server task started, about to call axum::serve for port {bound_port}"
        );
        let result = axum::serve(listener, app_clone).await;
        eprintln!("[server] HTTP server task finished axum::serve for port {bound_port}");
        match result {
            Ok(_) => eprintln!("[server] HTTP server finished normally"),
            Err(e) => {
                eprintln!("[server] HTTP server error: {e}");
            }
        }
    });

    // Give the server a moment to start listening.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok((tx, bound_port))
}

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

        let event = ProgressEvent::Finished {
            success: true,
            error: None,
        };

        let _ = tx.send(event);
    }
}
