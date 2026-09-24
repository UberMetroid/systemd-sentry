//! Resolves user session D-Bus socket paths.

use super::error::LogindError;
use std::path::{Path, PathBuf};

/// Resolves the standard user D-Bus socket path for a given UID (`/run/user/<uid>/bus`).
///
/// If `require_exists` is true, verifies that the socket path exists on the filesystem.
pub fn resolve_user_bus_address(uid: u32, require_exists: bool) -> Result<PathBuf, LogindError> {
    let candidate = PathBuf::from(format!("/run/user/{uid}/bus"));
    if require_exists && !candidate.exists() {
        return Err(LogindError::UserBusNotFound(uid, candidate));
    }
    Ok(candidate)
}

/// Resolves user session D-Bus socket with a custom base directory (useful for testing).
pub fn resolve_user_bus_address_with_base(
    base: &Path,
    uid: u32,
    require_exists: bool,
) -> Result<PathBuf, LogindError> {
    let candidate = base.join(format!("user/{uid}/bus"));
    if require_exists && !candidate.exists() {
        return Err(LogindError::UserBusNotFound(uid, candidate));
    }
    Ok(candidate)
}
