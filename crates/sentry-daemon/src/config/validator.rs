//! Configuration and policy pre-flight validator.

use crate::config::daemon_config::DaemonConfig;
use sentry_safety::policy::load_policy_with_dropins;

/// Result of configuration and policy validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// True if all checks passed cleanly.
    pub is_valid: bool,
    /// Informational warnings or issues encountered.
    pub messages: Vec<String>,
}

/// Perform exhaustive pre-flight validation on daemon and safety configurations.
pub fn validate_configuration(config: &DaemonConfig) -> ValidationReport {
    let mut messages = Vec::new();
    let mut is_valid = true;

    // 1. Validate RSS hysteresis constraints
    if config.rss_limit_mb == 0 {
        messages.push("rss_limit_mb must be greater than 0".to_string());
        is_valid = false;
    }
    if config.rss_degraded_mb >= config.rss_limit_mb {
        messages.push(format!(
            "rss_degraded_mb ({}) must be strictly less than rss_limit_mb ({})",
            config.rss_degraded_mb, config.rss_limit_mb
        ));
        is_valid = false;
    }
    if config.rss_recover_mb >= config.rss_degraded_mb {
        messages.push(format!(
            "rss_recover_mb ({}) must be strictly less than rss_degraded_mb ({})",
            config.rss_recover_mb, config.rss_degraded_mb
        ));
        is_valid = false;
    }

    // 2. Validate incident history limit
    if config.max_incident_history == 0 || config.max_incident_history > 1000 {
        messages.push("max_incident_history must be between 1 and 1000".to_string());
        is_valid = false;
    }

    // 3. Validate policy and drop-in parsing
    let base_path = std::path::Path::new(&config.policy_path);
    let dropin_dir = std::path::Path::new(&config.policy_dropin_dir);
    let policy = load_policy_with_dropins(base_path, dropin_dir);
    messages.push(format!(
        "Policy verified: {} unit override(s), {} protected unit(s)",
        policy.units.len(),
        policy.global.protected_units.len()
    ));

    // 4. Validate socket path format
    if !config.socket_path.starts_with('/') {
        messages.push("socket_path must be an absolute filesystem path".to_string());
        is_valid = false;
    }

    ValidationReport { is_valid, messages }
}
