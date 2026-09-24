//! 1:1 Unit QA tests for unprivileged container procfs statm/stat extractor.

use sentry_driver::cgroup::extract_pid_metrics;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_extract_pid_metrics_success() {
    let dir = tempdir().unwrap();
    let pid_dir = dir.path().join("1234");
    fs::create_dir_all(&pid_dir).unwrap();

    // 1. Mock /proc/1234/statm: size resident shared text lib data dt
    fs::write(pid_dir.join("statm"), "50000 1200 800 100 0 50 0\n").unwrap();

    // 2. Mock /proc/1234/stat
    // Field 14 (utime) = 150, Field 15 (stime) = 250
    let stat_content = "1234 (test-worker) S 1 1234 1234 0 -1 4194304 10 20 0 0 150 250 0 0 20 0 1 0 1000 10000 500\n";
    fs::write(pid_dir.join("stat"), stat_content).unwrap();

    // 3. Mock /proc/1234/cgroup
    fs::write(pid_dir.join("cgroup"), "0::/user.slice/worker.service\n").unwrap();

    let metrics = extract_pid_metrics(dir.path(), 1234).expect("Must extract metrics for PID");

    let page_size = rustix::param::page_size() as u64;
    assert_eq!(metrics.memory_current_bytes, Some(1200 * page_size));
    assert_eq!(metrics.cpu_stat.user_usec, 150 * 10_000);
    assert_eq!(metrics.cpu_stat.system_usec, 250 * 10_000);
    assert_eq!(metrics.cpu_stat.usage_usec, 400 * 10_000);
    assert_eq!(
        metrics.cgroup_path.as_deref(),
        Some("/user.slice/worker.service")
    );
}

#[test]
fn test_extract_pid_metrics_parentheses_in_process_name() {
    let dir = tempdir().unwrap();
    let pid_dir = dir.path().join("5678");
    fs::create_dir_all(&pid_dir).unwrap();

    fs::write(pid_dir.join("statm"), "10000 500 200 50 0 20 0\n").unwrap();

    // Process name with multiple parentheses: "(worker (sub) proc)"
    let stat_content = "5678 (worker (sub) proc) R 1 5678 5678 0 -1 4194304 10 20 0 0 80 40 0 0 20 0 1 0 1000 10000 500\n";
    fs::write(pid_dir.join("stat"), stat_content).unwrap();

    let metrics = extract_pid_metrics(dir.path(), 5678).expect("Must parse stat with nested parens");
    assert_eq!(metrics.cpu_stat.user_usec, 80 * 10_000);
    assert_eq!(metrics.cpu_stat.system_usec, 40 * 10_000);
    assert_eq!(metrics.cpu_stat.usage_usec, 120 * 10_000);
}

#[test]
fn test_extract_pid_metrics_missing_pid_returns_error() {
    let dir = tempdir().unwrap();
    assert!(extract_pid_metrics(dir.path(), 99999).is_err());
}
