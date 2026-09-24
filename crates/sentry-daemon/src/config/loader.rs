//! Configuration file loader with sensible defaults.

use crate::config::daemon_config::DaemonConfig;
use sentry_core::error::ConfigError;
use std::fs;
use std::path::Path;

/// Maximum size allowed for configuration file (64 KiB to prevent heap bloat).
pub const MAX_CONFIG_FILE_SIZE: u64 = 64 * 1024;

/// Load daemon configuration from path or default locations.
///
/// If no file exists at the given path, returns `DaemonConfig::default()`.
pub fn load_daemon_config(path: Option<&str>) -> Result<DaemonConfig, ConfigError> {
    let target_path = path.unwrap_or("/etc/systemd-sentry/config.toml");
    let file_path = Path::new(target_path);

    let mut config: DaemonConfig = if !file_path.exists() {
        DaemonConfig::default()
    } else {
        let metadata = fs::metadata(file_path).map_err(ConfigError::Io)?;

        if metadata.len() > MAX_CONFIG_FILE_SIZE {
            return Err(ConfigError::InvalidSetting {
                key: "file_size".to_string(),
                reason: format!(
                    "Config file {} exceeds maximum size limit of {} bytes",
                    target_path, MAX_CONFIG_FILE_SIZE
                ),
            });
        }

        let contents = fs::read_to_string(file_path).map_err(ConfigError::Io)?;
        toml::from_str(&contents).map_err(|e| ConfigError::ParseError(format!("{}: {}", target_path, e)))?
    };

    // Integrate with systemd-creds: discover decrypted credentials from $CREDENTIALS_DIRECTORY
    if config.provider.api_key.is_none() {
        if let Ok(creds_dir) = std::env::var("CREDENTIALS_DIRECTORY") {
            let openai_cred = Path::new(&creds_dir).join("openai_api_key");
            let generic_cred = Path::new(&creds_dir).join("api_key");
            if openai_cred.is_file() {
                if let Ok(key) = fs::read_to_string(openai_cred) {
                    config.provider.api_key = Some(key.trim().to_string());
                }
            } else if generic_cred.is_file() {
                if let Ok(key) = fs::read_to_string(generic_cred) {
                    config.provider.api_key = Some(key.trim().to_string());
                }
            }
        } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            if !key.trim().is_empty() {
                config.provider.api_key = Some(key.trim().to_string());
            }
        }
    }

    Ok(config)
}
