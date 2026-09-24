//! Resource reader fulfilling `resources/read` MCP requests.

use crate::protocol::error_codes::INVALID_PARAMS;
use crate::protocol::types::JsonRpcError;
use crate::storage::McpState;
use serde_json::{json, Value};

/// Reads and formats an MCP resource by URI.
pub fn read_resource(state: &McpState, uri: &str) -> Result<Value, JsonRpcError> {
    let normalized = if let Some(stripped) = uri.strip_prefix("sentry://") {
        if let Some(id) = stripped.strip_prefix("incidents/") {
            format!("incident://{id}")
        } else if let Some(unit) = stripped.strip_prefix("telemetry/") {
            format!("telemetry://{unit}")
        } else if stripped == "policy" {
            "policy://current".to_string()
        } else if stripped == "circuit" || stripped == "circuit/status" {
            "circuit://status".to_string()
        } else {
            uri.to_string()
        }
    } else {
        uri.to_string()
    };
    let uri_str = normalized.as_str();

    if let Some(id) = uri_str.strip_prefix("incident://") {
        match state.get_incident(id) {
            Some(incident) => {
                let text = serde_json::to_string_pretty(&incident)
                    .unwrap_or_else(|_| format!("{incident:?}"));
                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": text
                    }]
                }))
            }
            None => Err(JsonRpcError::new(
                INVALID_PARAMS,
                format!("Resource not found: '{uri}'"),
            )),
        }
    } else if let Some(unit) = uri_str.strip_prefix("telemetry://") {
        let telemetry = state.get_unit_telemetry(unit);
        let text = serde_json::to_string_pretty(&telemetry)
            .unwrap_or_else(|_| format!("{telemetry:?}"));
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": text
            }]
        }))
    } else if uri_str == "policy://current" {
        let policy_text = state.get_policy();
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "text/x-toml",
                "text": policy_text
            }]
        }))
    } else if uri_str == "circuit://status" {
        let circuit_table = state.get_circuit_status();
        let text = serde_json::to_string_pretty(&circuit_table)
            .unwrap_or_else(|_| format!("{circuit_table:?}"));
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": text
            }]
        }))
    } else {
        Err(JsonRpcError::new(
            INVALID_PARAMS,
            format!("Unsupported resource URI scheme: '{uri}'"),
        ))
    }
}
