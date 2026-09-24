//! 1:1 Unit QA tests for coredump xattr reader.

use sentry_driver::coredump::read_coredump_xattrs;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_coredump_xattrs_file_not_found() {
    let dir = tempdir().unwrap();
    let missing_path = dir.path().join("missing.core");
    let err = read_coredump_xattrs(&missing_path).unwrap_err();
    assert!(err.to_string().contains("Coredump file not found"));
}

#[test]
fn test_read_coredump_xattrs_plain_file() {
    let dir = tempdir().unwrap();
    let core_file = dir.path().join("dummy.core");
    fs::write(&core_file, b"ELF...").unwrap();

    let xattrs = read_coredump_xattrs(&core_file).unwrap();
    // In test environment without extended attributes set, attributes default to None
    assert!(xattrs.pid.is_none());
    assert!(xattrs.signal.is_none());
    assert!(xattrs.comm.is_none());
    assert!(xattrs.exe.is_none());
    assert!(xattrs.unit.is_none());
}
