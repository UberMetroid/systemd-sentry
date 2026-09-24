//! Unit tests for deterministic exit code triage.

use sentry_core::models::{RemediationAction, RiskLevel, Severity};
use sentry_diagnostic::fallback::triage_exit_code;

#[test]
fn test_triage_exit_137_oom() {
    let result = triage_exit_code(Some(137), None).expect("Matches OOM");
    assert_eq!(result.severity, Severity::High);
    assert_eq!(result.action, RemediationAction::RestartWithBackoff);
    assert_eq!(result.risk_level, RiskLevel::Medium);
    assert!(result.root_cause.summary.contains("Out-Of-Memory"));
}

#[test]
fn test_triage_sigkill() {
    let result = triage_exit_code(None, Some("SIGKILL")).expect("Matches SIGKILL");
    assert_eq!(result.severity, Severity::High);
    assert_eq!(result.action, RemediationAction::RestartWithBackoff);
}

#[test]
fn test_triage_exit_139_segfault() {
    let result = triage_exit_code(Some(139), None).expect("Matches segfault");
    assert_eq!(result.severity, Severity::Critical);
    assert_eq!(result.action, RemediationAction::EscalateToAdmin);
    assert_eq!(result.risk_level, RiskLevel::High);
    assert!(result.root_cause.summary.contains("Segmentation Fault"));
}

#[test]
fn test_triage_exit_134_sigabrt() {
    let result = triage_exit_code(Some(134), None).expect("Matches SIGABRT");
    assert_eq!(result.severity, Severity::Critical);
    assert_eq!(result.action, RemediationAction::EscalateToAdmin);
}

#[test]
fn test_triage_exit_203_exec() {
    let result = triage_exit_code(Some(203), None).expect("Matches EXIT_EXEC");
    assert_eq!(result.severity, Severity::Critical);
    assert_eq!(result.action, RemediationAction::NoAction);
    assert_eq!(result.risk_level, RiskLevel::High);
}

#[test]
fn test_triage_unknown_exit_code_returns_none() {
    assert!(triage_exit_code(Some(42), None).is_none());
}
