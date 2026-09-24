//! Formatted CLI help text for systemd-sentry.
//!
//! Provides clean, standard Unix manual-style help output.

/// Format the global help message.
pub fn format_global_help() -> String {
    r#"systemd-sentry - Autonomous zero-trust system supervisor for systemd

USAGE:
    systemd-sentry [OPTIONS] [SUBCOMMAND]
    sentry [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -c, --config <PATH>    Path to configuration file [default: /etc/systemd-sentry/config.toml]
    -s, --socket <PATH>    Path to UNIX domain socket [default: /run/systemd-sentry/sentry.sock]
        --setup            Launch the interactive configuration setup wizard
    -h, --help             Print help information
    -V, --version          Print version information

SUBCOMMANDS:
    daemon                 Run the supervisor daemon (default if no subcommand given)
    status [--json]        Query runtime status, tracked units, and circuit states
    check [--verbose]      Validate configuration and drop-in policies (exit 0 or 78)
    triage <unit> [--json] On-demand root-cause triage of a systemd unit
    monitor                Stream live crash, trip, and remediation events
    incidents [--limit N]  List recent incident journal slices and summaries
    inspect <id> [--json]  Inspect detailed diagnostic for an incident UUID
    reset <unit>           Reset circuit breaker lockout for a unit (requires root)
    mcp                    Run Model Context Protocol (MCP) server on stdio
    completions <shell>    Generate shell completions (bash, zsh, fish)
    setup                  Interactive configuration and credential wizard

EXAMPLES:
    # Run interactive setup wizard
    systemd-sentry --setup

    # Query supervisor status
    systemd-sentry status

    # On-demand triage for a failing unit
    systemd-sentry triage nginx.service

    # Stream real-time failure events
    systemd-sentry monitor

    # Reset circuit breaker for an unlocked service
    sudo systemd-sentry reset redis.service
"#
    .to_string()
}

/// Format help message for a specific subcommand.
pub fn format_subcommand_help(subcommand: &str) -> String {
    match subcommand {
        "status" => "USAGE:\n    systemd-sentry status [--json]\n\nQuery runtime status, tracked units, and circuit states.\n".to_string(),
        "check" => "USAGE:\n    systemd-sentry check [--verbose]\n\nValidate configuration and drop-in policies without launching daemon. Exits 0 on success or 78 (EX_CONFIG) on failure.\n".to_string(),
        "triage" => "USAGE:\n    systemd-sentry triage <UNIT> [--json]\n\nOn-demand root-cause triage of a systemd unit using local/cloud LLM or deterministic fallback rules.\n".to_string(),
        "monitor" => "USAGE:\n    systemd-sentry monitor\n\nStream live crash, trip, and remediation events from the running supervisor.\n".to_string(),
        "incidents" => "USAGE:\n    systemd-sentry incidents [--limit <N>] [--json]\n\nList recent incident journal slices and summaries from memory storage.\n".to_string(),
        "inspect" => "USAGE:\n    systemd-sentry inspect <ID> [--json]\n\nInspect detailed diagnostic and journal slice for an incident UUID.\n".to_string(),
        "reset" => "USAGE:\n    systemd-sentry reset <UNIT>\n\nReset circuit breaker lockout for a unit. Requires root (UID 0) or sentry user.\n".to_string(),
        "mcp" => "USAGE:\n    systemd-sentry mcp\n\nRun Model Context Protocol (MCP) server over standard input/output (stdio).\n".to_string(),
        "completions" => "USAGE:\n    systemd-sentry completions <bash|zsh|fish>\n\nGenerate shell completions for the specified shell.\n".to_string(),
        "setup" => "USAGE:\n    systemd-sentry setup (or systemd-sentry --setup)\n\nInteractive wizard for discovering local LLMs, testing connectivity, and configuring sentry.\n".to_string(),
        _ => format_global_help(),
    }
}
