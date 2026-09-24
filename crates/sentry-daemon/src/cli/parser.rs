//! Command-line argument parser for systemd-sentry.
//!
//! Pure Rust parser conforming to POSIX argument conventions.

use crate::cli::args::{CliArgs, Command};
use crate::cli::exit_codes::{EX_OK, EX_USAGE};
use crate::cli::help::{format_global_help, format_subcommand_help};

/// Parse a slice of string arguments (skipping `argv[0]`) into `CliArgs`.
pub fn parse_cli_args(args: &[String]) -> Result<CliArgs, (String, i32)> {
    let mut config_path = None;
    let mut socket_path = None;
    let mut idx = 0;
    let len = args.len();

    // Check for global flags before subcommand
    while idx < len {
        let arg = &args[idx];
        if arg == "-h" || arg == "--help" {
            return Err((format_global_help(), EX_OK));
        } else if arg == "-V" || arg == "--version" {
            return Err((format!("systemd-sentry {}", env!("CARGO_PKG_VERSION")), EX_OK));
        } else if arg == "--setup" {
            return Ok(CliArgs { config_path, socket_path, command: Command::Setup });
        } else if (arg == "-c" || arg == "--config") && idx + 1 < len {
            idx += 1;
            config_path = Some(args[idx].clone());
            idx += 1;
        } else if (arg == "-s" || arg == "--socket") && idx + 1 < len {
            idx += 1;
            socket_path = Some(args[idx].clone());
            idx += 1;
        } else if arg.starts_with('-') {
            return Err((format!("Unknown option: {}\nRun 'systemd-sentry --help' for usage.", arg), EX_USAGE));
        } else {
            break;
        }
    }

    if idx >= len {
        return Ok(CliArgs { config_path, socket_path, command: Command::Daemon });
    }

    let subcmd = args[idx].as_str();
    idx += 1;

    if args[idx..].iter().any(|a| a == "-h" || a == "--help") {
        return Err((format_subcommand_help(subcmd), EX_OK));
    }

    let command = match subcmd {
        "daemon" => Command::Daemon,
        "status" => {
            let json = args[idx..].iter().any(|a| a == "--json");
            Command::Status { json }
        }
        "check" => {
            let verbose = args[idx..].iter().any(|a| a == "-v" || a == "--verbose");
            Command::Check { verbose }
        }
        "triage" => {
            if idx >= len {
                return Err(("Missing required argument <UNIT> for 'triage'.\nUsage: systemd-sentry triage <UNIT> [--json]".to_string(), EX_USAGE));
            }
            let unit = args[idx].clone();
            idx += 1;
            let json = args[idx..].iter().any(|a| a == "--json");
            Command::Triage { unit, json }
        }
        "monitor" => Command::Monitor,
        "incidents" => {
            let mut limit = 20;
            let mut json = false;
            let mut i = idx;
            while i < len {
                if args[i] == "--json" {
                    json = true;
                } else if args[i] == "--limit" && i + 1 < len {
                    i += 1;
                    limit = args[i].parse().unwrap_or(20);
                }
                i += 1;
            }
            Command::Incidents { limit, json }
        }
        "inspect" => {
            if idx >= len {
                return Err(("Missing required argument <ID> for 'inspect'.\nUsage: systemd-sentry inspect <ID> [--json]".to_string(), EX_USAGE));
            }
            let id = args[idx].clone();
            idx += 1;
            let json = args[idx..].iter().any(|a| a == "--json");
            Command::Inspect { id, json }
        }
        "reset" => {
            if idx >= len {
                return Err(("Missing required argument <UNIT> for 'reset'.\nUsage: systemd-sentry reset <UNIT>".to_string(), EX_USAGE));
            }
            let unit = args[idx].clone();
            Command::Reset { unit }
        }
        "mcp" => Command::Mcp,
        "completions" => {
            if idx >= len {
                return Err(("Missing required argument <SHELL> for 'completions'.\nUsage: systemd-sentry completions <bash|zsh|fish>".to_string(), EX_USAGE));
            }
            let shell = args[idx].clone();
            Command::Completions { shell }
        }
        "setup" => Command::Setup,
        "help" => {
            let sub = if idx < len { Some(args[idx].clone()) } else { None };
            let text = sub.as_deref().map(format_subcommand_help).unwrap_or_else(format_global_help);
            return Err((text, EX_OK));
        }
        other => {
            return Err((format!("Unknown subcommand '{}'.\nRun 'systemd-sentry --help' for usage.", other), EX_USAGE));
        }
    };

    Ok(CliArgs { config_path, socket_path, command })
}
