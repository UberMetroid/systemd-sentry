//! 1:1 Unit QA tests for cgroup unit path locator.

use sentry_driver::cgroup::locate_unit_cgroup;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_locate_unit_cgroup_standard() {
    let dir = tempdir().unwrap();
    let unit_cgroup = dir.path().join("system.slice/nginx.service");
    fs::create_dir_all(&unit_cgroup).unwrap();

    let resolved = locate_unit_cgroup(dir.path(), "nginx.service").unwrap();
    assert_eq!(resolved, unit_cgroup);
}

#[test]
fn test_locate_unit_cgroup_already_prefixed() {
    let dir = tempdir().unwrap();
    let unit_cgroup = dir.path().join("system.slice/custom.service");
    fs::create_dir_all(&unit_cgroup).unwrap();

    let resolved = locate_unit_cgroup(dir.path(), "system.slice/custom.service").unwrap();
    assert_eq!(resolved, unit_cgroup);
}

#[test]
fn test_locate_unit_cgroup_missing_fails() {
    let dir = tempdir().unwrap();
    let err = locate_unit_cgroup(dir.path(), "nonexistent.service").unwrap_err();
    assert!(err.to_string().contains("Cgroup not found for unit"));
}
