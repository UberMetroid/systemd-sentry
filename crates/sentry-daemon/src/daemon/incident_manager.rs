//! Processes unit failure events, collects host telemetry, runs triage, and triggers remediation.

use crate::daemon::state::DaemonState;
use crate::ipc::protocol::IpcResponse;
use crate::system::RemediationExecutor;
use sentry_core::models::IncidentContext;
use sentry_diagnostic::fallback::DeterministicFallbackEngine;
use sentry_driver::cgroup::collect_cgroup_telemetry;
use sentry_driver::dbus::UnitFailedEvent;
use sentry_driver::psi::collect_system_psi;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Orchestrates telemetry collection, triage analysis, circuit breaking, and remediation.
pub struct IncidentManager;

impl IncidentManager {
    /// Process a unit failure event asynchronously.
    pub async fn handle_unit_failure(
        event: UnitFailedEvent,
        state: Arc<Mutex<DaemonState>>,
        remediation_executor: &RemediationExecutor,
    ) {
        let unit_name = event.unit.clone();
        info!("Processing failure incident for unit: {}", unit_name);

        // 1. Check load shedding status
        let is_degraded = {
            let s = state.lock().await;
            s.load_shedder.is_degraded()
        };

        // 2. Gather kernel telemetry
        let cgroup_root = std::path::Path::new("/sys/fs/cgroup");
        let proc_path = std::path::Path::new("/proc/pressure");
        let cgroup = collect_cgroup_telemetry(cgroup_root, &unit_name).ok();
        let pressure = collect_system_psi(proc_path).ok();

        // 3. Assemble incident context
        let details = sentry_core::models::UnitFailedDetails {
            unit: event.unit.clone(),
            active_state: event.active_state.clone(),
            sub_state: event.sub_state.clone(),
            result: event.result.clone(),
            exec_status: event.exec_status,
            main_pid: None,
        };
        let failure_event = sentry_core::models::DriverEvent::UnitFailed(details.clone());
        let mut incident = IncidentContext::new(&unit_name, failure_event);
        incident.cgroup = cgroup;
        incident.telemetry = pressure;

        // Connect systemd-coredump: scan for core dumps matching crashed unit
        let coredump_dir = std::path::Path::new("/var/lib/systemd/coredump");
        if details.result.as_deref() == Some("core-dump") || details.result.as_deref() == Some("signal") {
            let comm_prefix = unit_name.strip_suffix(".service").unwrap_or(&unit_name);
            incident.coredump = sentry_driver::coredump::find_latest_coredump(coredump_dir, Some(comm_prefix));
        }

        // 4. Perform triage: deterministic fallback if degraded, otherwise diagnostic engine
        let diagnostic = if is_degraded {
            info!("Load shedding active: performing deterministic fallback triage on {}", unit_name);
            DeterministicFallbackEngine::new().triage(&incident)
        } else {
            let engine = {
                let s = state.lock().await;
                s.diagnostic_engine.clone()
            };
            engine.diagnose(&incident).await
        };

        // 5. Update circuit breaker state and evaluate lockout
        let (breaker_state_label, action_allowed) = {
            let mut s = state.lock().await;
            let breaker_state = s.circuit_registry.record_failure(&unit_name, std::time::Instant::now());
            let allowed = breaker_state.allows_remediation();
            (breaker_state.label(), allowed)
        };

        info!(
            "Unit {} circuit state: {}, action allowed: {}",
            unit_name, breaker_state_label, action_allowed
        );

        // 6. Execute remediation if circuit permits and action is active modification
        if action_allowed && diagnostic.proposed_remediation.action.is_active_modification() {
            match remediation_executor.execute(&unit_name, &diagnostic.proposed_remediation).await {
                Ok(msg) => info!("Remediation succeeded for {}: {}", unit_name, msg),
                Err(err) => warn!("Remediation failed for {}: {}", unit_name, err),
            }
        } else if !action_allowed {
            warn!("Remediation skipped for {}: circuit breaker locked out", unit_name);
        }

        // 7. Store incident in MCP state and broadcast
        let event_payload = serde_json::json!({
            "incident_id": incident.incident_id,
            "unit": unit_name,
            "severity": diagnostic.severity,
            "root_cause": diagnostic.root_cause.summary,
            "action": diagnostic.proposed_remediation.action,
            "circuit_state": breaker_state_label,
        });

        let s = state.lock().await;
        s.mcp_state.record_incident(diagnostic);
        let _ = s.event_broadcaster.send(IpcResponse::Event { data: event_payload });
    }
}
