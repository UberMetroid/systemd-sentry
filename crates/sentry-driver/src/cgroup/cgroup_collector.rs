//! Aggregates cgroup v2 metrics into a complete `CgroupTelemetry` snapshot.

use super::cgroup_locator::locate_unit_cgroup;
use super::cpu_reader::read_cgroup_cpu;
use super::io_reader::read_cgroup_io;
use super::memory_reader::read_cgroup_memory;
use sentry_core::error::TelemetryError;
use sentry_core::models::CgroupTelemetry;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn read_sysfs_str<'a>(path: &Path, buf: &'a mut [u8; 512]) -> Result<Option<&'a str>, std::io::Error> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::PermissionDenied =>
        {
            return Ok(None)
        }
        Err(e) => return Err(e),
    };
    let mut total = 0;
    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(None),
            Err(e) => return Err(e),
        }
    }
    let s = std::str::from_utf8(&buf[..total])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(s))
}

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

    let mut buf = [0u8; 512];
    if let Ok(Some(events_str)) = read_sysfs_str(&cgroup_dir.join("cgroup.events"), &mut buf) {
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
        synthetic: false,
    })
}
