//! Resilient cgroup telemetry collector chaining cgroup v2, procfs PID extraction, and synthetic baseline.

use super::cgroup_collector::collect_cgroup_telemetry;
use super::proc_statm_extractor::extract_pid_metrics;
use sentry_core::models::{CgroupTelemetry, MemoryEvents};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Resiliently collects cgroup telemetry for `unit_name`.
///
/// 1. Probes cgroup v2 hierarchy at `base_cgroup` for `unit_name`.
/// 2. If cgroups v2 is absent or restricted, and `pid` is provided, extracts memory and CPU
///    metrics directly from `/proc/<pid>/statm` and `/proc/<pid>/stat`.
/// 3. If both fail or `pid` is absent, constructs a zeroed synthetic fallback baseline.
pub fn collect_cgroup_telemetry_resilient(
    base_cgroup: &Path,
    unit_name: &str,
    pid: Option<u32>,
) -> CgroupTelemetry {
    collect_cgroup_telemetry_resilient_with_proc(base_cgroup, unit_name, pid, Path::new("/proc"))
}

/// Helper allowing injection of custom procfs root for deterministic unit testing.
pub fn collect_cgroup_telemetry_resilient_with_proc(
    base_cgroup: &Path,
    unit_name: &str,
    pid: Option<u32>,
    proc_root: &Path,
) -> CgroupTelemetry {
    // Tier 1: Try cgroup v2 telemetry
    if let Ok(telemetry) = collect_cgroup_telemetry(base_cgroup, unit_name) {
        return telemetry;
    }

    // Tier 2: Try procfs extraction for the process if PID is available
    if let Some(target_pid) = pid.filter(|&p| p > 0) {
        if let Ok(proc_metrics) = extract_pid_metrics(proc_root, target_pid) {
            let timestamp_usec = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;

            let cgroup_path = proc_metrics
                .cgroup_path
                .map(|p| format!("/proc/{}/cgroup:{}", target_pid, p))
                .unwrap_or_else(|| format!("/proc/{}/synthetic", target_pid));

            return CgroupTelemetry {
                unit: Some(unit_name.to_string()),
                cgroup_path,
                memory_current_bytes: proc_metrics.memory_current_bytes,
                memory_max_bytes: None,
                memory_events: MemoryEvents::default(),
                cpu_stat: proc_metrics.cpu_stat,
                io_stats: Vec::new(),
                populated: Some(true),
                frozen: Some(false),
                timestamp_usec,
                synthetic: true,
            };
        }
    }

    // Tier 3: Synthetic fallback baseline
    CgroupTelemetry::synthetic_fallback(Some(unit_name.to_string()), None)
}
