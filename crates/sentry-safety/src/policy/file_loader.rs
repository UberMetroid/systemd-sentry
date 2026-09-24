//! Safe, bounded TOML policy file reader.

use crate::policy::model::PolicyConfig;
use sentry_core::error::ConfigError;
use std::fs;
use std::path::Path;

/// Maximum permissible size for a policy TOML file (64 KiB) to prevent memory exhaustion attacks.
pub const MAX_POLICY_FILE_SIZE: u64 = 64 * 1024;

/// Loads and parses a single policy TOML file, enforcing strict file size bounds.
pub fn load_policy_file(path: &Path) -> Result<PolicyConfig, ConfigError> {
    let metadata = fs::metadata(path).map_err(ConfigError::Io)?;

    if metadata.len() > MAX_POLICY_FILE_SIZE {
        return Err(ConfigError::ParseError(format!(
            "Policy file size ({} bytes) at {} exceeds security limit of {} bytes",
            metadata.len(),
            path.display(),
            MAX_POLICY_FILE_SIZE
        )));
    }

    let content = fs::read_to_string(path).map_err(ConfigError::Io)?;

    toml::from_str::<PolicyConfig>(&content).map_err(|e| ConfigError::ParseError(format!(
        "Failed parsing policy TOML at {}: {e}",
        path.display()
    )))
}
