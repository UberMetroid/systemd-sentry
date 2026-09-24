//! Diagnostic JSON triage parser & sanitizer for fuzzing.
//!
//! Robustly extracts structured diagnostic payloads from raw LLM output,
//! stripping markdown fences, isolating outer JSON braces, and validating schemas.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RemediationAction {
    NoAction,
    Restart,
    RestartWithBackoff,
    Reload,
    ResetFailed,
    EscalateToAdmin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    pub summary: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedRemediation {
    pub action: RemediationAction,
    pub rationale: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticPayload {
    pub incident_id: String,
    pub timestamp: String,
    pub unit_name: String,
    pub root_cause: RootCause,
    pub severity: Severity,
    pub proposed_remediation: ProposedRemediation,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TriageParseError {
    EmptyInput,
    NoJsonBoundary,
    MalformedJson,
    InvalidConfidenceRange,
}

pub fn sanitize_llm_json(input: &str) -> &str {
    let trimmed = input.trim();
    let unwrapped = if let Some(stripped) = trimmed.strip_prefix("```json") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else {
        trimmed
    };

    if let (Some(first_brace), Some(last_brace)) = (unwrapped.find('{'), unwrapped.rfind('}')) {
        if first_brace <= last_brace {
            return &unwrapped[first_brace..=last_brace];
        }
    }
    unwrapped
}

pub fn parse_and_validate_diagnostic_json(input: &str) -> Result<DiagnosticPayload, TriageParseError> {
    if input.trim().is_empty() {
        return Err(TriageParseError::EmptyInput);
    }

    let sanitized = sanitize_llm_json(input);
    if !sanitized.starts_with('{') || !sanitized.ends_with('}') {
        return Err(TriageParseError::NoJsonBoundary);
    }

    let payload: DiagnosticPayload = serde_json::from_str(sanitized)
        .map_err(|_| TriageParseError::MalformedJson)?;

    if !payload.proposed_remediation.confidence.is_finite()
        || payload.proposed_remediation.confidence < 0.0
        || payload.proposed_remediation.confidence > 1.0
    {
        return Err(TriageParseError::InvalidConfidenceRange);
    }

    Ok(payload)
}
