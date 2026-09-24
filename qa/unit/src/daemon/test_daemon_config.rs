//! 1:1 QA tests for configuration loader, validator, and systemd-creds integration.

use sentry_daemon::config::{
    load_daemon_config, validate_configuration, DaemonConfig, MAX_CONFIG_FILE_SIZE,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_default_daemon_config() {
    let config = DaemonConfig::default();
    assert_eq!(config.socket_path, "/run/systemd-sentry/sentry.sock");
    assert_eq!(config.rss_limit_mb, 15);
    assert_eq!(config.rss_degraded_mb, 13);
    assert_eq!(config.rss_recover_mb, 11);

    let report = validate_configuration(&config);
    assert!(report.is_valid);
}

#[test]
fn test_validator_rejects_inverted_hysteresis() {
    let mut config = DaemonConfig::default();
    config.rss_degraded_mb = 16; // degraded > limit (invalid)
    let report = validate_configuration(&config);
    assert!(!report.is_valid);

    let mut config2 = DaemonConfig::default();
    config2.rss_recover_mb = 14; // recover > degraded (invalid)
    let report2 = validate_configuration(&config2);
    assert!(!report2.is_valid);
}

#[test]
fn test_reject_oversized_config_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("oversized.toml");

    let giant_data = vec![b' '; (MAX_CONFIG_FILE_SIZE + 10) as usize];
    fs::write(&file_path, giant_data).unwrap();

    let result = load_daemon_config(Some(file_path.to_str().unwrap()));
    assert!(result.is_err());
}

#[test]
fn test_systemd_creds_directory_discovery() {
    let dir = tempdir().unwrap();
    let creds_dir = dir.path().join("credentials");
    fs::create_dir_all(&creds_dir).unwrap();

    let key_file = creds_dir.join("openai_api_key");
    fs::write(&key_file, "sk-systemd-creds-decrypted-secret\n").unwrap();

    let prev = std::env::var("CREDENTIALS_DIRECTORY").ok();
    std::env::set_var("CREDENTIALS_DIRECTORY", &creds_dir);

    let config = load_daemon_config(Some("/nonexistent/path.toml")).unwrap();
    assert_eq!(
        config.provider.api_key.as_deref(),
        Some("sk-systemd-creds-decrypted-secret")
    );

    match prev {
        Some(v) => std::env::set_var("CREDENTIALS_DIRECTORY", v),
        None => std::env::remove_var("CREDENTIALS_DIRECTORY"),
    }
}
