//! Extracts memory and CPU telemetry from `/proc/<pid>/statm` and `/proc/<pid>/stat`.

use sentry_core::models::CpuStat;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Telemetry metrics extracted directly from a process's procfs files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcPidMetrics {
    /// Resident memory in bytes (from `/proc/<pid>/statm`).
    pub memory_current_bytes: Option<u64>,
    /// CPU accounting statistics (from `/proc/<pid>/stat`).
    pub cpu_stat: CpuStat,
    /// Relative cgroup v2 path (from `/proc/<pid>/cgroup`), if available.
    pub cgroup_path: Option<String>,
}

fn read_small_file(path: &Path, buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut file = File::open(path)?;
    let mut total = 0;
    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(total)
}

/// Extracts telemetry metrics for `pid` under `proc_root`.
pub fn extract_pid_metrics(proc_root: &Path, pid: u32) -> Result<ProcPidMetrics, std::io::Error> {
    let pid_dir = proc_root.join(pid.to_string());
    let mut metrics = ProcPidMetrics::default();

    // 1. Extract memory from /proc/<pid>/statm: "size resident shared text lib data dt"
    let mut statm_buf = [0u8; 128];
    if let Ok(n) = read_small_file(&pid_dir.join("statm"), &mut statm_buf) {
        if let Ok(content) = std::str::from_utf8(&statm_buf[..n]) {
            let mut parts = content.split_whitespace();
            // Skip size (field 0), take resident pages (field 1)
            if let Some(resident_pages_str) = parts.nth(1) {
                if let Ok(pages) = resident_pages_str.parse::<u64>() {
                    let page_size = rustix::param::page_size() as u64;
                    metrics.memory_current_bytes = Some(pages.saturating_mul(page_size));
                }
            }
        }
    }

    // 2. Extract CPU from /proc/<pid>/stat
    let mut stat_buf = [0u8; 512];
    if let Ok(n) = read_small_file(&pid_dir.join("stat"), &mut stat_buf) {
        if let Ok(content) = std::str::from_utf8(&stat_buf[..n]) {
            // Find end of comm field "(...)"
            if let Some(comm_end) = content.rfind(')') {
                let rest = content.get(comm_end + 1..).unwrap_or("");
                let mut tokens = rest.split_whitespace();
                // tokens.nth(11) is utime (field 14)
                // tokens.next() is stime (field 15)
                if let (Some(utime_str), Some(stime_str)) = (tokens.nth(11), tokens.next()) {
                    let utime = utime_str.parse::<u64>().unwrap_or(0);
                    let stime = stime_str.parse::<u64>().unwrap_or(0);
                    // Linux USER_HZ is 100 ticks/sec, so 1 tick = 10,000 usec
                    let user_usec = utime.saturating_mul(10_000);
                    let system_usec = stime.saturating_mul(10_000);
                    let usage_usec = user_usec.saturating_add(system_usec);

                    metrics.cpu_stat = CpuStat {
                        usage_usec,
                        user_usec,
                        system_usec,
                        nr_periods: 0,
                        nr_throttled: 0,
                        throttled_usec: 0,
                    };
                }
            }
        }
    }

    // 3. Extract cgroup v2 path from /proc/<pid>/cgroup: "0::<path>"
    let mut cgroup_buf = [0u8; 256];
    if let Ok(n) = read_small_file(&pid_dir.join("cgroup"), &mut cgroup_buf) {
        if let Ok(content) = std::str::from_utf8(&cgroup_buf[..n]) {
            for line in content.lines() {
                if let Some(path) = line.strip_prefix("0::") {
                    let trimmed = path.trim();
                    if !trimmed.is_empty() {
                        metrics.cgroup_path = Some(trimmed.to_string());
                        break;
                    }
                }
            }
        }
    }

    // If both statm and stat failed to extract any metrics, return error
    if metrics.memory_current_bytes.is_none() && metrics.cpu_stat == CpuStat::default() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No process telemetry found for PID {}", pid),
        ));
    }

    Ok(metrics)
}
