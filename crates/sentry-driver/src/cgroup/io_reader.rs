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
            Some(d) => d,
            None => continue,
        };

        // Validate device conforms to MAJOR:MINOR with non-empty numeric components
        let is_valid_device = match device.split_once(':') {
            Some((maj, min)) => {
                !maj.is_empty()
                    && !min.is_empty()
                    && maj.chars().all(|c| c.is_ascii_digit())
                    && min.chars().all(|c| c.is_ascii_digit())
            }
            None => false,
        };
        if !is_valid_device {
            continue;
        }

        let mut dev = IoDeviceMetrics {
            device: device.to_string(),
            rbytes: 0,
            wbytes: 0,
            rios: 0,
            wios: 0,
            dbytes: 0,
            dios: 0,
        };

        let mut parsed_any_keys = false;
        for token in tokens {
            if let Some((k, v)) = token.split_once('=') {
                let val = v.parse::<u64>().unwrap_or(0);
                match k {
                    "rbytes" => { dev.rbytes = val; parsed_any_keys = true; },
                    "wbytes" => { dev.wbytes = val; parsed_any_keys = true; },
                    "rios" => { dev.rios = val; parsed_any_keys = true; },
                    "wios" => { dev.wios = val; parsed_any_keys = true; },
                    "dbytes" => { dev.dbytes = val; parsed_any_keys = true; },
                    "dios" => { dev.dios = val; parsed_any_keys = true; },
                    _ => {}
                }
            }
        }
        if parsed_any_keys {
            device_stats.push(dev);
        }
    }

    Ok(device_stats)
}
