//! MCP Resources catalog and schema listings.

use serde_json::{json, Value};

/// Returns the catalog of URI resources exposed by the MCP server.
pub fn list_resources() -> Value {
    json!([
        {
            "uri": "incident://{incident_id}",
            "name": "Incident Record",
            "description": "Complete serialized diagnostic report and raw evidence log for an incident UUID.",
            "mimeType": "application/json"
        },
        {
            "uri": "telemetry://{unit_name}",
            "name": "Live Unit Telemetry",
            "description": "Real-time cgroups v2 and PSI pressure snapshot for a specified systemd unit.",
            "mimeType": "application/json"
        },
        {
            "uri": "policy://current",
            "name": "Active Safety Policy",
            "description": "Active declarative safety policy loaded from /etc/systemd-sentry/policy.toml.",
            "mimeType": "text/x-toml"
        },
        {
            "uri": "circuit://status",
            "name": "Circuit Breaker Table",
            "description": "Global status of all unit circuit breakers and lockout timers.",
            "mimeType": "application/json"
        }
    ])
}
