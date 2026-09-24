//! Daemon configuration structure and defaults.

use sentry_diagnostic::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Global configuration for the systemd-sentry supervisor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Path to UNIX domain socket for IPC communication.
    pub socket_path: String,
    /// Path to primary safety policy TOML file.
    pub policy_path: String,
    /// Path to directory containing drop-in policy files.
    pub policy_dropin_dir: String,
    /// Absolute ceiling for resident set size (RSS) in megabytes before shutdown.
    pub rss_limit_mb: usize,
    /// Threshold in megabytes where load-shedding degraded mode is engaged.
    pub rss_degraded_mb: usize,
    /// Threshold in megabytes where degraded mode is cleared.
    pub rss_recover_mb: usize,
    /// Maximum number of recent incidents retained in memory.
    pub max_incident_history: usize,
    /// Logging verbosity level.
    pub log_level: String,
    /// Diagnostic LLM provider configuration.
    pub provider: ProviderConfig,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: "/run/systemd-sentry/sentry.sock".to_string(),
            policy_path: "/etc/systemd-sentry/policy.toml".to_string(),
            policy_dropin_dir: "/etc/systemd-sentry/policy.d".to_string(),
            rss_limit_mb: 15,
            rss_degraded_mb: 13,
            rss_recover_mb: 11,
            max_incident_history: 100,
            log_level: "info".to_string(),
            provider: ProviderConfig::default(),
        }
    }
}
