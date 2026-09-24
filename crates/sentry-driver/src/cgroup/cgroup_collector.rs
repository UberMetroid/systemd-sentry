//! Aggregates cgroup v2 metrics into a complete `CgroupTelemetry` snapshot.

use super::cgroup_locator::locate_unit_cgroup;
use super::cpu_reader::read_cgroup_cpu;
use super::io_reader::read_cgroup_io;
use super::memory_reader::read_cgroup_memory;
use sentry_core::error::TelemetryError;
use sentry_core::models::CgroupTelemetry;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Collects complete cgroup telemetry for a unit under `base_cgroup` path.
pub fn collect_cgroup_telemetry(
    base_cgroup: &Path,
    unit_name: &str,
) -> Result<CgroupTelemetry, TelemetryError> {
    let cgroup_dir = locate_unit_cgroup(base_cgroup, unit_name)?;

    let (memory_current_bytes, memory_max_bytes, memory_events) =
        read_cgroup_memory(&cgroup_dir)?;
    let cpu_stat = read_cgroup_cpu(&cgroup_dir)?;
    let io_stats = read_cgroup_io(&cgroup_dir)?;

    let mut populated = None;
    let mut frozen = None;

    if let Ok(events_str) = fs::read_to_string(cgroup_dir.join("cgroup.events")) {
        for line in events_str.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                match k {
                    "populated" => populated = Some(v == "1"),
                    "frozen" => frozen = Some(v == "1"),
                    _ => {}
                }
            }
        }
    }

    let timestamp_usec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;

    Ok(CgroupTelemetry {
        unit: Some(unit_name.to_string()),
        cgroup_path: cgroup_dir.to_string_lossy().to_string(),
        memory_current_bytes,
        memory_max_bytes,
        memory_events,
        cpu_stat,
        io_stats,
        populated,
        frozen,
        timestamp_usec,
    })
}
