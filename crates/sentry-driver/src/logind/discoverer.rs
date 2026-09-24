//! Orchestrator for finding active graphical desktop sessions.

use super::error::LogindError;
use super::model::GraphicalSession;
use super::session_filter::filter_graphical_sessions;
use super::session_lister::list_login_sessions;
use super::user_bus::resolve_user_bus_address;
use zbus::Connection;

/// Discovers all active Wayland and X11 user sessions via systemd-logind.
pub async fn discover_active_graphical_sessions(
    conn: &Connection,
) -> Result<Vec<GraphicalSession>, LogindError> {
    let raw_sessions = list_login_sessions(conn).await?;
    let mut graphical_sessions = Vec::new();

    for (id, uid, user_name, seat, session_path) in raw_sessions {
        if let Ok(Some((session_type, active))) = filter_graphical_sessions(conn, &session_path).await {
            // Resolve user session bus path without strictly failing if unmounted
            let user_bus_path = resolve_user_bus_address(uid, false)?;

            graphical_sessions.push(GraphicalSession {
                id,
                uid,
                user_name,
                seat,
                session_type,
                active,
                user_bus_path,
            });
        }
    }

    Ok(graphical_sessions)
}
