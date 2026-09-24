//! Reads and parses cgroup v2 I/O accounting metrics (`io.stat`).

use sentry_core::error::TelemetryError;
use sentry_core::models::IoDeviceMetrics;
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

/// Reads and parses `io.stat` file into a vector of per-device metrics.
pub fn read_cgroup_io(cgroup_dir: &Path) -> Result<Vec<IoDeviceMetrics>, TelemetryError> {
    let io_file = cgroup_dir.join("io.stat");
    let mut buf = [0u8; 512];
    let content = match read_sysfs_str(&io_file, &mut buf) {
        Ok(Some(c)) => c,
        Ok(None) => return Ok(Vec::new()),
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
