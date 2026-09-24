//! 1:1 Unit QA tests for notify subsystem.

use std::sync::Mutex;

/// Process-wide lock to serialize tests that mutate notification environment variables.
pub static NOTIFY_ENV_LOCK: Mutex<()> = Mutex::new(());

pub mod test_encoder;
pub mod test_sender;
pub mod test_socket_addr;
pub mod test_state;
pub mod test_watchdog_config;
pub mod test_watchdog_ticker;
