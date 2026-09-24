//! 1:1 Unit QA tests for PSI telemetry collector.

use sentry_driver::psi::{collect_cgroup_psi, collect_system_psi};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_collect_system_psi() {
    let dir = tempdir().unwrap();
    let content = "some avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
    fs::write(dir.path().join("cpu"), content).unwrap();
    fs::write(dir.path().join("memory"), content).unwrap();
    fs::write(dir.path().join("io"), content).unwrap();

    let telemetry = collect_system_psi(dir.path()).expect("Must collect system PSI");
    assert!(telemetry.unit.is_none());
    assert_eq!(telemetry.cpu.some.avg10, 0.0);
    assert_eq!(telemetry.memory.some.avg10, 0.0);
    assert_eq!(telemetry.io.some.avg10, 0.0);
}

#[test]
fn test_collect_cgroup_psi() {
    let dir = tempdir().unwrap();
    let content = "some avg10=3.50 avg60=2.00 avg300=1.00 total=12345\n";
    fs::write(dir.path().join("cpu.pressure"), content).unwrap();
    fs::write(dir.path().join("memory.pressure"), content).unwrap();
    fs::write(dir.path().join("io.pressure"), content).unwrap();

    let telemetry =
        collect_cgroup_psi(dir.path(), Some("api.service".into())).expect("Must collect cgroup PSI");
    assert_eq!(telemetry.unit.as_deref(), Some("api.service"));
    assert_eq!(telemetry.cpu.some.avg10, 3.50);
}
