//! 1:1 Unit QA tests for LISTEN_PID validation.

use sentry_driver::activation::validate_listen_pid;
use std::env;
use std::process;

#[test]
fn test_validate_listen_pid_matching() {
    env::set_var("LISTEN_PID", process::id().to_string());
    assert!(validate_listen_pid().unwrap());
    env::remove_var("LISTEN_PID");
}

#[test]
fn test_validate_listen_pid_mismatching() {
    let wrong_pid = process::id() + 42424;
    env::set_var("LISTEN_PID", wrong_pid.to_string());
    assert!(!validate_listen_pid().unwrap());
    env::remove_var("LISTEN_PID");
}

#[test]
fn test_validate_listen_pid_unset() {
    env::remove_var("LISTEN_PID");
    assert!(!validate_listen_pid().unwrap());
}

#[test]
fn test_validate_listen_pid_invalid_integer() {
    env::set_var("LISTEN_PID", "invalid_pid");
    let err = validate_listen_pid().unwrap_err();
    assert!(err.to_string().contains("Failed to parse LISTEN_PID"));
    env::remove_var("LISTEN_PID");
}
