//! Policy data models for zero-trust declarative remediation boundaries.

use sentry_core::models::RemediationAction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Declarative policy specification defining allowed actions and protected units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Global policies applicable across all managed units.
    #[serde(default)]
    pub global: GlobalPolicy,
    /// Specific per-unit policy overrides.
    #[serde(default)]
    pub units: HashMap<String, UnitPolicyOverride>,
}

/// Global remediation policy parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalPolicy {
    /// Critical system units that sentry must never touch under any circumstances.
    #[serde(default)]
    pub protected_units: Vec<String>,
    /// Allowed remediation actions globally approved for execution.
    #[serde(default)]
    pub allowed_actions: Vec<RemediationAction>,
    /// Default cooldown between remediations in seconds.
    #[serde(default = "default_cooldown")]
    pub cooldown_seconds: u64,
}

fn default_cooldown() -> u64 {
    300
}

impl Default for GlobalPolicy {
    fn default() -> Self {
        Self {
            protected_units: Vec::new(),
            allowed_actions: vec![
                RemediationAction::NoAction,
                RemediationAction::RestartWithBackoff,
                RemediationAction::Reload,
                RemediationAction::ResetFailed,
                RemediationAction::EscalateToAdmin,
            ],
            cooldown_seconds: 300,
        }
    }
}

/// Per-unit policy overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UnitPolicyOverride {
    /// Explicitly mark this unit as protected from any automatic intervention.
    pub protected: Option<bool>,
    /// Allowed remediation actions specifically authorized for this unit.
    pub allowed_actions: Option<Vec<RemediationAction>>,
    /// Hard override forcing a specific action regardless of LLM advisory.
    pub force_action: Option<RemediationAction>,
}
