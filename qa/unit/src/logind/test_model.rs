//! 1:1 Unit QA tests for logind models.

use sentry_driver::logind::{GraphicalSession, SessionType};
use std::path::PathBuf;

#[test]
fn test_session_type_is_graphical() {
    assert!(SessionType::Wayland.is_graphical());
    assert!(SessionType::X11.is_graphical());
    assert!(!SessionType::Tty.is_graphical());
    assert!(!SessionType::Other("unspecified".into()).is_graphical());
}

#[test]
fn test_graphical_session_model() {
    let session = GraphicalSession {
        id: "2".into(),
        uid: 1000,
        user_name: "testuser".into(),
        seat: "seat0".into(),
        session_type: SessionType::Wayland,
        active: true,
        user_bus_path: PathBuf::from("/run/user/1000/bus"),
    };

    assert_eq!(session.uid, 1000);
    assert!(session.active);
    assert!(session.session_type.is_graphical());
}
