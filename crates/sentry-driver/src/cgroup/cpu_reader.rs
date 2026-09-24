//! Reads and parses cgroup v2 CPU accounting metrics (`cpu.stat`).

use sentry_core::error::TelemetryError;
use sentry_core::models::CpuStat;
use std::fs;
use std::path::Path;

/// Reads and parses `cpu.stat` file into `CpuStat`.
pub fn read_cgroup_cpu(cgroup_dir: &Path) -> Result<CpuStat, TelemetryError> {
    let cpu_file = cgroup_dir.join("cpu.stat");
    let content = match fs::read_to_string(&cpu_file) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(CpuStat::default()),
        Err(e) => return Err(TelemetryError::Io("cpu.stat", e)),
    };

    let mut stats = CpuStat::default();

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
            let val = val_str.parse::<u64>().unwrap_or(0);
            match key {
                "usage_usec" => stats.usage_usec = val,
                "user_usec" => stats.user_usec = val,
                "system_usec" => stats.system_usec = val,
                "nr_periods" => stats.nr_periods = val,
                "nr_throttled" => stats.nr_throttled = val,
                "throttled_usec" => stats.throttled_usec = val,
                _ => {}
            }
        }
    }

    Ok(stats)
}
