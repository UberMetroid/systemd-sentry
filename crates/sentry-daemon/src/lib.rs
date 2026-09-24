//! systemd-sentry supervisor daemon and CLI suite.
//!
//! Autonomous, zero-trust system supervisor written in 100% pure Rust
//! for systemd-managed Linux environments.
//!
//! Features:
//! - Multiplexed event loop monitoring system D-Bus and journal stream.
//! - Sliding-window circuit breaking and flap lockout.
//! - Guarded remediation strictly bounded by drop-in safety policies.
//! - Sub-15MB RSS memory footprint with zero-allocation process accounting.
//! - Pure Rust inter-process communication over UNIX domain sockets with `SO_PEERCRED`.
//! - Standard Unix sysexits exit codes and headless scriptability.

#![deny(missing_docs)]

pub mod cli;
pub mod commands;
pub mod config;
pub mod daemon;
pub mod ipc;
pub mod system;

use cli::{
    parse_cli_args, Command, EX_OK,
};
use commands::{
    execute_check, execute_daemon, execute_incidents, execute_inspect, execute_mcp,
    execute_monitor, execute_reset, execute_setup, execute_status, execute_triage,
};
use config::load_daemon_config;

/// Primary entrypoint executing systemd-sentry from an argument slice.
pub async fn entrypoint(args: &[String]) -> i32 {
    let cli_args = match parse_cli_args(args) {
        Ok(a) => a,
        Err((msg, code)) => {
            if code == EX_OK {
                println!("{}", msg);
            } else {
                eprintln!("{}", msg);
            }
            return code;
        }
    };

    let config = match load_daemon_config(cli_args.config_path.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration load error: {}", e);
            return cli::EX_CONFIG;
        }
    };

    let socket_path = cli_args
        .socket_path
        .as_deref()
        .unwrap_or(&config.socket_path);

    match cli_args.command {
        Command::Daemon => execute_daemon(config).await,
        Command::Status { json } => execute_status(socket_path, json).await,
        Command::Check { verbose } => execute_check(&config, verbose),
        Command::Triage { unit, json } => execute_triage(&unit, json, &config).await,
        Command::Monitor => execute_monitor(socket_path).await,
        Command::Incidents { limit, json } => execute_incidents(socket_path, limit, json).await,
        Command::Inspect { id, json } => execute_inspect(socket_path, &id, json).await,
        Command::Reset { unit } => execute_reset(socket_path, &unit).await,
        Command::Mcp => execute_mcp().await,
        Command::Completions { shell } => match cli::generate_completions(&shell) {
            Ok(script) => {
                print!("{}", script);
                EX_OK
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                cli::EX_USAGE
            }
        },
        Command::Setup => execute_setup().await,
        Command::Version => {
            println!("systemd-sentry {}", env!("CARGO_PKG_VERSION"));
            EX_OK
        }
        Command::Help { subcommand } => {
            let text = subcommand
                .as_deref()
                .map(cli::format_subcommand_help)
                .unwrap_or_else(cli::format_global_help);
            println!("{}", text);
            EX_OK
        }
    }
}
