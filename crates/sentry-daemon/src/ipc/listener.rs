//! UNIX domain socket listener with systemd socket activation and stale socket recovery.

use sentry_driver::activation::parse_listen_fds;
use std::fs;
use std::io::{Error, ErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::Path;
use tokio::net::UnixListener;

/// Bind to the designated UNIX domain socket path or adopt systemd socket-activated FD 3.
pub fn bind_or_activate_socket(socket_path: &str) -> Result<UnixListener, Error> {
    // 1. Check for systemd socket activation ($LISTEN_FDS)
    if let Ok(mut fds) = parse_listen_fds(true) {
        if let Some(sock) = fds
            .iter()
            .position(|s| s.name == "sentry" || s.name == "systemd-sentry" || s.name == "sentry.socket")
            .map(|idx| fds.remove(idx))
            .or_else(|| fds.into_iter().next())
        {
            return sock.into_tokio_unix_listener();
        }
    }

    // 2. Fallback: Bind manually to filesystem path
    let path = Path::new(socket_path);

    // Ensure parent directory exists with 0755 permissions
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o755))?;
        }
    }

    // Test for stale socket
    if path.exists() {
        match StdUnixStream::connect(path) {
            Ok(_) => {
                return Err(Error::new(
                    ErrorKind::AddrInUse,
                    format!("Another systemd-sentry daemon is already actively listening on {}", socket_path),
                ));
            }
            Err(_) => {
                // Connection failed (connection refused or broken socket) -> safely remove stale socket
                let _ = fs::remove_file(path);
            }
        }
    }

    let listener = UnixListener::bind(path)?;

    // Set socket file permissions to 0660 (accessible by owner and group)
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o660));

    Ok(listener)
}
