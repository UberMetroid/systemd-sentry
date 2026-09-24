//! Resolves `$NOTIFY_SOCKET` paths to Unix socket addresses.

use super::error::NotifyError;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::SocketAddr;
use std::path::Path;

/// Resolves a raw `$NOTIFY_SOCKET` path into a `SocketAddr`,
/// handling Linux abstract sockets (`@`-prefixed) and filesystem paths.
pub fn resolve_notify_address(raw_path: &str) -> Result<SocketAddr, NotifyError> {
    if raw_path.is_empty() {
        return Err(NotifyError::InvalidSocketAddress(
            raw_path.to_string(),
            "Path cannot be empty".to_string(),
        ));
    }

    if let Some(abstract_name) = raw_path.strip_prefix('@') {
        SocketAddr::from_abstract_name(abstract_name.as_bytes()).map_err(|e| {
            NotifyError::InvalidSocketAddress(
                raw_path.to_string(),
                format!("Failed to create abstract socket address: {e}"),
            )
        })
    } else {
        let path = Path::new(raw_path);
        SocketAddr::from_pathname(path).map_err(|e| {
            NotifyError::InvalidSocketAddress(
                raw_path.to_string(),
                format!("Failed to create filesystem socket address: {e}"),
            )
        })
    }
}
