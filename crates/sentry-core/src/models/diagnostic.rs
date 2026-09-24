//! Diagnostic payload schema matching requirements R3 & F16.

use crate::models::cgroup::CgroupTelemetry;
use crate::models::psi::PressureTelemetry;
use crate::models::remediation::RemediationAction;
use crate::models::severity::Severity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identified root cause analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootCause {
    /// Concise single-line summary of failure cause.
    pub summary: String,
    /// Detailed diagnostic reasoning.
    pub detail: String,
}

/// Evidentiary data backing the diagnosis.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Evidence {
    /// Crucial journal lines leading up to failure.
    pub journal_lines: Vec<String>,
    /// Process exit code if available.
    pub exit_code: Option<i32>,
    /// Terminating signal name (e.g. "SIGSEGV").
    pub signal: Option<String>,
    /// Top extracted coredump backtrace lines.
    pub coredump: Option<String>,
    /// PSI telemetry reading at incident time.
    pub psi: Option<PressureTelemetry>,
    /// Cgroup metrics at incident time.
    pub cgroup: Option<CgroupTelemetry>,
}

/// Risk level associated with proposed remediation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    /// Low operational risk (e.g. reload or restart of isolated worker).
    Low,
    /// Medium operational risk (temporary interruption).
    Medium,
    /// High operational risk (data loss potential or cascading failure).
    High,
}

/// Proposed remediation step with confidence scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedRemediation {
    /// Bounded action proposal.
    pub action: RemediationAction,
    /// Rationale explaining why action was chosen.
    pub rationale: String,
    /// Risk assessment.
    pub risk_level: RiskLevel,
    /// Model confidence in recommendation (0.0 to 1.0).
    pub confidence: f32,
}

/// Complete incident diagnostic payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticPayload {
    /// Unique incident UUID v4.
    pub incident_id: Uuid,
    /// Timestamp of diagnostic generation.
    pub timestamp: DateTime<Utc>,
    /// Target service unit name.
    pub unit_name: String,
    /// Root cause determination.
    pub root_cause: RootCause,
    /// Collected evidence.
    pub evidence: Evidence,
    /// Assessed severity level.
    pub severity: Severity,
    /// Proposed safe remediation.
    pub proposed_remediation: ProposedRemediation,
}

impl DiagnosticPayload {
    /// Serializes payload to formatted JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes payload from JSON string.
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
