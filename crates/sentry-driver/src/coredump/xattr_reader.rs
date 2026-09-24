//! Reads `user.coredump.*` extended attributes from coredump files via `rustix`.

use rustix::fs::getxattr;
use sentry_core::error::CoredumpError;
use sentry_core::models::CoredumpXattrs;
use std::path::Path;

/// Reads `user.coredump.*` extended attributes from the specified coredump file.
pub fn read_coredump_xattrs(path: &Path) -> Result<CoredumpXattrs, CoredumpError> {
    if !path.exists() {
        return Err(CoredumpError::FileNotFound(path.to_path_buf()));
    }

    let read_attr = |name: &str| -> Option<String> {
        let mut buf = vec![0u8; 1024];
        match getxattr(path, name, &mut buf) {
            Ok(size) => {
                buf.truncate(size);
                String::from_utf8(buf).ok()
            }
            Err(_) => None,
        }
    };

    Ok(CoredumpXattrs {
        pid: read_attr("user.coredump.pid").and_then(|s| s.parse::<u32>().ok()),
        signal: read_attr("user.coredump.signal").and_then(|s| s.parse::<i32>().ok()),
        comm: read_attr("user.coredump.comm"),
        exe: read_attr("user.coredump.exe"),
    })
}
