//! Parses watchdog configuration from systemd environment variables.

use super::error::NotifyError;
use std::env;
use std::process;
use std::time::Duration;

/// Configuration for the background watchdog heartbeat ticker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchdogConfig {
    /// Recommended ping interval: `raw_usec / 2`.
    pub interval: Duration,
    /// Raw watchdog timeout in microseconds from `$WATCHDOG_USEC`.
    pub raw_usec: u64,
    /// Target PID from `$WATCHDOG_PID` if specified.
    pub target_pid: Option<u32>,
}

/// Parses `$WATCHDOG_USEC` and `$WATCHDOG_PID`, returning `Some(WatchdogConfig)`
/// configured with `interval = WATCHDOG_USEC / 2`.
///
/// Returns `Ok(None)` if `$WATCHDOG_USEC` is unset, empty, or 0, or if `$WATCHDOG_PID`
/// does not match the current process.
pub fn parse_watchdog_config(unset_env: bool) -> Result<Option<WatchdogConfig>, NotifyError> {
    let usec_str = match env::var("WATCHDOG_USEC") {
        Ok(v) if !v.is_empty() => v,
        _ => return Ok(None),
    };

    let usec = usec_str.parse::<u64>().map_err(|e| {
        NotifyError::InvalidWatchdogInterval(usec_str.clone(), e)
    })?;

    if usec == 0 {
        return Ok(None);
    }

    let current_pid = process::id();
    let target_pid = if let Ok(pid_str) = env::var("WATCHDOG_PID") {
        let pid = pid_str.parse::<u32>().map_err(|e| {
            NotifyError::InvalidWatchdogPid(pid_str.clone(), e)
        })?;
        if pid != current_pid {
            return Ok(None);
        }
        Some(pid)
    } else {
        None
    };

    if unset_env {
        env::remove_var("WATCHDOG_USEC");
        env::remove_var("WATCHDOG_PID");
    }

    let interval = Duration::from_micros(usec / 2);

    Ok(Some(WatchdogConfig {
        interval,
        raw_usec: usec,
        target_pid,
    }))
}
