//! 1:1 Unit QA tests for cgroup CPU statistics reader.

use sentry_driver::cgroup::read_cgroup_cpu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_cgroup_cpu_standard() {
    let dir = tempdir().unwrap();
    let content = "\
usage_usec 1500000
user_usec 1000000
system_usec 500000
nr_periods 100
nr_throttled 5
throttled_usec 25000
";
    fs::write(dir.path().join("cpu.stat"), content).unwrap();

    let stats = read_cgroup_cpu(dir.path()).unwrap();
    assert_eq!(stats.usage_usec, 1500000);
    assert_eq!(stats.user_usec, 1000000);
    assert_eq!(stats.system_usec, 500000);
    assert_eq!(stats.nr_periods, 100);
    assert_eq!(stats.nr_throttled, 5);
    assert_eq!(stats.throttled_usec, 25000);
}

#[test]
fn test_read_cgroup_cpu_missing_defaults() {
    let dir = tempdir().unwrap();
    let stats = read_cgroup_cpu(dir.path()).unwrap();
    assert_eq!(stats.usage_usec, 0);
    assert_eq!(stats.nr_throttled, 0);
}
