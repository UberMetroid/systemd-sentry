//! Logind driver error types.

use std::path::PathBuf;
use thiserror::Error;

/// Logind session discovery error variants.
#[derive(Error, Debug)]
pub enum LogindError {
    /// Pure Rust zbus error communicating with logind.
    #[error("zbus communication error with systemd-logind: {0}")]
    Zbus(#[from] zbus::Error),

    /// Session object path not found.
    #[error("Session '{0}' not found")]
    SessionNotFound(String),

    /// User session D-Bus socket path does not exist.
    #[error("User D-Bus socket not found for UID {0} at {1}")]
    UserBusNotFound(u32, PathBuf),

    /// Standard filesystem I/O error.
    #[error("I/O error verifying logind user bus: {0}")]
    Io(#[from] std::io::Error),
}
