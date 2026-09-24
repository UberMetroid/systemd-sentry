use chrono::Utc;
use sentry_core::models::*;
use std::str::FromStr;
use uuid::Uuid;

#[test]
fn test_severity_ordering_and_serde() {
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
    assert!(Severity::Critical.is_urgent());
    assert!(!Severity::Low.is_urgent());

    let json = serde_json::to_string(&Severity::High).unwrap();
    assert_eq!(json, "\"HIGH\"");
    let parsed: Severity = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, Severity::High);
    assert_eq!(Severity::from_str("critical").unwrap(), Severity::Critical);
}

#[test]
fn test_remediation_action_properties() {
    assert!(RemediationAction::Restart.is_active_modification());
    assert!(!RemediationAction::NoAction.is_active_modification());
    assert_eq!(RemediationAction::Restart.systemd_job_mode(), Some("replace"));
    assert_eq!(RemediationAction::NoAction.systemd_job_mode(), None);

    let json = serde_json::to_string(&RemediationAction::RestartWithBackoff).unwrap();
    assert_eq!(json, "\"RESTART_WITH_BACKOFF\"");
    let parsed: RemediationAction = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, RemediationAction::RestartWithBackoff);
}

#[test]
fn test_psi_and_pressure_telemetry() {
    let some = PsiLine::new(5.5, 3.2, 1.1, 1000000);
    assert!(some.is_stalled(5.0));
    assert!(!some.is_stalled(10.0));

    let record = PsiRecord::new(some.clone(), None);
    assert!(!record.is_fully_stalled(1.0));

    let telemetry = PressureTelemetry::new(
        Some("test.service".into()),
        record.clone(),
        record.clone(),
        record,
        1700000000,
    );
    assert!(telemetry.has_critical_pressure(5.0));
}

#[test]
fn test_cgroup_telemetry_oom_and_memory_util() {
    let mut telem = CgroupTelemetry {
        unit: Some("foo.service".into()),
        cgroup_path: "/system.slice/foo.service".into(),
        memory_current_bytes: Some(500),
        memory_max_bytes: Some(1000),
        memory_events: MemoryEvents::default(),
        cpu_stat: CpuStat::default(),
        io_stats: Vec::new(),
        populated: Some(true),
        frozen: Some(false),
        timestamp_usec: 123456,
        synthetic: false,
    };

    assert!(!telem.had_oom_kill());
    assert_eq!(telem.memory_utilization(), Some(0.5));

    telem.memory_events.oom_kill = 2;
    assert!(telem.had_oom_kill());
}

#[test]
fn test_coredump_record() {
    let record = CoredumpRecord::new("crash.service", 1234, 11, "SIGSEGV");
    assert!(record.is_memory_fault());
    assert_eq!(record.unit, "crash.service");
    assert_eq!(record.signal, 11);
}

#[test]
fn test_driver_event_unit_name() {
    let details = UnitFailedDetails {
        unit: "nginx.service".into(),
        active_state: "failed".into(),
        sub_state: "failed".into(),
        result: Some("exit-code".into()),
        exec_status: Some(1),
        main_pid: Some(42),
    };
    let event = DriverEvent::UnitFailed(details);
    assert_eq!(event.unit_name(), Some("nginx.service"));
}

#[test]
fn test_incident_context_builder() {
    let details = UnitFailedDetails {
        unit: "demo.service".into(),
        active_state: "failed".into(),
        sub_state: "failed".into(),
        result: None,
        exec_status: None,
        main_pid: None,
    };
    let ctx = IncidentContext::new("demo.service", DriverEvent::UnitFailed(details))
        .with_journal_lines(vec!["error: line 1".into()]);

    assert_eq!(ctx.unit, "demo.service");
    assert_eq!(ctx.journal_lines.len(), 1);
}

#[test]
fn test_diagnostic_payload_serde() {
    let payload = DiagnosticPayload {
        incident_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        unit_name: "demo.service".into(),
        root_cause: RootCause {
            summary: "Segfault in worker".into(),
            detail: "SIGSEGV at 0xdeadbeef".into(),
        },
        evidence: Evidence {
            journal_lines: vec!["Segmentation fault".into()],
            exit_code: Some(139),
            signal: Some("SIGSEGV".into()),
            coredump: None,
            psi: None,
            cgroup: None,
        },
        severity: Severity::High,
        proposed_remediation: ProposedRemediation {
            action: RemediationAction::RestartWithBackoff,
            rationale: "Restarting after crash".into(),
            risk_level: RiskLevel::Low,
            confidence: 0.95,
        },
    };

    let json = payload.to_json_pretty().unwrap();
    let deserialized = DiagnosticPayload::from_json_str(&json).unwrap();
    assert_eq!(deserialized.unit_name, payload.unit_name);
    assert_eq!(deserialized.severity, Severity::High);
}
