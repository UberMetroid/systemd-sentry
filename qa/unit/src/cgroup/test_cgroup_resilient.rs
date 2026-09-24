//! 1:1 Unit QA tests for resilient cgroup telemetry collection with unprivileged fallback.

use sentry_core::models::CgroupTelemetry;
use sentry_driver::cgroup::{
    collect_cgroup_telemetry_resilient, collect_cgroup_telemetry_resilient_with_proc,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cgroup_models_synthetic_fallback() {
    let syn = CgroupTelemetry::synthetic_fallback(Some("test.service".into()), None);
    assert!(syn.is_synthetic());
    assert_eq!(syn.unit.as_deref(), Some("test.service"));
    assert!(syn.cgroup_path.contains("synthetic"));
    assert!(syn.memory_current_bytes.is_none());
    assert!(syn.memory_max_bytes.is_none());
    assert!(!syn.had_oom_kill());
}

#[test]
fn test_collect_cgroup_telemetry_resilient_live() {
    let dir = tempdir().unwrap();
    let unit_cgroup = dir.path().join("system.slice/api.service");
    fs::create_dir_all(&unit_cgroup).unwrap();

    fs::write(unit_cgroup.join("memory.current"), "104857600\n").unwrap();
    fs::write(unit_cgroup.join("cpu.stat"), "usage_usec 500000\n").unwrap();
    fs::write(unit_cgroup.join("io.stat"), "").unwrap();

    let telemetry = collect_cgroup_telemetry_resilient(dir.path(), "api.service", None);
    assert!(!telemetry.is_synthetic());
    assert!(!telemetry.synthetic);
    assert_eq!(telemetry.unit.as_deref(), Some("api.service"));
    assert_eq!(telemetry.memory_current_bytes, Some(104857600));
    assert_eq!(telemetry.cpu_stat.usage_usec, 500000);
}

#[test]
fn test_collect_cgroup_telemetry_resilient_procfs_fallback() {
    let cgroup_dir = tempdir().unwrap();
    let proc_dir = tempdir().unwrap();

    let pid_dir = proc_dir.path().join("4321");
    fs::create_dir_all(&pid_dir).unwrap();

    fs::write(pid_dir.join("statm"), "20000 500 200 50 0 20 0\n").unwrap();
    let stat_content = "4321 (worker) S 1 4321 4321 0 -1 4194304 10 20 0 0 300 100 0 0 20 0 1 0 1000 10000 500\n";
    fs::write(pid_dir.join("stat"), stat_content).unwrap();

    let telemetry = collect_cgroup_telemetry_resilient_with_proc(
        cgroup_dir.path(),
        "worker.service",
        Some(4321),
        proc_dir.path(),
    );

    assert!(telemetry.is_synthetic());
    assert!(telemetry.synthetic);
    assert!(telemetry.cgroup_path.starts_with("/proc/4321/"));
    assert_eq!(telemetry.unit.as_deref(), Some("worker.service"));
    let page_size = rustix::param::page_size() as u64;
    assert_eq!(telemetry.memory_current_bytes, Some(500 * page_size));
    assert_eq!(telemetry.cpu_stat.usage_usec, 400 * 10_000);
}

#[test]
fn test_collect_cgroup_telemetry_resilient_synthetic_fallback() {
    let cgroup_dir = tempdir().unwrap();
    let proc_dir = tempdir().unwrap();

    // No cgroup and nonexistent PID
    let telemetry = collect_cgroup_telemetry_resilient_with_proc(
        cgroup_dir.path(),
        "dead.service",
        Some(88888),
        proc_dir.path(),
    );

    assert!(telemetry.is_synthetic());
    assert!(telemetry.synthetic);
    assert_eq!(telemetry.unit.as_deref(), Some("dead.service"));
    assert!(telemetry.memory_current_bytes.is_none());
}
