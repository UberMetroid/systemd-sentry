//! Handler for `explain_incident` tool call.

use crate::storage::McpState;
use serde_json::{json, Value};

/// Executes the `explain_incident` forensic tool.
pub fn execute_explain_incident(state: &McpState, args: Option<&Value>) -> Value {
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

    let focus = args
        .and_then(|a| a.get("focus"))
        .and_then(|v| v.as_str())
        .unwrap_or("root_cause");

    let incident = match state.get_incident(incident_id) {
        Some(inc) => inc,
        None => {
            return json!({
                "content": [{
                    "type": "text",
                    "text": format!("Incident not found: '{incident_id}'")
                }],
                "isError": true
            });
        }
    };

    let explanation = match focus {
        "root_cause" => format!(
            "Forensic Analysis for incident {}:\nUnit: {}\nSummary: {}\nDetail: {}",
            incident.incident_id, incident.unit_name, incident.root_cause.summary, incident.root_cause.detail
        ),
        "safety_validation" => format!(
            "Safety Assessment for incident {}:\nUnit: {}\nProposed Action: {:?}\nRisk Level: {:?}\nConfidence: {:.2}\nRationale: {}",
            incident.incident_id,
            incident.unit_name,
            incident.proposed_remediation.action,
            incident.proposed_remediation.risk_level,
            incident.proposed_remediation.confidence,
            incident.proposed_remediation.rationale
        ),
        "coredump_trace" => format!(
            "Coredump Stack Trace for incident {}:\nUnit: {}\nTrace:\n{}",
            incident.incident_id,
            incident.unit_name,
            incident.evidence.coredump.as_deref().unwrap_or("No coredump recorded for this incident.")
        ),
        "psi_pressure" => format!(
            "Subsystem Pressure Telemetry (PSI) for incident {}:\nUnit: {}\nPSI: {:?}",
            incident.incident_id, incident.unit_name, incident.evidence.psi
        ),
        other => format!(
            "Unrecognized focus area: '{other}'. Available: 'root_cause', 'safety_validation', 'coredump_trace', 'psi_pressure'."
        ),
    };

    json!({
        "content": [{
            "type": "text",
            "text": explanation
        }],
        "isError": false
    })
}
