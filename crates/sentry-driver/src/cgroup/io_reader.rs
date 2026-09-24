//! Reads and parses cgroup v2 I/O accounting metrics (`io.stat`).

use sentry_core::error::TelemetryError;
use sentry_core::models::IoDeviceMetrics;
use std::fs;
use std::path::Path;

/// Reads and parses `io.stat` file into a vector of per-device metrics.
pub fn read_cgroup_io(cgroup_dir: &Path) -> Result<Vec<IoDeviceMetrics>, TelemetryError> {
    let io_file = cgroup_dir.join("io.stat");
    let content = match fs::read_to_string(&io_file) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(TelemetryError::Io("io.stat", e)),
    };

    let mut device_stats = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let device = match tokens.next() {
            Some(d) => d.to_string(),
            None => continue,
        };

        let mut dev = IoDeviceMetrics {
            device,
            rbytes: 0,
            wbytes: 0,
            rios: 0,
            wios: 0,
            dbytes: 0,
            dios: 0,
        };

        for token in tokens {
            if let Some((k, v)) = token.split_once('=') {
                let val = v.parse::<u64>().unwrap_or(0);
                match k {
                    "rbytes" => dev.rbytes = val,
                    "wbytes" => dev.wbytes = val,
                    "rios" => dev.rios = val,
                    "wios" => dev.wios = val,
                    "dbytes" => dev.dbytes = val,
                    "dios" => dev.dios = val,
                    _ => {}
                }
            }
        }
        device_stats.push(dev);
    }

    Ok(device_stats)
}
