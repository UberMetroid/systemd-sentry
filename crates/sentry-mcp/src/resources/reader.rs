//! Resource reader fulfilling `resources/read` MCP requests.

use crate::protocol::error_codes::INVALID_PARAMS;
use crate::protocol::types::JsonRpcError;
use crate::storage::McpState;
use serde_json::{json, Value};

/// Reads and formats an MCP resource by URI.
pub fn read_resource(state: &McpState, uri: &str) -> Result<Value, JsonRpcError> {
    if let Some(id) = uri.strip_prefix("incident://") {
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
    } else if let Some(unit) = uri.strip_prefix("telemetry://") {
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
    } else if uri == "policy://current" {
        let policy_text = state.get_policy();
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "text/x-toml",
                "text": policy_text
            }]
        }))
    } else if uri == "circuit://status" {
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
