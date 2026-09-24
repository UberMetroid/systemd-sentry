//! 1:1 Unit QA tests for cgroup telemetry collector.

use sentry_driver::cgroup::collect_cgroup_telemetry;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_collect_cgroup_telemetry_full() {
    let dir = tempdir().unwrap();
    let unit_dir = dir.path().join("system.slice/worker.service");
    fs::create_dir_all(&unit_dir).unwrap();

    fs::write(unit_dir.join("memory.current"), b"123456\n").unwrap();
    fs::write(unit_dir.join("memory.max"), b"500000\n").unwrap();
    fs::write(unit_dir.join("cpu.stat"), b"usage_usec 500\n").unwrap();
    fs::write(
        unit_dir.join("cgroup.events"),
        b"populated 1\nfrozen 0\n",
    )
    .unwrap();

    let telem = collect_cgroup_telemetry(dir.path(), "worker.service").unwrap();
    assert_eq!(telem.unit.as_deref(), Some("worker.service"));
    assert_eq!(telem.memory_current_bytes, Some(123456));
    assert_eq!(telem.memory_max_bytes, Some(500000));
    assert_eq!(telem.cpu_stat.usage_usec, 500);
    assert_eq!(telem.populated, Some(true));
    assert_eq!(telem.frozen, Some(false));
    assert!(telem.timestamp_usec > 0);
}
