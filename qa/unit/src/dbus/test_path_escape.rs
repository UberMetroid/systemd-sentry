//! 1:1 Unit QA tests for D-Bus unit path escaping.

use sentry_driver::dbus::{escape_unit_name, unit_name_to_object_path};

#[test]
fn test_escape_standard_unit_name() {
    assert_eq!(escape_unit_name("nginx.service"), "nginx_2eservice");
    assert_eq!(
        escape_unit_name("user@1000.service"),
        "user_401000_2eservice"
    );
    assert_eq!(escape_unit_name("app-worker.slice"), "app_2dworker_2eslice");
    assert_eq!(escape_unit_name("simple"), "simple");
}

#[test]
fn test_unit_name_to_object_path() {
    let path = unit_name_to_object_path("postgres.service").unwrap();
    assert_eq!(
        path.as_str(),
        "/org/freedesktop/systemd1/unit/postgres_2eservice"
    );
}
