//! Handler for `get_incident` tool call.

use crate::storage::McpState;
use serde_json::{json, Value};

/// Executes the `get_incident` tool.
pub fn execute_get_incident(state: &McpState, args: Option<&Value>) -> Value {
    let incident_id = match args.and_then(|a| a.get("incident_id")).and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return json!({
                "content": [{
                    "type": "text",
                    "text": "Missing required argument 'incident_id'"
                }],
                "isError": true
            });
        }
    };

    match state.get_incident(incident_id) {
        Some(incident) => {
            let text = serde_json::to_string_pretty(&incident)
                .unwrap_or_else(|_| format!("{incident:?}"));
            json!({
                "content": [{
                    "type": "text",
                    "text": text
                }],
                "isError": false
            })
        }
        None => json!({
            "content": [{
                "type": "text",
                "text": format!("Incident not found: '{incident_id}'")
            }],
            "isError": true
        }),
    }
}
