//! Deterministic pattern matching over systemd journal log lines.

use sentry_core::models::{RemediationAction, RiskLevel, RootCause, Severity};

/// Matched signature from journal analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalMatchResult {
    /// Categorized root cause.
    pub root_cause: RootCause,
    /// Assessed severity.
    pub severity: Severity,
    /// Bounded remediation proposal.
    pub action: RemediationAction,
    /// Operational risk level.
    pub risk_level: RiskLevel,
    /// Confidence scaled from 0.0 to 1.0 (stored as thousandths).
    pub confidence_permille: u32,
}

/// Scans journal lines for known fatal system signatures.
pub fn scan_journal_patterns(lines: &[String]) -> Option<JournalMatchResult> {
    for line in lines.iter().rev() {
        let lower = line.to_lowercase();

        // 1. OOM Killer signatures
        if lower.contains("out of memory:")
            || lower.contains("invoked oom-killer")
            || lower.contains("memory cgroup out of memory")
            || (lower.contains("killed process") && lower.contains("score"))
        {
            return Some(JournalMatchResult {
                root_cause: RootCause {
                    summary: "Kernel OOM killer invoked due to cgroup memory exhaustion".to_string(),
                    detail: format!("Journal records OOM condition: '{line}'"),
                },
                severity: Severity::High,
                action: RemediationAction::RestartWithBackoff,
                risk_level: RiskLevel::Medium,
                confidence_permille: 950,
            });
        }

        // 2. Segfault / Memory Fault signatures
        if lower.contains("segfault at")
            || lower.contains("general protection fault")
            || lower.contains("invalid opcode")
        {
            return Some(JournalMatchResult {
                root_cause: RootCause {
                    summary: "Memory corruption or segmentation fault in process".to_string(),
                    detail: format!("Journal records memory fault: '{line}'"),
                },
                severity: Severity::Critical,
                action: RemediationAction::EscalateToAdmin,
                risk_level: RiskLevel::High,
                confidence_permille: 950,
            });
        }

        // 3. Port / Bind conflict signatures
        if lower.contains("address already in use") || lower.contains("eaddrinuse") {
            return Some(JournalMatchResult {
                root_cause: RootCause {
                    summary: "Network socket port conflict (EADDRINUSE)".to_string(),
                    detail: format!("Journal records bind collision: '{line}'"),
                },
                severity: Severity::High,
                action: RemediationAction::RestartWithBackoff,
                risk_level: RiskLevel::Medium,
                confidence_permille: 900,
            });
        }

        // 4. Exec / Spawn failures
        if lower.contains("failed at step exec spawning")
            || lower.contains("no such file or directory")
        {
            return Some(JournalMatchResult {
                root_cause: RootCause {
                    summary: "Exec spawn failure: binary or dependency not found".to_string(),
                    detail: format!("Journal records exec failure: '{line}'"),
                },
                severity: Severity::Critical,
                action: RemediationAction::NoAction,
                risk_level: RiskLevel::High,
                confidence_permille: 950,
            });
        }

        // 5. Unhandled Panic / Assertion
        if lower.contains("panic") || lower.contains("assertion failed") {
            return Some(JournalMatchResult {
                root_cause: RootCause {
                    summary: "Process panic or assertion failure".to_string(),
                    detail: format!("Journal records panic or assertion failure: '{line}'"),
                },
                severity: Severity::Critical,
                action: RemediationAction::EscalateToAdmin,
                risk_level: RiskLevel::High,
                confidence_permille: 900,
            });
        }

        // 6. Disk space / File system read-only
        if lower.contains("no space left on device") || lower.contains("read-only file system") {
            return Some(JournalMatchResult {
                root_cause: RootCause {
                    summary: "Storage system exhaustion or read-only filesystem".to_string(),
                    detail: format!("Journal records filesystem failure: '{line}'"),
                },
                severity: Severity::Critical,
                action: RemediationAction::NoAction,
                risk_level: RiskLevel::High,
                confidence_permille: 950,
            });
        }
    }

    None
}
