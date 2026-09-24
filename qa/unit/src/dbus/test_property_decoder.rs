//! 1:1 Unit QA tests for D-Bus property decoding and failure detection.

use sentry_driver::dbus::{decode_unit_properties, extract_unit_failed_event, UnitStateUpdate};
use std::collections::HashMap;
use zbus::zvariant::Value;

#[test]
fn test_decode_unit_properties_success() {
    let mut changed = HashMap::new();
    changed.insert("ActiveState".into(), Value::from("active").try_to_owned().unwrap());
    changed.insert("SubState".into(), Value::from("running").try_to_owned().unwrap());
    changed.insert("MainPID".into(), Value::from(1001u32).try_to_owned().unwrap());
    changed.insert("Result".into(), Value::from("success").try_to_owned().unwrap());

    let update = decode_unit_properties(
        "my.service".into(),
        "org.freedesktop.systemd1.Unit",
        &changed,
    )
    .expect("Must decode properties");

    assert_eq!(update.unit, "my.service");
    assert_eq!(update.active_state.as_deref(), Some("active"));
    assert_eq!(update.sub_state.as_deref(), Some("running"));
    assert_eq!(update.main_pid, Some(1001));

    assert!(extract_unit_failed_event(&update).is_none());
}

#[test]
fn test_extract_unit_failed_event_on_failure() {
    let update = UnitStateUpdate {
        unit: "crashed.service".into(),
        active_state: Some("failed".into()),
        sub_state: Some("failed".into()),
        result: Some("core-dump".into()),
        exec_main_code: Some(11),
        exec_main_status: Some(139),
        main_pid: Some(4096),
        cgroup: None,
    };

    let failed = extract_unit_failed_event(&update).expect("Must detect failure");
    assert_eq!(failed.unit, "crashed.service");
    assert_eq!(failed.active_state, "failed");
    assert_eq!(failed.result.as_deref(), Some("core-dump"));
    assert_eq!(failed.exec_code, Some(11));
    assert_eq!(failed.exec_status, Some(139));
}

#[test]
fn test_decode_ignored_interface_returns_none() {
    let changed = HashMap::new();
    let res = decode_unit_properties("test.service".into(), "org.other.Interface", &changed);
    assert!(res.is_none());
}
