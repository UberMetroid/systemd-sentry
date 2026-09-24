//! Command: On-demand root-cause triage of a systemd unit.

use crate::cli::exit_codes::EX_OK;
use crate::config::daemon_config::DaemonConfig;
use sentry_core::models::{IncidentContext, UnitFailedDetails};
use sentry_diagnostic::DiagnosticEngine;
use sentry_driver::cgroup::collect_cgroup_telemetry;
use sentry_driver::psi::collect_system_psi;

/// Execute the `triage` subcommand.
pub async fn execute_triage(unit: &str, json: bool, config: &DaemonConfig) -> i32 {
    let cgroup_root = std::path::Path::new("/sys/fs/cgroup");
    let proc_path = std::path::Path::new("/proc/pressure");
    let cgroup = collect_cgroup_telemetry(cgroup_root, unit).ok();
    let pressure = collect_system_psi(proc_path).ok();

    let failure_event = sentry_core::models::DriverEvent::UnitFailed(UnitFailedDetails {
        unit: unit.to_string(),
        active_state: "failed".to_string(),
        sub_state: "failed".to_string(),
        result: Some("exit-code".to_string()),
        exec_status: Some(1),
        main_pid: None,
    });

    let mut incident = IncidentContext::new(unit, failure_event);
    incident.cgroup = cgroup;
    incident.telemetry = pressure;

    let provider: std::sync::Arc<dyn sentry_diagnostic::LlmProvider> = std::sync::Arc::from(
        sentry_diagnostic::provider::create_provider(&config.provider),
    );
    let engine = DiagnosticEngine::new(Some(provider));
    let diagnostic = engine.diagnose(&incident).await;

    if json {
        println!("{}", serde_json::to_string_pretty(&diagnostic).unwrap_or_default());
    } else {
        println!("============================================================");
        println!(" DIAGNOSTIC TRIAGE REPORT: {}", unit);
        println!("============================================================");
        println!("Severity:     {:?}", diagnostic.severity);
        println!("Root Cause:   {}", diagnostic.root_cause.summary);
        println!("Details:      {}", diagnostic.root_cause.detail);
        println!("\nProposed Remediation:");
        println!("  Action:     {:?}", diagnostic.proposed_remediation.action);
        println!("  Confidence: {:.2}", diagnostic.proposed_remediation.confidence);
        println!("  Rationale:  {}", diagnostic.proposed_remediation.rationale);
        println!("============================================================");
    }

    EX_OK
}
