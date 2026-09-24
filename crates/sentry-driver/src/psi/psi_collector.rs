//! Collector aggregating CPU, memory, and I/O PSI records.

use super::psi_reader::read_psi_file;
use sentry_core::error::PsiError;
use sentry_core::models::PressureTelemetry;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Collects system-wide PSI telemetry from `/proc/pressure/` (or a mock directory).
pub fn collect_system_psi(proc_path: &Path) -> Result<PressureTelemetry, PsiError> {
    let cpu = read_psi_file(&proc_path.join("cpu"))?;
    let memory = read_psi_file(&proc_path.join("memory"))?;
    let io = read_psi_file(&proc_path.join("io"))?;

    let timestamp_usec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;

    Ok(PressureTelemetry::new(
        None,
        cpu,
        memory,
        io,
        timestamp_usec,
    ))
}

/// Collects per-cgroup PSI telemetry from `cpu.pressure`, `memory.pressure`, and `io.pressure`.
pub fn collect_cgroup_psi(
    cgroup_dir: &Path,
    unit_name: Option<String>,
) -> Result<PressureTelemetry, PsiError> {
    let cpu = read_psi_file(&cgroup_dir.join("cpu.pressure"))?;
    let memory = read_psi_file(&cgroup_dir.join("memory.pressure"))?;
    let io = read_psi_file(&cgroup_dir.join("io.pressure"))?;

    let timestamp_usec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;

    Ok(PressureTelemetry::new(
        unit_name,
        cpu,
        memory,
        io,
        timestamp_usec,
    ))
}
