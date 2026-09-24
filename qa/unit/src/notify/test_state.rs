//! 1:1 Unit QA tests for notify state formatting.

use sentry_driver::notify::NotifyState;

#[test]
fn test_notify_state_standard_formats() {
    assert_eq!(NotifyState::Ready.format_key_value(), "READY=1");
    assert_eq!(
        NotifyState::Status("operational".into()).format_key_value(),
        "STATUS=operational"
    );
    assert_eq!(NotifyState::Watchdog.format_key_value(), "WATCHDOG=1");
    assert_eq!(NotifyState::Reloading.format_key_value(), "RELOADING=1");
    assert_eq!(NotifyState::Stopping.format_key_value(), "STOPPING=1");
    assert_eq!(NotifyState::MainPid(12345).format_key_value(), "MAINPID=12345");
    assert_eq!(NotifyState::Errno(2).format_key_value(), "ERRNO=2");
    assert_eq!(
        NotifyState::BusError("org.err".into()).format_key_value(),
        "BUSERROR=org.err"
    );
    assert_eq!(
        NotifyState::ExtendTimeout(5000000).format_key_value(),
        "EXTEND_TIMEOUT_USEC=5000000"
    );
    assert_eq!(NotifyState::Barrier.format_key_value(), "BARRIER=1");
    assert_eq!(
        NotifyState::Custom("FOO".into(), "BAR".into()).format_key_value(),
        "FOO=BAR"
    );
}

#[test]
fn test_notify_state_equality_and_clone() {
    let s1 = NotifyState::Status("starting".into());
    let s2 = s1.clone();
    assert_eq!(s1, s2);
    assert_ne!(s1, NotifyState::Ready);
}
