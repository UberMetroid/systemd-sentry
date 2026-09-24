//! 1:1 Unit QA tests for cgroup IO statistics reader.

use sentry_driver::cgroup::read_cgroup_io;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_cgroup_io_standard() {
    let dir = tempdir().unwrap();
    let content = "259:0 rbytes=1048576 wbytes=2097152 rios=256 wios=512 dbytes=0 dios=0\n";
    fs::write(dir.path().join("io.stat"), content).unwrap();

    let stats = read_cgroup_io(dir.path()).unwrap();
    assert_eq!(stats.len(), 1);
    let dev = &stats[0];
    assert_eq!(dev.device, "259:0");
    assert_eq!(dev.rbytes, 1048576);
    assert_eq!(dev.wbytes, 2097152);
    assert_eq!(dev.rios, 256);
    assert_eq!(dev.wios, 512);
}

#[test]
fn test_read_cgroup_io_missing_file_returns_empty() {
    let dir = tempdir().unwrap();
    let stats = read_cgroup_io(dir.path()).unwrap();
    assert!(stats.is_empty());
}
