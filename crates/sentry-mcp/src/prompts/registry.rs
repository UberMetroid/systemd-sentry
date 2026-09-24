//! MCP Prompts catalog and argument definitions.

use serde_json::{json, Value};

/// Returns the catalog of prompt templates supported by the MCP server.
pub fn list_prompts() -> Value {
    json!([
        {
            "name": "triage_incident",
            "description": "Generate prompt template for comprehensive root-cause triage of a systemd failure incident.",
            "arguments": [
                {
                    "name": "incident_id",
                    "description": "Unique UUID of the incident to triage",
                    "required": true
                }
            ]
        },
        {
            "name": "analyze_flapping_service",
            "description": "Generate prompt template to analyze a flapping service and recommend lockout or stabilization actions.",
            "arguments": [
                {
                    "name": "unit_name",
                    "description": "Target systemd unit name (e.g. 'api-worker.service')",
                    "required": true
                }
            ]
        }
    ])
}
