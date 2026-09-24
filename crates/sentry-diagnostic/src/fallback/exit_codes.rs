//! Deterministic exit code and signal triage rules.

use sentry_core::models::{RemediationAction, RiskLevel, RootCause, Severity};

/// Classification result from analyzing exit codes and signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitCodeTriageResult {
    /// Categorized root cause.
    pub root_cause: RootCause,
    /// Assessed severity.
    pub severity: Severity,
    /// Bounded remediation proposal.
    pub action: RemediationAction,
    /// Operational risk level.
    pub risk_level: RiskLevel,
    /// Confidence scaled from 0.0 to 1.0 (stored as thousandths e.g. 950 = 0.95).
    pub confidence_permille: u32,
}

/// Evaluates process exit code and terminating signal name.
pub fn triage_exit_code(exit_code: Option<i32>, signal: Option<&str>) -> Option<ExitCodeTriageResult> {
    let sig_upper = signal.map(|s| s.trim().to_uppercase());

    // 1. Check exit code 137, SIGKILL, or systemd-oomd Result=oom-kill
    if exit_code == Some(137)
        || sig_upper.as_deref() == Some("SIGKILL")
        || sig_upper.as_deref() == Some("KILL")
        || sig_upper.as_deref() == Some("OOM-KILL")
        || sig_upper.as_deref() == Some("OOM_KILL")
    {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Process terminated by Out-Of-Memory killer (SIGKILL / systemd-oomd)".to_string(),
                detail: "Kernel invoked OOM killer or systemd-oomd terminated unit due to cgroup memory/PSI pressure exhaustion.".to_string(),
            },
            severity: Severity::High,
            action: RemediationAction::RestartWithBackoff,
            risk_level: RiskLevel::Medium,
            confidence_permille: 950,
        });
    }

    // 2. Check exit code 139 or SIGSEGV (Segmentation Fault)
    if exit_code == Some(139) || sig_upper.as_deref() == Some("SIGSEGV") || sig_upper.as_deref() == Some("SEGV") {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Segmentation Fault: invalid memory read or write (SIGSEGV)".to_string(),
                detail: "Process crashed due to unauthorized memory access or null pointer dereference (exit code 139).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 3. Check exit code 134 or SIGABRT (Process Abort / Panic)
    if exit_code == Some(134) || sig_upper.as_deref() == Some("SIGABRT") || sig_upper.as_deref() == Some("ABRT") {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Process Abort: internal assertion failure or panic (SIGABRT)".to_string(),
                detail: "Process terminated abnormally via std::abort or panic handler (exit code 134).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 900,
        });
    }

    // 4. Check exit code 203 (systemd EXIT_EXEC)
    if exit_code == Some(203) {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Exec Failure: executable missing or invalid dynamic linker (EXIT_EXEC)".to_string(),
                detail: "systemd could not spawn process (exit code 203). Binary is missing, lacks execute permissions, or dynamic interpreter is absent.".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::NoAction,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 5. Check exit code 1
    if exit_code == Some(1) {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "General process error (exit code 1)".to_string(),
                detail: "Process exited with unhandled application error status 1.".to_string(),
            },
            severity: Severity::Medium,
            action: RemediationAction::RestartWithBackoff,
            risk_level: RiskLevel::Medium,
            confidence_permille: 600,
        });
    }

    None
}
