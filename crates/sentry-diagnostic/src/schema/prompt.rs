//! Prompt representation for LLM diagnostic inference.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sentry_core::models::{CgroupTelemetry, IncidentContext, PressureTelemetry};

/// Structured diagnostic prompt constructed from system telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticPrompt {
    /// Target service unit name.
    pub unit_name: String,
    /// Process exit code if available.
    pub exit_code: Option<i32>,
    /// Terminating signal name (e.g. "SIGSEGV", "SIGKILL").
    pub signal: Option<String>,
    /// Recent journal log lines.
    pub journal_lines: Vec<String>,
    /// Extracted crash stack trace if available.
    pub coredump_trace: Option<String>,
    /// Subsystem pressure telemetry (PSI).
    pub psi_stats: Option<PressureTelemetry>,
    /// Cgroup resource stats if available.
    pub cgroup_stats: Option<CgroupTelemetry>,
    /// Current or recorded active state.
    pub active_state: String,
    /// Current or recorded sub state.
    pub sub_state: String,
    /// Incident timestamp.
    pub timestamp: DateTime<Utc>,
}

impl DiagnosticPrompt {
    /// System prompt instructions for LLM diagnostic models.
    pub const SYSTEM_PROMPT: &'static str =
        "You are systemd-sentry, an autonomous Linux diagnostic agent. \
Analyze the provided systemd service failure telemetry and output a single valid JSON object \
strictly adhering to the following schema: \
{\
  \"incident_id\": \"00000000-0000-0000-0000-000000000000\",\
  \"timestamp\": \"<ISO8601-utc>\",\
  \"unit_name\": \"<target-service-unit>\",\
  \"root_cause\": {\
    \"summary\": \"<one-sentence summary of failure>\",\
    \"detail\": \"<detailed technical analysis of logs, signals, or pressure>\"\
  },\
  \"evidence\": {\
    \"journal_lines\": [\"<relevant journal lines>\"],\
    \"exit_code\": <numeric exit code or null>,\
    \"signal\": <signal string or null>\
  },\
  \"severity\": \"LOW|MEDIUM|HIGH|CRITICAL\",\
  \"proposed_remediation\": {\
    \"action\": \"NO_ACTION|RESTART|RESTART_WITH_BACKOFF|RELOAD|RESET_FAILED|ESCALATE_TO_ADMIN\",\
    \"rationale\": \"<technical justification for action>\",\
    \"risk_level\": \"LOW|MEDIUM|HIGH\",\
    \"confidence\": <number between 0.0 and 1.0>\
  }\
} \
Do not include markdown code fences, commentary, or text outside the JSON object.";

    /// Constructs a prompt from an `IncidentContext`.
    pub fn from_incident_context(ctx: &IncidentContext) -> Self {
        let (active_state, sub_state, exit_code, signal) = match &ctx.failure_event {
            sentry_core::models::DriverEvent::UnitFailed(details) => (
                details.active_state.clone(),
                details.sub_state.clone(),
                details.exec_status,
                details.result.clone(),
            ),
            _ => ("failed".to_string(), "failed".to_string(), None, None),
        };

        let coredump_trace = ctx
            .coredump
            .as_ref()
            .and_then(|c| c.stack_trace.clone());

        let signal = if let Some(c) = &ctx.coredump {
            Some(c.signal_name.clone())
        } else {
            signal
        };

        Self {
            unit_name: ctx.unit.clone(),
            exit_code,
            signal,
            journal_lines: ctx.journal_lines.clone(),
            coredump_trace,
            psi_stats: ctx.telemetry.clone(),
            cgroup_stats: ctx.cgroup.clone(),
            active_state,
            sub_state,
            timestamp: ctx.timestamp,
        }
    }

    /// Serializes telemetry to a prompt string.
    pub fn to_prompt_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}
