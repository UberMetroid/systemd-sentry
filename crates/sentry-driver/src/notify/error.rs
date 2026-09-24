//! Notification subsystem error types.

use std::io;
use thiserror::Error;

/// Notification errors.
#[derive(Error, Debug)]
pub enum NotifyError {
    /// Standard I/O error during notification socket write.
    #[error("I/O error during notification: {0}")]
    Io(#[from] io::Error),

    /// Invalid NOTIFY_SOCKET address format or parsing failure.
    #[error("Invalid NOTIFY_SOCKET address '{0}': {1}")]
    InvalidSocketAddress(String, String),

    /// Status string exceeds maximum permitted length.
    #[error("Status string exceeds maximum permitted length ({0} > 1024 bytes)")]
    StatusTooLong(usize),

    /// Failed to parse WATCHDOG_USEC.
    #[error("Failed to parse WATCHDOG_USEC '{0}': {1}")]
    InvalidWatchdogInterval(String, std::num::ParseIntError),

    /// Failed to parse WATCHDOG_PID.
    #[error("Failed to parse WATCHDOG_PID '{0}': {1}")]
    InvalidWatchdogPid(String, std::num::ParseIntError),
}
