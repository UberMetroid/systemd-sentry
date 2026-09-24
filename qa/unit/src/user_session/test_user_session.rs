//! Tests for rootless user session and bus discoverer.

use sentry_driver::user_session::UserSessionDiscoverer;
use std::fs::{create_dir_all, File};
use tempfile::tempdir;

#[test]
fn test_discover_active_uids_bounded() {
    let temp = tempdir().expect("tempdir");
    let base = temp.path();

    // Create 70 mock user dirs (exceeding MAX_DISCOVERED_UIDS = 64)
    for uid in 1000..1070 {
        let user_dir = base.join(format!("{uid}"));
        create_dir_all(&user_dir).expect("create dir");
        // Create mock bus socket file
        let bus_file = user_dir.join("bus");
        File::create(&bus_file).expect("create bus file");
    }

    let uids = UserSessionDiscoverer::discover_active_uids_in_path(base);
    assert_eq!(uids.len(), UserSessionDiscoverer::MAX_DISCOVERED_UIDS);
    assert_eq!(uids[0], 1000);
}

#[test]
fn test_resolve_user_socket_path() {
    let path = UserSessionDiscoverer::resolve_user_socket(1001);
    assert_eq!(path.to_str(), Some("/run/user/1001/bus"));
}
