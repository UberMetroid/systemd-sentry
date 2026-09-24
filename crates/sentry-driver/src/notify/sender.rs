//! Pure Rust socket writer transmitting datagram notifications to systemd.

use super::encoder::encode_notify_payload;
use super::error::NotifyError;
use super::socket_addr::resolve_notify_address;
use super::state::NotifyState;
use std::env;
use std::os::unix::net::UnixDatagram;

/// Transmits a set of notification states to `$NOTIFY_SOCKET`.
///
/// Returns `Ok(true)` if notification was sent, or `Ok(false)` if `$NOTIFY_SOCKET`
/// was unset or empty.
pub fn send_notify(states: &[NotifyState], unset_env: bool) -> Result<bool, NotifyError> {
    let socket_var = match env::var("NOTIFY_SOCKET") {
        Ok(val) if !val.is_empty() => val,
        _ => return Ok(false),
    };

    if unset_env {
        env::remove_var("NOTIFY_SOCKET");
    }

    let addr = resolve_notify_address(&socket_var)?;
    let payload = encode_notify_payload(states);
    let socket = UnixDatagram::unbound()?;
    socket.set_nonblocking(true)?;

    match socket.send_to_addr(payload.as_bytes(), &addr) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // Socket buffer full; non-blocking send avoids thread stall
            Ok(false)
        }
        Err(e) => Err(NotifyError::Io(e)),
    }
}

/// Convenience helper: notifies systemd that service is READY (`READY=1`).
pub fn notify_ready() -> Result<bool, NotifyError> {
    send_notify(&[NotifyState::Ready], false)
}

/// Convenience helper: notifies systemd of current status string (`STATUS=...`).
pub fn notify_status(status: impl Into<String>) -> Result<bool, NotifyError> {
    let s = status.into();
    if s.len() > 1024 {
        return Err(NotifyError::StatusTooLong(s.len()));
    }
    send_notify(&[NotifyState::Status(s)], false)
}

/// Convenience helper: sends watchdog keep-alive pulse (`WATCHDOG=1`).
pub fn notify_watchdog() -> Result<bool, NotifyError> {
    send_notify(&[NotifyState::Watchdog], false)
}

/// Convenience helper: notifies systemd that service is STOPPING (`STOPPING=1`).
pub fn notify_stopping() -> Result<bool, NotifyError> {
    send_notify(&[NotifyState::Stopping], false)
}

/// Convenience helper: notifies systemd that service is RELOADING (`RELOADING=1`).
pub fn notify_reloading() -> Result<bool, NotifyError> {
    send_notify(&[NotifyState::Reloading], false)
}
