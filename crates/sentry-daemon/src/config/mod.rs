//! Configuration loader, defaults, and validation.

pub mod daemon_config;
pub mod loader;
pub mod validator;

pub use daemon_config::DaemonConfig;
pub use loader::{load_daemon_config, MAX_CONFIG_FILE_SIZE};
pub use validator::{validate_configuration, ValidationReport};
