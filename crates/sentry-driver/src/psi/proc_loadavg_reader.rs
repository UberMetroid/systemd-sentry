//! Pure-Rust stack reader for `/proc/loadavg` deriving synthetic CPU pressure metrics.

use sentry_core::error::PsiError;
use sentry_core::models::{PsiLine, PsiRecord};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

/// Cached online logical processor count to eliminate heap allocations on hot path.
static ONLINE_CPUS: OnceLock<f64> = OnceLock::new();

/// Reads `/proc/loadavg` from `path` into a fixed stack buffer (`[u8; 128]`)
/// and derives synthetic CPU pressure metrics (`PsiRecord`).
pub fn read_proc_loadavg(path: &Path) -> Result<PsiRecord, PsiError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(PsiError::Io(e)),
    };

    let mut buf = [0u8; 128];
    let mut total = 0;

    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(PsiError::Io(e)),
        }
    }

    let text = std::str::from_utf8(&buf[..total])
        .map_err(|e| PsiError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

    parse_loadavg_to_psi(text)
}

/// Parses the text of `/proc/loadavg` into synthetic `PsiRecord`.
pub fn parse_loadavg_to_psi(content: &str) -> Result<PsiRecord, PsiError> {
    let mut tokens = content.split_whitespace();

    let load1_str = tokens.next().ok_or_else(|| {
        PsiError::MissingField("load1", content.to_string())
    })?;
    let load5_str = tokens.next().ok_or_else(|| {
        PsiError::MissingField("load5", content.to_string())
    })?;
    let load15_str = tokens.next().ok_or_else(|| {
        PsiError::MissingField("load15", content.to_string())
    })?;

    let load1: f64 = load1_str.parse().map_err(|_| {
        PsiError::InvalidLineFormat(format!("invalid float load1: {}", load1_str))
    })?;
    let load5: f64 = load5_str.parse().map_err(|_| {
        PsiError::InvalidLineFormat(format!("invalid float load5: {}", load5_str))
    })?;
    let load15: f64 = load15_str.parse().map_err(|_| {
        PsiError::InvalidLineFormat(format!("invalid float load15: {}", load15_str))
    })?;

    if load1 < 0.0 || load5 < 0.0 || load15 < 0.0 {
        return Err(PsiError::InvalidLineFormat("negative load values are invalid".to_string()));
    }

    // Determine number of online logical processors to normalize load pressure.
    let cpus = *ONLINE_CPUS.get_or_init(|| {
        std::thread::available_parallelism()
            .map(|n| n.get() as f64)
            .unwrap_or(1.0)
    });

    // Contention occurs when load exceeds available cores:
    // (load - cpus) / cpus * 100%, clamped to [0.0, 100.0]
    let avg10 = if load1 > cpus {
        ((load1 - cpus) / cpus * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let avg60 = if load5 > cpus {
        ((load5 - cpus) / cpus * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let avg300 = if load15 > cpus {
        ((load15 - cpus) / cpus * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let some = PsiLine::new(avg10, avg60, avg300, 0);
    // On Linux kernel < 6.9, CPU pressure only reports 'some'; 'full' is None.
    Ok(PsiRecord::new(some, None))
}
