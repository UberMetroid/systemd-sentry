//! 1:1 Unit QA tests for PSI file reader.

use sentry_driver::psi::read_psi_file;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_psi_file_success() {
    let dir = tempdir().unwrap();
    let psi_path = dir.path().join("memory");
    let content = "some avg10=0.10 avg60=0.05 avg300=0.01 total=999\n";
    fs::write(&psi_path, content).unwrap();

    let record = read_psi_file(&psi_path).unwrap();
    assert_eq!(record.some.avg10, 0.10);
    assert_eq!(record.some.total_usec, 999);
}

#[test]
fn test_read_psi_file_missing_fails() {
    let dir = tempdir().unwrap();
    let psi_path = dir.path().join("missing");
    assert!(read_psi_file(&psi_path).is_err());
}
