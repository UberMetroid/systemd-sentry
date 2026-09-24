//! 1:1 Unit QA tests for socket activation subsystem.

use std::sync::Mutex;

/// Process-wide lock to serialize tests that mutate socket activation environment variables.
pub static ACTIVATION_ENV_LOCK: Mutex<()> = Mutex::new(());

pub mod test_activation_parser;
pub mod test_fd_flags;
pub mod test_model;
pub mod test_name_parser;
pub mod test_pid_validator;
