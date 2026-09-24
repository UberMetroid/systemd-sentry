//! Pure Rust remediation executor using systemd D-Bus.
//!
//! Strictly verifies proposed actions against PolicyGatekeeper before execution.

use sentry_core::models::{ProposedRemediation, RemediationAction};
use sentry_safety::policy::PolicyGatekeeper;
use tracing::{error, info, warn};
use zbus::Connection;

/// Executes authorized remediations against systemd units.
pub struct RemediationExecutor {
    gatekeeper: PolicyGatekeeper,
}

impl RemediationExecutor {
    /// Create a new executor wrapping the active policy gatekeeper.
    pub fn new(gatekeeper: PolicyGatekeeper) -> Self {
        Self { gatekeeper }
    }

    /// Update the internal policy gatekeeper (e.g. during SIGHUP reload).
    pub fn update_gatekeeper(&mut self, gatekeeper: PolicyGatekeeper) {
        self.gatekeeper = gatekeeper;
    }

    /// Execute a remediation action if permitted by safety policy.
    pub async fn execute(
        &self,
        unit: &str,
        remediation: &ProposedRemediation,
    ) -> Result<String, String> {
        // 1. Validate through zero-trust policy gatekeeper
        if let Err(e) = self.gatekeeper.validate(unit, remediation.action) {
            warn!("Remediation for unit {} blocked by policy: {}", unit, e);
            return Err(format!("Blocked by policy: {}", e));
        }

        // 2. Connect to system D-Bus and dispatch typed manager call
        let connection = Connection::system().await.map_err(|e| {
            format!("Failed to connect to system D-Bus: {}", e)
        })?;

        match remediation.action {
            RemediationAction::Restart | RemediationAction::RestartWithBackoff => {
                info!("Executing systemd RestartUnit on {}", unit);
                let reply: zbus::Result<zbus::zvariant::OwnedObjectPath> = connection
                    .call_method(
                        Some("org.freedesktop.systemd1"),
                        "/org/freedesktop/systemd1",
                        Some("org.freedesktop.systemd1.Manager"),
                        "RestartUnit",
                        &(unit, "replace"),
                    )
                    .await
                    .map_err(|e| format!("D-Bus call RestartUnit failed: {}", e))?
                    .body()
                    .deserialize();

                match reply {
                    Ok(path) => Ok(format!("Dispatched RestartUnit: {}", path.as_str())),
                    Err(e) => {
                        error!("RestartUnit failed for {}: {}", unit, e);
                        Err(format!("D-Bus call RestartUnit failed: {}", e))
                    }
                }
            }
            RemediationAction::ResetFailed => {
                info!("Executing systemd ResetFailedUnit on {}", unit);
                let reply: zbus::Result<()> = connection
                    .call_method(
                        Some("org.freedesktop.systemd1"),
                        "/org/freedesktop/systemd1",
                        Some("org.freedesktop.systemd1.Manager"),
                        "ResetFailedUnit",
                        &(unit,),
                    )
                    .await
                    .map_err(|e| format!("D-Bus call ResetFailedUnit failed: {}", e))?
                    .body()
                    .deserialize();

                match reply {
                    Ok(_) => Ok(format!("ResetFailedUnit succeeded for {}", unit)),
                    Err(e) => {
                        error!("ResetFailedUnit failed for {}: {}", unit, e);
                        Err(format!("D-Bus call ResetFailedUnit failed: {}", e))
                    }
                }
            }
            RemediationAction::Reload => {
                info!("Executing systemd ReloadUnit on {}", unit);
                let reply: zbus::Result<zbus::zvariant::OwnedObjectPath> = connection
                    .call_method(
                        Some("org.freedesktop.systemd1"),
                        "/org/freedesktop/systemd1",
                        Some("org.freedesktop.systemd1.Manager"),
                        "ReloadUnit",
                        &(unit, "replace"),
                    )
                    .await
                    .map_err(|e| format!("D-Bus call ReloadUnit failed: {}", e))?
                    .body()
                    .deserialize();

                match reply {
                    Ok(path) => Ok(format!("Dispatched ReloadUnit: {}", path.as_str())),
                    Err(e) => {
                        error!("ReloadUnit failed for {}: {}", unit, e);
                        Err(format!("D-Bus call ReloadUnit failed: {}", e))
                    }
                }
            }
            RemediationAction::NoAction | RemediationAction::EscalateToAdmin => {
                info!("EscalateToAdmin / NoAction for {}: no automatic remediation taken", unit);
                Ok("No automatic action taken; observation only".to_string())
            }
        }
    }
}
