//! Reads and parses cgroup v2 CPU accounting metrics (`cpu.stat`).

use sentry_core::error::TelemetryError;
use sentry_core::models::CpuStat;
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

/// Reads and parses `cpu.stat` file into `CpuStat`.
pub fn read_cgroup_cpu(cgroup_dir: &Path) -> Result<CpuStat, TelemetryError> {
    let cpu_file = cgroup_dir.join("cpu.stat");
    let mut buf = [0u8; 512];
    let content = match read_sysfs_str(&cpu_file, &mut buf) {
        Ok(Some(c)) => c,
        Ok(None) => return Ok(CpuStat::default()),
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
