//! Prompt template generator for `prompts/get` requests.

use crate::protocol::error_codes::INVALID_PARAMS;
use crate::protocol::types::JsonRpcError;
use crate::storage::McpState;
use serde_json::{json, Value};

/// Generates populated prompt messages for client agent workflows.
pub fn generate_prompt(
    state: &McpState,
    name: &str,
    args: Option<&Value>,
) -> Result<Value, JsonRpcError> {
    match name {
        "triage_incident" => {
            let incident_id = match args.and_then(|a| a.get("incident_id")).and_then(|v| v.as_str()) {
                Some(id) => id,
                None => {
                    return Err(JsonRpcError::new(
                        INVALID_PARAMS,
                        "Missing required argument 'incident_id'",
                    ));
                }
            };

            let incident = match state.get_incident(incident_id) {
                Some(inc) => inc,
                None => {
                    return Err(JsonRpcError::new(
                        INVALID_PARAMS,
                        format!("Incident not found: '{incident_id}'"),
                    ));
                }
            };

            let policy = state.get_policy();
            let incident_text = serde_json::to_string_pretty(&incident).unwrap_or_default();

            Ok(json!({
                "description": format!("Root-cause triage for incident {incident_id}"),
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Perform a comprehensive root-cause triage of incident {incident_id}.\n\n\
                                Incident Record:\n{incident_text}\n\n\
                                Safety Policy:\n{policy}\n\n\
                                Provide a detailed analysis explaining the root cause, evidentiary backing, and evaluate whether the proposed remediation is safe under current policy."
                            )
                        }
                    }
                ]
            }))
        }
        "analyze_flapping_service" => {
            let unit_name = match args.and_then(|a| a.get("unit_name")).and_then(|v| v.as_str()) {
                Some(name) => name,
                None => {
                    return Err(JsonRpcError::new(
                        INVALID_PARAMS,
                        "Missing required argument 'unit_name'",
                    ));
                }
            };

            let telemetry = state.get_unit_telemetry(unit_name);
            let telemetry_text = serde_json::to_string_pretty(&telemetry).unwrap_or_default();
            let circuit_status = state.get_circuit_status();
            let circuit_text = serde_json::to_string_pretty(&circuit_status).unwrap_or_default();

            Ok(json!({
                "description": format!("Flap analysis for unit {unit_name}"),
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Analyze flapping behavior and failure storm risks for unit {unit_name}.\n\n\
                                Telemetry:\n{telemetry_text}\n\n\
                                Circuit Breaker State:\n{circuit_text}\n\n\
                                Assess whether the unit should be locked out, stabilized, or escalated to human administration."
                            )
                        }
                    }
                ]
            }))
        }
        other => Err(JsonRpcError::new(
            INVALID_PARAMS,
            format!("Unknown prompt template: '{other}'"),
        )),
    }
}
