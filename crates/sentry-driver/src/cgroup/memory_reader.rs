//! Reads and parses cgroup v2 memory telemetry files.

use sentry_core::error::TelemetryError;
use sentry_core::models::{CgroupMemoryStats, MemoryEvents};
use std::fs;
use std::path::Path;

/// Reads `memory.current`, `memory.max` (handling "max"), and `memory.events`.
pub fn read_cgroup_memory(
    cgroup_dir: &Path,
) -> Result<(Option<u64>, Option<u64>, MemoryEvents), TelemetryError> {
    // 1. memory.current
    let current = match fs::read_to_string(cgroup_dir.join("memory.current")) {
        Ok(s) => {
            let val = s.trim().parse::<u64>().map_err(|e| {
                TelemetryError::ParseInt("memory.current", e)
            })?;
            Some(val)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(TelemetryError::Io("memory.current", e)),
    };

    // 2. memory.max ("max" indicates unlimited ceiling)
    let max = match fs::read_to_string(cgroup_dir.join("memory.max")) {
        Ok(s) => {
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
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(TelemetryError::Io("memory.max", e)),
    };

    // 3. memory.events
    let mut events = MemoryEvents::default();
    if let Ok(events_str) = fs::read_to_string(cgroup_dir.join("memory.events")) {
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
