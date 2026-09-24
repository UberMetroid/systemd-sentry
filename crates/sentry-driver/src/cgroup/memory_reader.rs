//! Reads and parses cgroup v2 memory telemetry files.

use sentry_core::error::TelemetryError;
use sentry_core::models::{CgroupMemoryStats, MemoryEvents};
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

/// Reads `memory.current`, `memory.max` (handling "max"), and `memory.events`.
pub fn read_cgroup_memory(
    cgroup_dir: &Path,
) -> Result<(Option<u64>, Option<u64>, MemoryEvents), TelemetryError> {
    let mut buf = [0u8; 512];

    // 1. memory.current
    let current = match read_sysfs_str(&cgroup_dir.join("memory.current"), &mut buf) {
        Ok(Some(s)) => {
            let val = s.trim().parse::<u64>().map_err(|e| {
                TelemetryError::ParseInt("memory.current", e)
            })?;
            Some(val)
        }
        Ok(None) => None,
        Err(e) => return Err(TelemetryError::Io("memory.current", e)),
    };

    // 2. memory.max ("max" indicates unlimited ceiling)
    let max = match read_sysfs_str(&cgroup_dir.join("memory.max"), &mut buf) {
        Ok(Some(s)) => {
            let trimmed = s.trim();
            if trimmed == "max" {
                None
            } else {
                let val = trimmed.parse::<u64>().map_err(|e| {
                    TelemetryError::ParseInt("memory.max", e)
                })?;
                Some(val)
            }
        }
        Ok(None) => None,
        Err(e) => return Err(TelemetryError::Io("memory.max", e)),
    };

    // 3. memory.events
    let mut events = MemoryEvents::default();
    if let Ok(Some(events_str)) = read_sysfs_str(&cgroup_dir.join("memory.events"), &mut buf) {
        for line in events_str.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
                let val = val_str.parse::<u64>().unwrap_or(0);
                match key {
                    "low" => events.low = val,
                    "high" => events.high = val,
                    "max" => events.max = val,
                    "oom" => events.oom = val,
                    "oom_kill" => events.oom_kill = val,
                    "oom_group_kill" => events.oom_group_kill = val,
                    _ => {}
                }
            }
        }
    }

    Ok((current, max, events))
}

/// Reads cgroup memory metrics into a unified `CgroupMemoryStats` struct.
pub fn read_cgroup_memory_stats(cgroup_dir: &Path) -> Result<CgroupMemoryStats, TelemetryError> {
    let (current, max, events) = read_cgroup_memory(cgroup_dir)?;
    Ok(CgroupMemoryStats {
        current: current.unwrap_or(0),
        max,
        low_events: events.low,
        high_events: events.high,
        max_events: events.max,
        oom_events: events.oom,
        oom_kill_events: events.oom_kill,
    })
}
