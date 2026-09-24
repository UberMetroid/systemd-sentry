//! Handler for `list_incidents` tool call.

use crate::storage::McpState;
use serde_json::{json, Value};

/// Executes the `list_incidents` tool.
pub fn execute_list_incidents(state: &McpState, args: Option<&Value>) -> Value {
    let limit = args
        .and_then(|a| a.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(10);

    let unit_filter = args
        .and_then(|a| a.get("unit_name"))
        .and_then(|v| v.as_str());

    let severity_filter = args
        .and_then(|a| a.get("severity"))
        .and_then(|v| v.as_str());

    let incidents = state.list_incidents(limit, unit_filter, severity_filter);

    let summaries: Vec<Value> = incidents
        .into_iter()
        .map(|inc| {
            json!({
                "incident_id": inc.incident_id.to_string(),
                "timestamp": inc.timestamp.to_rfc3339(),
                "unit_name": inc.unit_name,
                "severity": inc.severity.as_str(),
                "root_cause_summary": inc.root_cause.summary,
                "proposed_action": inc.proposed_remediation.action.as_str()
            })
        })
        .collect();

    let text = serde_json::to_string_pretty(&summaries)
        .unwrap_or_else(|_| "[]".to_string());

    json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "isError": false
    })
}
