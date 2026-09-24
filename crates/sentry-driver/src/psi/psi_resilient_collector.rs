//! Resilient PSI collectors with zero-overhead fallback for unprivileged containers.

use super::proc_loadavg_reader::read_proc_loadavg;
use super::psi_collector::{collect_cgroup_psi, collect_system_psi};
use sentry_core::models::{PressureTelemetry, PsiLine, PsiRecord};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn read_proc_meminfo(path: &Path) -> Option<PsiRecord> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 512];
    let mut total = 0;

    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return None,
        }
    }

    let text = std::str::from_utf8(&buf[..total]).ok()?;
    let mut mem_total_kb: Option<u64> = None;
    let mut mem_avail_kb: Option<u64> = None;

    for line in text.lines() {
        if line.starts_with("MemTotal:") {
            mem_total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok());
        } else if line.starts_with("MemAvailable:") {
            mem_avail_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok());
        }
        if mem_total_kb.is_some() && mem_avail_kb.is_some() {
            break;
        }
    }

    let (total_kb, avail_kb) = (mem_total_kb?, mem_avail_kb?);
    if total_kb == 0 {
        return None;
    }

    let used_ratio = (total_kb.saturating_sub(avail_kb)) as f64 / total_kb as f64;
    // Synthesize pressure if memory utilization exceeds 90%
    let stall_pct = if used_ratio > 0.90 {
        ((used_ratio - 0.90) * 1000.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let line = PsiLine::new(stall_pct, stall_pct, stall_pct, 0);
    Some(PsiRecord::new(line.clone(), Some(line)))
}

/// Resiliently collects system-wide PSI telemetry.
/// Falls back to `/proc/loadavg` and `/proc/meminfo` before `PressureTelemetry::synthetic_zero`.
pub fn collect_system_psi_resilient(proc_path: &Path) -> PressureTelemetry {
    if let Ok(telemetry) = collect_system_psi(proc_path) {
        return telemetry;
    }

    // Attempt fallback via /proc/loadavg and /proc/meminfo
    let parent = proc_path.parent().unwrap_or(proc_path);
    let loadavg_path = parent.join("loadavg");
    let meminfo_path = parent.join("meminfo");

    let cpu = read_proc_loadavg(&loadavg_path)
        .or_else(|_| read_proc_loadavg(Path::new("/proc/loadavg")))
        .unwrap_or_else(|_| PsiRecord::zero());

    let memory = read_proc_meminfo(&meminfo_path)
        .or_else(|| read_proc_meminfo(Path::new("/proc/meminfo")))
        .unwrap_or_else(PsiRecord::zero);

    let io = PsiRecord::zero();

    let timestamp_usec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;

    PressureTelemetry::new_synthetic(None, cpu, memory, io, timestamp_usec)
}

/// Resiliently collects cgroup-level PSI telemetry with fallback to synthetic zero.
pub fn collect_cgroup_psi_resilient(
    cgroup_dir: &Path,
    unit_name: Option<String>,
) -> PressureTelemetry {
    if let Ok(telemetry) = collect_cgroup_psi(cgroup_dir, unit_name.clone()) {
        return telemetry;
    }

    PressureTelemetry::synthetic_zero(unit_name)
}
