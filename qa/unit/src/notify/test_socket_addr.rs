//! 1:1 Unit QA tests for notify socket address resolution.

use sentry_driver::notify::resolve_notify_address;
use tempfile::tempdir;

#[test]
fn test_resolve_abstract_socket_address() {
    let raw = "@sentry_unit_test_socket";
    let addr = resolve_notify_address(raw).expect("Must resolve abstract socket");
    assert!(addr.as_pathname().is_none());
}

#[test]
fn test_resolve_filesystem_socket_address() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("notify.sock");
    let raw = socket_path.to_str().unwrap();

    let addr = resolve_notify_address(raw).expect("Must resolve filesystem socket");
    assert_eq!(addr.as_pathname(), Some(socket_path.as_path()));
}

#[test]
fn test_resolve_empty_socket_address_fails() {
    let err = resolve_notify_address("").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Path cannot be empty"));
}
