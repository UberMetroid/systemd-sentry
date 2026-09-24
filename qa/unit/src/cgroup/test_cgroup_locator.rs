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
fn test_locate_unit_cgroup_flat_container() {
    let dir = tempdir().unwrap();
    let flat_cgroup = dir.path().join("container-app.service");
    fs::create_dir_all(&flat_cgroup).unwrap();

    let resolved = locate_unit_cgroup(dir.path(), "container-app.service").unwrap();
    assert_eq!(resolved, flat_cgroup);
}

#[test]
fn test_locate_unit_cgroup_user_slice() {
    let dir = tempdir().unwrap();
    let user_cgroup = dir.path().join("user.slice/user-app.service");
    fs::create_dir_all(&user_cgroup).unwrap();

    let resolved = locate_unit_cgroup(dir.path(), "user-app.service").unwrap();
    assert_eq!(resolved, user_cgroup);
}

#[test]
fn test_locate_unit_cgroup_docker_container() {
    let dir = tempdir().unwrap();
    let docker_cgroup = dir.path().join("docker/my-container.scope");
    fs::create_dir_all(&docker_cgroup).unwrap();

    let resolved = locate_unit_cgroup(dir.path(), "my-container.scope").unwrap();
    assert_eq!(resolved, docker_cgroup);
}

#[test]
fn test_locate_unit_cgroup_missing_fails() {
    let dir = tempdir().unwrap();
    let err = locate_unit_cgroup(dir.path(), "nonexistent.service").unwrap_err();
    assert!(err.to_string().contains("Cgroup not found for unit"));
}
