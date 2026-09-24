//! 1:1 Unit QA tests for D-Bus match rule formatting.

use sentry_driver::dbus::{build_manager_match_rule, build_unit_properties_match_rule};

#[test]
fn test_build_manager_match_rule() {
    let rule = build_manager_match_rule();
    let rule_str = rule.to_string();
    assert!(rule_str.contains("sender='org.freedesktop.systemd1'"));
    assert!(rule_str.contains("interface='org.freedesktop.systemd1.Manager'"));
    assert!(rule_str.contains("type='signal'"));
}

#[test]
fn test_build_unit_properties_match_rule() {
    let rule = build_unit_properties_match_rule();
    let rule_str = rule.to_string();
    assert!(rule_str.contains("sender='org.freedesktop.systemd1'"));
    assert!(rule_str.contains("interface='org.freedesktop.DBus.Properties'"));
    assert!(rule_str.contains("member='PropertiesChanged'"));
    assert!(rule_str.contains("path_namespace='/org/freedesktop/systemd1/unit'"));
}
