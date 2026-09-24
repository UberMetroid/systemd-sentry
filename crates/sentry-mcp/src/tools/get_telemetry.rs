//! Handler for `get_unit_telemetry` tool call.

use crate::storage::McpState;
use serde_json::{json, Value};

/// Executes the `get_unit_telemetry` tool.
pub fn execute_get_telemetry(state: &McpState, args: Option<&Value>) -> Value {
    let unit_name = match args.and_then(|a| a.get("unit_name")).and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            return json!({
                "content": [{
                    "type": "text",
                    "text": "Missing required argument 'unit_name'"
                }],
                "isError": true
            });
        }
    };

    let telemetry = state.get_unit_telemetry(unit_name);
    let text = serde_json::to_string_pretty(&telemetry)
        .unwrap_or_else(|_| format!("{telemetry:?}"));

    json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "isError": false
    })
}
