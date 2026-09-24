//! Locates unit cgroup hierarchy path under `/sys/fs/cgroup`.

use sentry_core::error::TelemetryError;
use std::path::{Path, PathBuf};

/// Resolves the cgroup directory path for a given systemd unit under `base_path`.
/// Supports standard system units, flat container units, user slices, and docker units.
pub fn locate_unit_cgroup(base_path: &Path, unit_name: &str) -> Result<PathBuf, TelemetryError> {
    let sanitized_unit = unit_name.trim_start_matches('/');

    let candidates = if sanitized_unit.starts_with("system.slice/")
        || sanitized_unit.starts_with("user.slice/")
        || sanitized_unit.starts_with("docker/")
    {
        vec![base_path.join(sanitized_unit)]
    } else {
        vec![
            base_path.join("system.slice").join(sanitized_unit),
            base_path.join(sanitized_unit),
            base_path.join("user.slice").join(sanitized_unit),
            base_path.join("docker").join(sanitized_unit),
        ]
    };

    let default_path = candidates[0].clone();

    for candidate in candidates {
        if candidate.exists() {
            if !candidate.is_dir() {
                return Err(TelemetryError::NotADirectory(candidate));
            }
            return Ok(candidate);
        }
    }

    Err(TelemetryError::CgroupNotFound {
        unit: unit_name.to_string(),
        path: default_path,
    })
}
