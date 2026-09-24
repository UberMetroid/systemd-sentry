//! Filters logind sessions by graphical display type and active status.

use super::error::LogindError;
use super::model::SessionType;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

/// Inspects properties on a session object to determine if it is an active graphical session.
pub async fn filter_graphical_sessions(
    conn: &Connection,
    path: &OwnedObjectPath,
) -> Result<Option<(SessionType, bool)>, LogindError> {
    // 1. Query "Type" property
    let type_reply = conn
        .call_method(
            Some("org.freedesktop.login1"),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.login1.Session", "Type"),
        )
        .await;

    let type_str: String = match type_reply {
        Ok(reply) => {
            let val: zbus::zvariant::OwnedValue = reply.body().deserialize()?;
            <&str>::try_from(&val).map(|s| s.to_string()).unwrap_or_default()
        }
        Err(_) => return Ok(None),
    };

    let session_type = match type_str.to_lowercase().as_str() {
        "wayland" => SessionType::Wayland,
        "x11" => SessionType::X11,
        "tty" => SessionType::Tty,
        other => SessionType::Other(other.to_string()),
    };

    if !session_type.is_graphical() {
        return Ok(None);
    }

    // 2. Query "Active" property
    let active_reply = conn
        .call_method(
            Some("org.freedesktop.login1"),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.login1.Session", "Active"),
        )
        .await;

    let is_active: bool = match active_reply {
        Ok(reply) => {
            let val: zbus::zvariant::OwnedValue = reply.body().deserialize()?;
            <bool>::try_from(&val).unwrap_or(false)
        }
        Err(_) => false,
    };

    Ok(Some((session_type, is_active)))
}
