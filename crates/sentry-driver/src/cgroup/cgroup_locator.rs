//! Locates unit cgroup hierarchy path under `/sys/fs/cgroup`.

use sentry_core::error::TelemetryError;
use std::path::{Path, PathBuf};

/// Resolves the cgroup directory path for a given systemd unit under `base_path`.
pub fn locate_unit_cgroup(base_path: &Path, unit_name: &str) -> Result<PathBuf, TelemetryError> {
    let sanitized_unit = unit_name.trim_start_matches('/');
    let candidate = if sanitized_unit.starts_with("system.slice/") {
        base_path.join(sanitized_unit)
    } else {
        base_path.join("system.slice").join(sanitized_unit)
    };

    if !candidate.exists() {
        return Err(TelemetryError::CgroupNotFound {
            unit: unit_name.to_string(),
            path: candidate,
        });
    }

    if !candidate.is_dir() {
        return Err(TelemetryError::NotADirectory(candidate));
    }

    Ok(candidate)
}
