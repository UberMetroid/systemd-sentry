//! 1:1 Unit QA tests for cgroup memory telemetry reader.

use sentry_driver::cgroup::{read_cgroup_memory, read_cgroup_memory_stats};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn test_read_cgroup_memory_with_integer_max() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("memory.current"), b"10485760\n").unwrap();
    fs::write(dir.path().join("memory.max"), b"20971520\n").unwrap();
    fs::write(
        dir.path().join("memory.events"),
        b"low 1\nhigh 2\nmax 3\noom 4\noom_kill 5\noom_group_kill 6\n",
    )
    .unwrap();

    let (current, max, events) = read_cgroup_memory(dir.path()).unwrap();
    assert_eq!(current, Some(10485760));
    assert_eq!(max, Some(20971520));
    assert_eq!(events.low, 1);
    assert_eq!(events.high, 2);
    assert_eq!(events.max, 3);
    assert_eq!(events.oom, 4);
    assert_eq!(events.oom_kill, 5);
    assert_eq!(events.oom_group_kill, 6);
}

#[test]
fn test_read_cgroup_memory_with_max_string() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("memory.current"), b"5000\n").unwrap();
    fs::write(dir.path().join("memory.max"), b"max\n").unwrap();

    let (current, max, _) = read_cgroup_memory(dir.path()).unwrap();
    assert_eq!(current, Some(5000));
    assert_eq!(max, None);

    let stats = read_cgroup_memory_stats(dir.path()).unwrap();
    assert_eq!(stats.current, 5000);
    assert_eq!(stats.max, None);
}

#[test]
#[cfg(unix)]
fn test_read_cgroup_memory_permission_denied_defaults() {
    let dir = tempdir().unwrap();
    let mem_file = dir.path().join("memory.current");
    fs::write(&mem_file, b"1048576\n").unwrap();
    fs::set_permissions(&mem_file, fs::Permissions::from_mode(0o000)).unwrap();

    let (current, max, _) = read_cgroup_memory(dir.path()).expect("Must not error on EACCES");
    assert_eq!(current, None);
    assert_eq!(max, None);

    fs::set_permissions(&mem_file, fs::Permissions::from_mode(0o644)).unwrap();
}
