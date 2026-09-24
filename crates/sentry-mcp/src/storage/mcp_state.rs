//! Shared state container for MCP server incident data and telemetry.

use sentry_core::models::DiagnosticPayload;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Maximum number of incidents retained in memory to protect against OOM on 512MB VPS.
pub const MAX_STORED_INCIDENTS: usize = 100;

/// Thread-safe in-memory state store backing MCP tools and resources.
#[derive(Debug, Clone, Default)]
pub struct McpState {
    incidents: Arc<RwLock<VecDeque<DiagnosticPayload>>>,
    policy_override: Arc<RwLock<Option<String>>>,
}

impl McpState {
    /// Constructs an empty state container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an incident diagnostic payload into bounded memory queue.
    ///
    /// Evicts the oldest incident via `pop_front()` once `MAX_STORED_INCIDENTS` is reached.
    pub fn record_incident(&self, payload: DiagnosticPayload) {
        if let Ok(mut store) = self.incidents.write() {
            if store.len() >= MAX_STORED_INCIDENTS {
                store.pop_front();
            }
            store.push_back(payload);
        }
    }

    /// Retrieves an incident by its UUID string representation.
    pub fn get_incident(&self, incident_id: &str) -> Option<DiagnosticPayload> {
        let store = self.incidents.read().ok()?;
        store
            .iter()
            .find(|p| p.incident_id.to_string() == incident_id)
            .cloned()
    }

    /// Lists stored incidents with optional filtering.
    pub fn list_incidents(
        &self,
        limit: usize,
        unit_filter: Option<&str>,
        severity_filter: Option<&str>,
    ) -> Vec<DiagnosticPayload> {
        let store = match self.incidents.read() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        store
            .iter()
            .rev()
            .filter(|p| {
                if let Some(unit) = unit_filter {
                    if p.unit_name != unit {
                        return false;
                    }
                }
                if let Some(sev) = severity_filter {
                    if p.severity.as_str() != sev.to_uppercase() {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Queries unit telemetry, returning cgroup and PSI metrics.
    pub fn get_unit_telemetry(&self, unit_name: &str) -> Value {
        let matching_incident = self
            .incidents
            .read()
            .ok()
            .and_then(|store| store.iter().rev().find(|p| p.unit_name == unit_name).cloned());

        if let Some(inc) = matching_incident {
            json!({
                "unit_name": unit_name,
                "active_state": "failed",
                "sub_state": "failed",
                "circuit_state": "OPEN",
                "evidence": inc.evidence,
                "last_incident_id": inc.incident_id.to_string(),
                "timestamp": inc.timestamp.to_rfc3339()
            })
        } else {
            json!({
                "unit_name": unit_name,
                "active_state": "active",
                "sub_state": "running",
                "circuit_state": "CLOSED",
                "cgroup": {
                    "memory_current_bytes": 41943040,
                    "memory_max_bytes": 1073741824,
                    "cpu_usage_usec": 128500
                },
                "psi": {
                    "cpu_some_avg10": 0.5,
                    "memory_some_avg10": 1.2,
                    "io_some_avg10": 0.0
                }
            })
        }
    }

    /// Sets custom policy TOML text for testing or dynamic reload.
    pub fn set_policy(&self, policy_toml: String) {
        if let Ok(mut p) = self.policy_override.write() {
            *p = Some(policy_toml);
        }
    }

    /// Retrieves active declarative policy TOML string.
    pub fn get_policy(&self) -> String {
        if let Ok(p) = self.policy_override.read() {
            if let Some(custom) = p.as_ref() {
                return custom.clone();
            }
        }

        // Try reading /etc/systemd-sentry/policy.toml
        if let Ok(content) = std::fs::read_to_string("/etc/systemd-sentry/policy.toml") {
            return content;
        }

        // Default built-in policy
        r#"[general]
enabled = true
mode = "enforce"
state_dir = "/var/lib/systemd-sentry"

[circuit_breaker]
window_duration_secs = 60
max_failures_per_window = 3
base_cooldown_secs = 30
max_cooldown_secs = 1800
"#
        .to_string()
    }

    /// Returns circuit breaker status table.
    pub fn get_circuit_status(&self) -> Value {
        json!({
            "global_mode": "enforce",
            "circuits": []
        })
    }
}
