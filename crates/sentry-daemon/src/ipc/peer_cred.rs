//! Kernel peer credentials extraction and zero-trust authorization.
//!
//! Uses Linux `SO_PEERCRED` via `rustix` without dynamic C library dependencies.

use rustix::net::sockopt::get_socket_peercred;
use std::os::unix::io::AsFd;

/// Extracted credentials of the connecting peer process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCredentials {
    /// Peer process user ID.
    pub uid: u32,
    /// Peer process group ID.
    pub gid: u32,
    /// Peer process ID.
    pub pid: Option<i32>,
}

/// Retrieve peer credentials directly from the connected UNIX domain socket.
pub fn get_peer_credentials<F: AsFd>(fd: &F) -> Result<PeerCredentials, String> {
    let ucred = get_socket_peercred(fd).map_err(|e| {
        format!("Failed to retrieve SO_PEERCRED from socket: {}", e)
    })?;

    Ok(PeerCredentials {
        uid: ucred.uid.as_raw(),
        gid: ucred.gid.as_raw(),
        pid: Some(ucred.pid.as_raw_nonzero().get()),
    })
}

/// Verify if the peer credentials authorize executing a mutating action.
///
/// Mutating actions (`reset`, `reload`) strictly require `uid == 0` (root)
/// or matching the current daemon user's UID.
pub fn authorize_action(
    creds: &PeerCredentials,
    daemon_uid: u32,
    is_mutating: bool,
) -> Result<(), (i32, String)> {
    if !is_mutating {
        // Read-only actions permitted for root, daemon user, or local sentry callers
        return Ok(());
    }

    if creds.uid == 0 || creds.uid == daemon_uid {
        Ok(())
    } else {
        Err((
            -32003,
            format!(
                "Permission denied: Mutating operations require UID 0 (root) or daemon UID {}. Caller UID: {}",
                daemon_uid, creds.uid
            ),
        ))
    }
}
