//! MCP Tool catalog and schema registrations.

use serde_json::{json, Value};

/// Returns the list of tools exposed by systemd-sentry-mcp.
pub fn list_tools() -> Value {
    json!([
        {
            "name": "get_incident",
            "description": "Retrieve full diagnostic payload, logs, signals, and PSI metrics for a specific incident UUID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "incident_id": {
                        "type": "string",
                        "description": "Unique UUID of the incident"
                    }
                },
                "required": ["incident_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_incidents",
            "description": "List recent failure incidents recorded by systemd-sentry with optional severity and unit filtering.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of incidents to return (default: 10)",
                        "default": 10
                    },
                    "unit_name": {
                        "type": "string",
                        "description": "Filter by systemd unit name (e.g. 'nginx.service')"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["LOW", "MEDIUM", "HIGH", "CRITICAL"],
                        "description": "Filter by severity level"
                    }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "get_unit_telemetry",
            "description": "Inspect live real-time cgroups v2 resource usage, PSI pressure metrics, D-Bus ActiveState, and circuit breaker trip state for any unit.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "unit_name": {
                        "type": "string",
                        "description": "Target systemd unit (e.g. 'postgresql.service')"
                    }
                },
                "required": ["unit_name"],
                "additionalProperties": false
            }
        },
        {
            "name": "explain_incident",
            "description": "Generate an interactive forensic breakdown of an incident with specific focus (root cause, remediation safety, kernel evidence).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "incident_id": {
                        "type": "string",
                        "description": "Target incident UUID"
                    },
                    "focus": {
                        "type": "string",
                        "enum": ["root_cause", "safety_validation", "coredump_trace", "psi_pressure"],
                        "description": "Aspect of the incident to drill into"
                    }
                },
                "required": ["incident_id"],
                "additionalProperties": false
            }
        }
    ])
}
