//! Subcommand implementations for systemd-sentry CLI.

pub mod cmd_check;
pub mod cmd_daemon;
pub mod cmd_incidents;
pub mod cmd_inspect;
pub mod cmd_mcp;
pub mod cmd_monitor;
pub mod cmd_reset;
pub mod cmd_setup;
pub mod cmd_status;
pub mod cmd_triage;

pub use cmd_check::execute_check;
pub use cmd_daemon::execute_daemon;
pub use cmd_incidents::execute_incidents;
pub use cmd_inspect::execute_inspect;
pub use cmd_mcp::execute_mcp;
pub use cmd_monitor::execute_monitor;
pub use cmd_reset::execute_reset;
pub use cmd_setup::execute_setup;
pub use cmd_status::execute_status;
pub use cmd_triage::execute_triage;
