//! MCP Tools implementation and dispatcher.

pub mod explain_incident;
pub mod get_incident;
pub mod get_telemetry;
pub mod list_incidents;
pub mod registry;

pub use explain_incident::execute_explain_incident;
pub use get_incident::execute_get_incident;
pub use get_telemetry::execute_get_telemetry;
pub use list_incidents::execute_list_incidents;
pub use registry::list_tools;

use crate::storage::McpState;
use serde_json::{json, Value};

/// Dispatches tool call by name to its corresponding handler.
pub fn call_tool(state: &McpState, name: &str, args: Option<&Value>) -> Value {
    match name {
        "get_incident" => execute_get_incident(state, args),
        "list_incidents" => execute_list_incidents(state, args),
        "get_unit_telemetry" => execute_get_telemetry(state, args),
        "explain_incident" => execute_explain_incident(state, args),
        other => json!({
            "content": [{
                "type": "text",
                "text": format!("Unknown tool: '{other}'")
            }],
            "isError": true
        }),
    }
}
