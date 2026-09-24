//! 1:1 Unit QA tests for user D-Bus address resolution.

use sentry_driver::logind::{resolve_user_bus_address, resolve_user_bus_address_with_base};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolve_user_bus_without_existence_check() {
    let path = resolve_user_bus_address(1000, false).unwrap();
    assert_eq!(path.to_str().unwrap(), "/run/user/1000/bus");
}

#[test]
fn test_resolve_user_bus_with_base_and_existence() {
    let dir = tempdir().unwrap();
    let user_dir = dir.path().join("user/1000");
    fs::create_dir_all(&user_dir).unwrap();
    let bus_file = user_dir.join("bus");
    fs::write(&bus_file, b"").unwrap();

    let path = resolve_user_bus_address_with_base(dir.path(), 1000, true).unwrap();
    assert_eq!(path, bus_file);
}

#[test]
fn test_resolve_user_bus_missing_fails_when_required() {
    let dir = tempdir().unwrap();
    let err = resolve_user_bus_address_with_base(dir.path(), 1000, true).unwrap_err();
    assert!(err.to_string().contains("User D-Bus socket not found"));
}
