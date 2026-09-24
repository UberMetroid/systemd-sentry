//! 1:1 Unit QA tests for D-Bus unit path unescaping.

use sentry_driver::dbus::{object_path_to_unit_name, unescape_unit_name};

#[test]
fn test_unescape_standard_unit_name() {
    assert_eq!(
        unescape_unit_name("nginx_2eservice").unwrap(),
        "nginx.service"
    );
    assert_eq!(
        unescape_unit_name("user_401000_2eservice").unwrap(),
        "user@1000.service"
    );
    assert_eq!(
        unescape_unit_name("app_2dworker_2eslice").unwrap(),
        "app-worker.slice"
    );
}

#[test]
fn test_object_path_to_unit_name() {
    let path = "/org/freedesktop/systemd1/unit/docker_2eservice";
    let unit = object_path_to_unit_name(path).unwrap();
    assert_eq!(unit, "docker.service");
}

#[test]
fn test_unescape_corrupted_hex_fails() {
    // Truncated escape sequence
    assert!(unescape_unit_name("foo_2").is_err());
    // Non-hex digits
    assert!(unescape_unit_name("foo_zz").is_err());
}

#[test]
fn test_object_path_missing_prefix_fails() {
    let path = "/org/other/path/foo";
    assert!(object_path_to_unit_name(path).is_err());
}
