//! 1:1 Unit QA tests for cgroup IO statistics reader.

use sentry_driver::cgroup::read_cgroup_io;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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

#[test]
fn test_read_cgroup_io_invalid_device_formats_skipped() {
    let dir = tempdir().unwrap();
    let content = "\
invalid_device_no_colon rbytes=100
8:notanumber rbytes=200
:999 rbytes=300
8: rbytes=400
8:0 rbytes=1024 wbytes=2048
";
    fs::write(dir.path().join("io.stat"), content).unwrap();

    let stats = read_cgroup_io(dir.path()).unwrap();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].device, "8:0");
    assert_eq!(stats[0].rbytes, 1024);
}

#[test]
#[cfg(unix)]
fn test_read_cgroup_io_permission_denied_returns_empty() {
    let dir = tempdir().unwrap();
    let io_file = dir.path().join("io.stat");
    fs::write(&io_file, "259:0 rbytes=1048576\n").unwrap();
    fs::set_permissions(&io_file, fs::Permissions::from_mode(0o000)).unwrap();

    let stats = read_cgroup_io(dir.path()).expect("Must not error on EACCES");
    assert!(stats.is_empty());

    fs::set_permissions(&io_file, fs::Permissions::from_mode(0o644)).unwrap();
}
