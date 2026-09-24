//! 1:1 Unit QA tests for pure-Rust /proc/loadavg reader and synthetic CPU pressure parser.

use sentry_driver::psi::{parse_loadavg_to_psi, read_proc_loadavg};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_proc_loadavg_standard() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("loadavg");
    fs::write(&file_path, "0.50 1.25 2.00 3/400 98765\n").unwrap();

    let record = read_proc_loadavg(&file_path).expect("Must parse loadavg");
    assert!(record.full.is_none());
    assert_eq!(record.some.total_usec, 0);
}

#[test]
fn test_read_proc_loadavg_missing_file_returns_error() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("nonexistent_loadavg");
    assert!(read_proc_loadavg(&file_path).is_err());
}

#[test]
fn test_parse_loadavg_zero_contention() {
    // When load is 0.00, pressure must be 0.00%
    let record = parse_loadavg_to_psi("0.00 0.00 0.00 1/100 1234\n").unwrap();
    assert_eq!(record.some.avg10, 0.0);
    assert_eq!(record.some.avg60, 0.0);
    assert_eq!(record.some.avg300, 0.0);
}

#[test]
fn test_parse_loadavg_extreme_contention_clamped() {
    // High load average (1000.0) should clamp to 100.0%
    let record = parse_loadavg_to_psi("1000.00 800.00 500.00 50/1000 9999\n").unwrap();
    assert_eq!(record.some.avg10, 100.0);
    assert_eq!(record.some.avg60, 100.0);
    assert_eq!(record.some.avg300, 100.0);
}

#[test]
fn test_parse_loadavg_malformed_inputs() {
    assert!(parse_loadavg_to_psi("").is_err());
    assert!(parse_loadavg_to_psi("0.50").is_err());
    assert!(parse_loadavg_to_psi("0.50 1.00").is_err());
    assert!(parse_loadavg_to_psi("not_a_float 1.00 2.00").is_err());
    assert!(parse_loadavg_to_psi("-1.00 1.00 2.00").is_err());
}

#[test]
fn test_parse_loadavg_cached_repeated_calls() {
    for _ in 0..100 {
        let record = parse_loadavg_to_psi("0.50 1.25 2.00 3/400 98765\n").unwrap();
        assert!(record.full.is_none());
        assert_eq!(record.some.total_usec, 0);
    }
}
