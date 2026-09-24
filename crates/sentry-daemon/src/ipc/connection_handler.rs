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
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let mut bytes_read = 0;
        let mut exceeded = false;

        loop {
            let available = match reader.fill_buf().await {
                Ok(b) => b,
                Err(e) => {
                    debug!("Error reading from IPC client: {}", e);
                    break;
                }
            };
            if available.is_empty() {
                break;
            }
            if let Some(pos) = available.iter().position(|&b| b == b'\n') {
                let to_take = pos + 1;
                if !exceeded && bytes_read + to_take <= MAX_IPC_FRAME_SIZE {
                    buf.extend_from_slice(&available[..to_take]);
                } else {
                    exceeded = true;
                }
                bytes_read += to_take;
                reader.consume(to_take);
                break;
            } else {
                let len = available.len();
                if !exceeded && bytes_read + len <= MAX_IPC_FRAME_SIZE {
                    buf.extend_from_slice(available);
                } else {
                    exceeded = true;
                }
                bytes_read += len;
                reader.consume(len);
            }
        }

        if bytes_read == 0 {
            break;
        }

        if exceeded || bytes_read > MAX_IPC_FRAME_SIZE {
            warn!("Client frame exceeded {} byte limit; disconnecting", MAX_IPC_FRAME_SIZE);
            let _ = write_response(&mut write_half, &IpcResponse::Error {
                code: -32001,
                message: "Frame size exceeded 32 KiB limit".to_string(),
            }).await;
            break;
        }

        let line = match std::str::from_utf8(&buf) {
            Ok(s) => s,
            Err(_) => {
                let _ = write_response(&mut write_half, &IpcResponse::Error {
                    code: -32700,
                    message: "Parse error: invalid UTF-8".to_string(),
                }).await;
                continue;
            }
        };

        let request: IpcRequest = match serde_json::from_str(line) {
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
                {
                    let mut s = state.lock().await;
                    let base = std::path::Path::new(&s.config.policy_path);
                    let dropin = std::path::Path::new(&s.config.policy_dropin_dir);
                    let policy = sentry_safety::policy::load_policy_with_dropins(base, dropin);
                    s.policy_gatekeeper = sentry_safety::policy::PolicyGatekeeper::new(policy);
                }
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

                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            if write_response(&mut write_half, &event).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Event subscriber lagged, skipped {} event(s)", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
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
