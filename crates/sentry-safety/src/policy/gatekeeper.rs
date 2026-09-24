//! Zero-trust policy gatekeeper guarding system execution against unauthorized remediation.

use crate::policy::model::PolicyConfig;
use sentry_core::error::SafetyError;
use sentry_core::models::RemediationAction;

/// Zero-trust gatekeeper evaluating proposed LLM advisories against declarative policy rules.
#[derive(Debug, Clone)]
pub struct PolicyGatekeeper {
    policy: PolicyConfig,
}

impl PolicyGatekeeper {
    /// Constructs a gatekeeper wrapping the active policy configuration.
    pub fn new(policy: PolicyConfig) -> Self {
        Self { policy }
    }

    /// Returns a reference to the active policy configuration.
    pub fn policy(&self) -> &PolicyConfig {
        &self.policy
    }

    /// Evaluates a proposed remediation action for the specified unit against policy.
    ///
    /// - Checks global protected units list (blacklist).
    /// - Checks per-unit overrides and forced actions.
    /// - Validates against global allowed actions.
    ///
    /// Returns the approved (or forced) `RemediationAction` on success, or `SafetyError` if denied.
    pub fn validate(
        &self,
        unit: &str,
        proposed: RemediationAction,
    ) -> Result<RemediationAction, SafetyError> {
        // Safe passive actions (NoAction, EscalateToAdmin) are always allowed through
        if !proposed.is_active_modification() {
            return Ok(proposed);
        }

        // Check global protected units blacklist
        if self.policy.global.protected_units.iter().any(|u| u == unit) {
            return Err(SafetyError::UnitBlacklisted(unit.to_string()));
        }

        // Check per-unit overrides
        if let Some(unit_override) = self.policy.units.get(unit) {
            if unit_override.protected == Some(true) {
                return Err(SafetyError::UnitBlacklisted(unit.to_string()));
            }

            if let Some(forced) = unit_override.force_action {
                return Ok(forced);
            }

            if let Some(allowed) = &unit_override.allowed_actions {
                if !allowed.contains(&proposed) {
                    return Err(SafetyError::ActionDisallowed {
                        action: proposed.to_string(),
                        unit: unit.to_string(),
                        reason: "Action is not in the unit's allowed_actions list".to_string(),
                    });
                }
                return Ok(proposed);
            }
        }

        // Check global allowed actions
        if !self.policy.global.allowed_actions.contains(&proposed) {
            return Err(SafetyError::ActionDisallowed {
                action: proposed.to_string(),
                unit: unit.to_string(),
                reason: "Action is not permitted by global policy".to_string(),
            });
        }

        Ok(proposed)
    }
}
