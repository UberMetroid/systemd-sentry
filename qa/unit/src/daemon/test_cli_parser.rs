//! 1:1 QA tests for CLI argument parsing and exit codes.

use sentry_daemon::cli::{parse_cli_args, Command, EX_OK, EX_USAGE};

#[test]
fn test_parse_default_daemon() {
    let args = vec![];
    let parsed = parse_cli_args(&args).expect("Default args should parse to Daemon");
    assert_eq!(parsed.command, Command::Daemon);
    assert_eq!(parsed.config_path, None);
    assert_eq!(parsed.socket_path, None);
}

#[test]
fn test_parse_global_flags() {
    let args = vec![
        "-c".to_string(),
        "/etc/sentry.toml".to_string(),
        "-s".to_string(),
        "/run/sentry.sock".to_string(),
        "status".to_string(),
        "--json".to_string(),
    ];
    let parsed = parse_cli_args(&args).expect("Should parse global flags");
    assert_eq!(parsed.config_path.as_deref(), Some("/etc/sentry.toml"));
    assert_eq!(parsed.socket_path.as_deref(), Some("/run/sentry.sock"));
    assert_eq!(parsed.command, Command::Status { json: true });
}

#[test]
fn test_parse_subcommands() {
    // Check
    let args = vec!["check".to_string(), "--verbose".to_string()];
    let parsed = parse_cli_args(&args).unwrap();
    assert_eq!(parsed.command, Command::Check { verbose: true });

    // Triage
    let args = vec!["triage".to_string(), "nginx.service".to_string(), "--json".to_string()];
    let parsed = parse_cli_args(&args).unwrap();
    assert_eq!(
        parsed.command,
        Command::Triage {
            unit: "nginx.service".to_string(),
            json: true,
        }
    );

    // Incidents
    let args = vec!["incidents".to_string(), "--limit".to_string(), "50".to_string()];
    let parsed = parse_cli_args(&args).unwrap();
    assert_eq!(parsed.command, Command::Incidents { limit: 50, json: false });

    // Reset
    let args = vec!["reset".to_string(), "redis.service".to_string()];
    let parsed = parse_cli_args(&args).unwrap();
    assert_eq!(parsed.command, Command::Reset { unit: "redis.service".to_string() });

    // Mcp
    let args = vec!["mcp".to_string()];
    let parsed = parse_cli_args(&args).unwrap();
    assert_eq!(parsed.command, Command::Mcp);
}

#[test]
fn test_parse_unknown_options_and_missing_arguments() {
    // Unknown option
    let args = vec!["--unknown-flag".to_string()];
    let err = parse_cli_args(&args).unwrap_err();
    assert_eq!(err.1, EX_USAGE);

    // Missing triage unit
    let args = vec!["triage".to_string()];
    let err = parse_cli_args(&args).unwrap_err();
    assert_eq!(err.1, EX_USAGE);

    // Missing completions shell
    let args = vec!["completions".to_string()];
    let err = parse_cli_args(&args).unwrap_err();
    assert_eq!(err.1, EX_USAGE);
}

#[test]
fn test_help_and_version() {
    let args = vec!["--help".to_string()];
    let res = parse_cli_args(&args).unwrap_err();
    assert_eq!(res.1, EX_OK);
    assert!(res.0.contains("systemd-sentry"));

    let args = vec!["--version".to_string()];
    let res = parse_cli_args(&args).unwrap_err();
    assert_eq!(res.1, EX_OK);
    assert!(res.0.contains("systemd-sentry"));
}
