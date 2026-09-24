//! Queries systemd-logind for active user sessions.

use super::error::LogindError;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

/// Raw session summary tuple returned by `org.freedesktop.login1.Manager.ListSessions`.
pub type RawSessionInfo = (String, u32, String, String, OwnedObjectPath);

/// Queries `org.freedesktop.login1.Manager.ListSessions` over the system D-Bus.
pub async fn list_login_sessions(conn: &Connection) -> Result<Vec<RawSessionInfo>, LogindError> {
    let reply = conn
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "ListSessions",
            &(),
        )
        .await?;

    let sessions: Vec<RawSessionInfo> = reply.body().deserialize()?;
    Ok(sessions)
}
