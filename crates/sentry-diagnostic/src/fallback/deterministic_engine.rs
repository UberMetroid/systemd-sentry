//! Deterministic rule-based fallback triage engine.

use crate::fallback::exit_codes::triage_exit_code;
use crate::fallback::journal_patterns::scan_journal_patterns;
use sentry_core::models::{
    DiagnosticPayload, Evidence, IncidentContext, ProposedRemediation, RemediationAction,
    RiskLevel, RootCause, Severity,
};

/// Deterministic fallback triage engine operating on Linux kernel and systemd heuristics.
#[derive(Debug, Default, Clone)]
pub struct DeterministicFallbackEngine;

impl DeterministicFallbackEngine {
    /// Constructs a new fallback engine.
    pub fn new() -> Self {
        Self
    }

    /// Synthesizes a valid `DiagnosticPayload` from ground truth subsystem telemetry.
    pub fn triage(&self, ctx: &IncidentContext) -> DiagnosticPayload {
        let (exit_code, signal) = match &ctx.failure_event {
            sentry_core::models::DriverEvent::UnitFailed(details) => {
                (details.exec_status, details.result.as_deref())
            }
            _ => (None, None),
        };

        let coredump_trace = ctx
            .coredump
            .as_ref()
            .and_then(|c| c.stack_trace.clone());

        let signal_name = ctx
            .coredump
            .as_ref()
            .map(|c| c.signal_name.as_str())
            .or(signal);

        // Priority 1: High-fidelity journal signatures
        if let Some(journal_match) = scan_journal_patterns(&ctx.journal_lines) {
            return self.build_payload(
                ctx,
                journal_match.root_cause,
                journal_match.severity,
                journal_match.action,
                journal_match.risk_level,
                journal_match.confidence_permille as f32 / 1000.0,
                exit_code,
                signal_name,
                coredump_trace,
            );
        }

        // Priority 2: Process exit code and signal
        if let Some(exit_match) = triage_exit_code(exit_code, signal_name) {
            return self.build_payload(
                ctx,
                exit_match.root_cause,
                exit_match.severity,
                exit_match.action,
                exit_match.risk_level,
                exit_match.confidence_permille as f32 / 1000.0,
                exit_code,
                signal_name,
                coredump_trace,
            );
        }

        // Priority 3: Cgroup memory limit / OOM check
        if let Some(cgroup) = &ctx.cgroup {
            if cgroup.had_oom_kill() {
                return self.build_payload(
                    ctx,
                    RootCause {
                        summary: "Cgroup OOM kill event recorded".to_string(),
                        detail: format!(
                            "Cgroup memory events recorded {} OOM kills in {}.",
                            cgroup.memory_events.oom_kill, cgroup.cgroup_path
                        ),
                    },
                    Severity::High,
                    RemediationAction::RestartWithBackoff,
                    RiskLevel::Medium,
                    0.90,
                    exit_code,
                    signal_name,
                    coredump_trace,
                );
            }

            if let (Some(cur), Some(max)) = (cgroup.memory_current_bytes, cgroup.memory_max_bytes) {
                if cur >= max {
                    return self.build_payload(
                        ctx,
                        RootCause {
                            summary: "Cgroup memory limit reached (cgroup v2 memory.max)".to_string(),
                            detail: format!(
                                "Current memory usage {}B reached or exceeded cgroup maximum {}B in {}.",
                                cur, max, cgroup.cgroup_path
                            ),
                        },
                        Severity::High,
                        RemediationAction::RestartWithBackoff,
                        RiskLevel::Medium,
                        0.85,
                        exit_code,
                        signal_name,
                        coredump_trace,
                    );
                }
            }
        }

        // Priority 4: Default generic fallback
        self.build_payload(
            ctx,
            RootCause {
                summary: format!("Unit {} entered failed state without explicit signature", ctx.unit),
                detail: "Subsystem failed without recognized crash signal or journal pattern. Safe backoff restart recommended.".to_string(),
            },
            Severity::Medium,
            RemediationAction::RestartWithBackoff,
            RiskLevel::Medium,
            0.30,
            exit_code,
            signal_name,
            coredump_trace,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build_payload(
        &self,
        ctx: &IncidentContext,
        root_cause: RootCause,
        severity: Severity,
        action: RemediationAction,
        risk_level: RiskLevel,
        confidence: f32,
        exit_code: Option<i32>,
        signal: Option<&str>,
        coredump: Option<String>,
    ) -> DiagnosticPayload {
        DiagnosticPayload {
            incident_id: ctx.incident_id,
            timestamp: ctx.timestamp,
            unit_name: ctx.unit.clone(),
            root_cause,
            evidence: Evidence {
                journal_lines: ctx.journal_lines.clone(),
                exit_code,
                signal: signal.map(|s| s.to_string()),
                coredump,
                psi: ctx.telemetry.clone(),
                cgroup: ctx.cgroup.clone(),
            },
            severity,
            proposed_remediation: ProposedRemediation {
                action,
                rationale: format!(
                    "Deterministic fallback engine evaluated telemetry and prescribed action {:?}",
                    action
                ),
                risk_level,
                confidence,
            },
        }
    }
}
