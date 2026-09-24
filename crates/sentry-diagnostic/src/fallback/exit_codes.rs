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

    // 1. Check exit code 137, raw signal 9 (SIGKILL), or systemd-oomd Result=oom-kill
    if exit_code == Some(137)
        || exit_code == Some(9)
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

    // 2. Check exit code 139, raw signal 11, or SIGSEGV (Segmentation Fault)
    if exit_code == Some(139)
        || exit_code == Some(11)
        || sig_upper.as_deref() == Some("SIGSEGV")
        || sig_upper.as_deref() == Some("SEGV")
    {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Segmentation Fault: invalid memory read or write (SIGSEGV)".to_string(),
                detail: "Process crashed due to unauthorized memory access or null pointer dereference (exit code 139 / SIGSEGV).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 3. Check exit code 134, raw signal 6, or SIGABRT (Process Abort / Panic)
    if exit_code == Some(134)
        || exit_code == Some(6)
        || sig_upper.as_deref() == Some("SIGABRT")
        || sig_upper.as_deref() == Some("ABRT")
    {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Process Abort: internal assertion failure or panic (SIGABRT)".to_string(),
                detail: "Process terminated abnormally via std::abort or panic handler (exit code 134 / SIGABRT).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 900,
        });
    }

    // 4. Check exit code 132, raw signal 4, or SIGILL (Illegal Instruction)
    if exit_code == Some(132)
        || exit_code == Some(4)
        || sig_upper.as_deref() == Some("SIGILL")
        || sig_upper.as_deref() == Some("ILL")
    {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Illegal Instruction: processor executed invalid opcode (SIGILL)".to_string(),
                detail: "Process executed an illegal instruction or unsupported CPU instruction set (exit code 132 / SIGILL).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 5. Check exit code 135, raw signal 7, or SIGBUS (Bus Error)
    if exit_code == Some(135)
        || exit_code == Some(7)
        || sig_upper.as_deref() == Some("SIGBUS")
        || sig_upper.as_deref() == Some("BUS")
    {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Bus Error: non-existent physical address or unaligned memory access (SIGBUS)".to_string(),
                detail: "Process crashed due to bus error or page fault on truncated mmap (exit code 135 / SIGBUS).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 6. Check exit code 136, raw signal 8, or SIGFPE (Floating Point Exception)
    if exit_code == Some(136)
        || exit_code == Some(8)
        || sig_upper.as_deref() == Some("SIGFPE")
        || sig_upper.as_deref() == Some("FPE")
    {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Floating Point Exception: arithmetic fault or division by zero (SIGFPE)".to_string(),
                detail: "Process crashed due to an unhandled arithmetic exception (exit code 136 / SIGFPE).".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 7. Check systemd watchdog timeout
    if sig_upper.as_deref() == Some("WATCHDOG") {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Service failed systemd watchdog heartbeat keep-alive (WatchdogSec timeout)".to_string(),
                detail: "Service did not send WATCHDOG=1 heartbeat within configured WatchdogSec window; terminated by systemd.".to_string(),
            },
            severity: Severity::High,
            action: RemediationAction::RestartWithBackoff,
            risk_level: RiskLevel::Medium,
            confidence_permille: 950,
        });
    }

    // 8. Check systemd start limit reached
    if sig_upper.as_deref() == Some("START-LIMIT-HIT") || sig_upper.as_deref() == Some("START_LIMIT_HIT") {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Unit reached systemd start rate limit (StartLimitBurst exceeded)".to_string(),
                detail: "Unit failed repeatedly within StartLimitIntervalSec window; systemd refused further automatic start attempts.".to_string(),
            },
            severity: Severity::Critical,
            action: RemediationAction::EscalateToAdmin,
            risk_level: RiskLevel::High,
            confidence_permille: 950,
        });
    }

    // 9. Check systemd execution timeout
    if sig_upper.as_deref() == Some("TIMEOUT") {
        return Some(ExitCodeTriageResult {
            root_cause: RootCause {
                summary: "Service execution timed out (TimeoutSec exceeded)".to_string(),
                detail: "systemd terminated unit because startup, shutdown, or runtime exceeded configured timeout window.".to_string(),
            },
            severity: Severity::High,
            action: RemediationAction::RestartWithBackoff,
            risk_level: RiskLevel::Medium,
            confidence_permille: 900,
        });
    }

    // 10. Check exit code 203 (systemd EXIT_EXEC)
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

    // 11. Check exit code 1
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
