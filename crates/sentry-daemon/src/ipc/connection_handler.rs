//! Async IPC connection handler enforcing peer credentials and frame limits.

use crate::daemon::state::DaemonState;
use crate::ipc::peer_cred::{authorize_action, get_peer_credentials};
use crate::ipc::protocol::{IpcRequest, IpcResponse, MAX_IPC_FRAME_SIZE};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Process a single incoming IPC client connection.
pub async fn handle_ipc_connection(
    mut stream: UnixStream,
    state: Arc<Mutex<DaemonState>>,
    daemon_uid: u32,
) {
    let creds = match get_peer_credentials(&stream) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to obtain peer credentials: {}", e);
            return;
        }
    };

    let (read_half, mut write_half) = stream.split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = match reader.read_line(&mut line).await {
            Ok(0) => break, // Client disconnected
            Ok(n) if n > MAX_IPC_FRAME_SIZE => {
                warn!("Client frame exceeded {} byte limit; disconnecting", MAX_IPC_FRAME_SIZE);
                let _ = write_response(&mut write_half, &IpcResponse::Error {
                    code: -32001,
                    message: "Frame size exceeded 32 KiB limit".to_string(),
                }).await;
                break;
            }
            Ok(n) => n,
            Err(e) => {
                debug!("Error reading from IPC client: {}", e);
                break;
            }
        };

        if bytes_read == 0 {
            break;
        }

        let request: IpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let _ = write_response(&mut write_half, &IpcResponse::Error {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                }).await;
                continue;
            }
        };

        let is_mutating = matches!(request, IpcRequest::ResetCircuit { .. } | IpcRequest::ReloadConfig);
        if let Err((code, msg)) = authorize_action(&creds, daemon_uid, is_mutating) {
            let _ = write_response(&mut write_half, &IpcResponse::Error { code, message: msg }).await;
            continue;
        }

        match request {
            IpcRequest::Ping => {
                let _ = write_response(&mut write_half, &IpcResponse::Ok {
                    data: serde_json::json!({ "pong": true }),
                }).await;
            }
            IpcRequest::Status => {
                let response = {
                    let mut s = state.lock().await;
                    let rss = s.statm_reader.read_rss_bytes();
                    let uptime = s.start_time.elapsed().as_secs();
                    let dropped = s.dropped_events.load(Ordering::Relaxed);
                    let degraded = s.load_shedder.is_degraded();
                    let breakers = s.circuit_registry.list_snapshots(std::time::Instant::now());

                    IpcResponse::Ok {
                        data: serde_json::json!({
                            "uptime_seconds": uptime,
                            "rss_bytes": rss,
                            "rss_mb": (rss as f64) / (1024.0 * 1024.0),
                            "degraded_mode": degraded,
                            "dropped_events": dropped,
                            "tracked_units_count": breakers.len(),
                            "circuit_breakers": breakers,
                        }),
                    }
                };
                let _ = write_response(&mut write_half, &response).await;
            }
            IpcRequest::ListIncidents { limit } => {
                let incidents = {
                    let s = state.lock().await;
                    s.mcp_state.list_incidents(limit, None, None)
                };
                let _ = write_response(&mut write_half, &IpcResponse::Ok {
                    data: serde_json::to_value(incidents).unwrap_or_default(),
                }).await;
            }
            IpcRequest::InspectIncident { id } => {
                let result = {
                    let s = state.lock().await;
                    s.mcp_state.get_incident(&id)
                };
                let resp = match result {
                    Some(inc) => IpcResponse::Ok { data: serde_json::to_value(inc).unwrap_or_default() },
                    None => IpcResponse::Error { code: 404, message: format!("Incident {} not found", id) },
                };
                let _ = write_response(&mut write_half, &resp).await;
            }
            IpcRequest::ResetCircuit { unit } => {
                let mut s = state.lock().await;
                s.circuit_registry.reset_unit(&unit, std::time::Instant::now());
                let _ = write_response(&mut write_half, &IpcResponse::Ok {
                    data: serde_json::json!({ "unit": unit, "reset": true }),
                }).await;
            }
            IpcRequest::ReloadConfig => {
                let _ = write_response(&mut write_half, &IpcResponse::Ok {
                    data: serde_json::json!({ "reloaded": true }),
                }).await;
            }
            IpcRequest::SubscribeEvents => {
                let mut rx = {
                    let s = state.lock().await;
                    s.event_broadcaster.subscribe()
                };
                let _ = write_response(&mut write_half, &IpcResponse::Ok {
                    data: serde_json::json!({ "subscribed": true }),
                }).await;

                while let Ok(event) = rx.recv().await {
                    if write_response(&mut write_half, &event).await.is_err() {
                        break;
                    }
                }
                break;
            }
        }
    }
}

async fn write_response<W: AsyncWriteExt + Unpin>(writer: &mut W, response: &IpcResponse) -> Result<(), std::io::Error> {
    let mut bytes = serde_json::to_vec(response)?;
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await
}
