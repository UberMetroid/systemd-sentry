//! Scans `/var/lib/systemd/coredump` for matching crash dumps and metadata.

use super::xattr_reader::read_coredump_xattrs;
use sentry_core::models::CoredumpRecord;
use std::fs;
use std::path::Path;

/// Scan coredump directory for the most recent crash matching an executable or unit command name.
pub fn find_latest_coredump(coredump_dir: &Path, comm_match: Option<&str>) -> Option<CoredumpRecord> {
    if !coredump_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(coredump_dir).ok()?;
    let mut latest_record: Option<(u64, CoredumpRecord)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Only inspect files containing "core" or ending in ".zst" / ".lz4"
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.contains("core") {
            continue;
        }

        if let Ok(xattrs) = read_coredump_xattrs(&path) {
            let matched = match (comm_match, xattrs.unit.as_deref(), xattrs.comm.as_deref()) {
                (Some(expected), Some(actual_unit), _) => {
                    let exp = expected.strip_suffix(".service").unwrap_or(expected);
                    let act = actual_unit.strip_suffix(".service").unwrap_or(actual_unit);
                    act == exp
                }
                (Some(expected), None, Some(actual_comm)) => {
                    let exp = expected.strip_suffix(".service").unwrap_or(expected);
                    !actual_comm.is_empty()
                        && (actual_comm == exp || exp.starts_with(actual_comm) || actual_comm.starts_with(exp))
                }
                (None, _, _) => true,
                _ => false,
            };

            if matched {
                let mtime = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_micros() as u64)
                    .unwrap_or(0);

                let record = CoredumpRecord {
                    unit: xattrs
                        .unit
                        .or_else(|| comm_match.map(|s| s.to_string()))
                        .unwrap_or_default(),
                    pid: xattrs.pid.unwrap_or(0),
                    signal: xattrs.signal.unwrap_or(0),
                    signal_name: xattrs
                        .signal
                        .map(|s| match s {
                            11 => "SIGSEGV",
                            6 => "SIGABRT",
                            4 => "SIGILL",
                            7 => "SIGBUS",
                            8 => "SIGFPE",
                            9 => "SIGKILL",
                            15 => "SIGTERM",
                            _ => "UNKNOWN",
                        }.to_string())
                        .unwrap_or_else(|| "UNKNOWN".to_string()),
                    executable: xattrs.exe,
                    stack_trace: None,
                    core_file: Some(path.display().to_string()),
                    timestamp_usec: mtime,
                    uid: None,
                    gid: None,
                };

                if let Some((prev_time, _)) = latest_record {
                    if mtime > prev_time {
                        latest_record = Some((mtime, record));
                    }
                } else {
                    latest_record = Some((mtime, record));
                }
            }
        }
    }

    latest_record.map(|(_, rec)| rec)
}
