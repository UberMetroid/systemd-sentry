//! Models representing discovered user desktop sessions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Graphical session display protocol type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    /// Wayland compositor session.
    Wayland,
    /// X11 display server session.
    X11,
    /// Terminal or text console session.
    Tty,
    /// Other unclassified session type.
    Other(String),
}

impl SessionType {
    /// Returns true if this session type is a graphical display protocol.
    pub fn is_graphical(&self) -> bool {
        matches!(self, Self::Wayland | Self::X11)
    }
}

/// Metadata describing an active graphical desktop session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphicalSession {
    /// Logind session ID (e.g. "2", "c1").
    pub id: String,
    /// Numeric POSIX user ID of session owner.
    pub uid: u32,
    /// Username of session owner.
    pub user_name: String,
    /// Hardware seat ID (e.g. "seat0").
    pub seat: String,
    /// Session display protocol type.
    pub session_type: SessionType,
    /// Whether the session is currently active (focused).
    pub active: bool,
    /// Path to user session D-Bus socket (`/run/user/<uid>/bus`).
    pub user_bus_path: PathBuf,
}
