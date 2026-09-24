//! CLI argument and command representations for systemd-sentry.
//!
//! Strongly typed representation of parsed command-line invocations.

/// Supported subcommands for systemd-sentry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Run the persistent supervisor daemon.
    Daemon,
    /// Query and display daemon runtime status and circuit breaker states.
    Status {
        /// Format output as machine-readable JSON.
        json: bool,
    },
    /// Validate configuration and drop-in policies without launching daemon.
    Check {
        /// Print detailed rule-by-rule verification report.
        verbose: bool,
    },
    /// On-demand root-cause triage of a specific systemd unit.
    Triage {
        /// Name of the systemd unit (e.g. `nginx.service`).
        unit: String,
        /// Format triage diagnostic as JSON.
        json: bool,
    },
    /// Stream live crash, trip, and remediation events from the daemon.
    Monitor,
    /// Query recent incident journal slices and triage history.
    Incidents {
        /// Maximum number of incidents to return (default: 20).
        limit: usize,
        /// Format output as machine-readable JSON.
        json: bool,
    },
    /// Inspect the detailed diagnostic and journal slice of a specific incident ID.
    Inspect {
        /// Incident UUID.
        id: String,
        /// Format output as machine-readable JSON.
        json: bool,
    },
    /// Manually reset circuit breaker lockout for a unit (requires root or sentry user).
    Reset {
        /// Name of the systemd unit to unlock.
        unit: String,
    },
    /// Run the Model Context Protocol (MCP) server over standard input/output.
    Mcp,
    /// Generate shell auto-completion scripts.
    Completions {
        /// Target shell (`bash`, `zsh`, `fish`).
        shell: String,
    },
    /// Interactive configuration and connection setup wizard.
    Setup,
    /// Display version information.
    Version,
    /// Display help information.
    Help {
        /// Specific subcommand help topic, if any.
        subcommand: Option<String>,
    },
}

/// Parsed global CLI arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    /// Path to primary configuration file (default: `/etc/systemd-sentry/config.toml`).
    pub config_path: Option<String>,
    /// Path to UNIX domain socket (default: `/run/systemd-sentry/sentry.sock`).
    pub socket_path: Option<String>,
    /// Subcommand to execute.
    pub command: Command,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            config_path: None,
            socket_path: None,
            command: Command::Daemon,
        }
    }
}
