//! Inter-process communication (IPC) protocol definitions and limits.
//!
//! Provides JSON request and response payloads exchanged over `/run/systemd-sentry/sentry.sock`.

use serde::{Deserialize, Serialize};

/// Maximum allowed IPC frame size in bytes (32 KiB) to prevent memory exhaustion attacks.
pub const MAX_IPC_FRAME_SIZE: usize = 32 * 1024;

/// Requests sent from CLI clients to the supervisor daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcRequest {
    /// Liveness ping.
    Ping,
    /// Query daemon status, uptime, and tracked units.
    Status,
    /// List recent incidents from in-memory ring buffer.
    ListIncidents {
        /// Maximum number of incidents to return.
        limit: usize,
    },
    /// Inspect full telemetry and diagnostic for a specific incident ID.
    InspectIncident {
        /// Incident UUID.
        id: String,
    },
    /// Reset circuit breaker for a locked unit (requires root or sentry user).
    ResetCircuit {
        /// Target unit name.
        unit: String,
    },
    /// Trigger atomic reload of policy and configuration files.
    ReloadConfig,
    /// Subscribe to live stream of events (for `systemd-sentry monitor`).
    SubscribeEvents,
}

/// Responses returned by the daemon to CLI clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum IpcResponse {
    /// Operation succeeded with attached JSON payload.
    #[serde(rename = "ok")]
    Ok {
        /// Result data payload.
        data: serde_json::Value,
    },
    /// Operation failed with error code and description.
    #[serde(rename = "error")]
    Error {
        /// Numeric error code (e.g. sysexits or JSON-RPC style).
        code: i32,
        /// Human-readable explanation.
        message: String,
    },
    /// Asynchronous notification event for subscribed monitor clients.
    #[serde(rename = "event")]
    Event {
        /// Event payload.
        data: serde_json::Value,
    },
}
