//! 1:1 Unit QA tests for notify wire encoding.

use sentry_driver::notify::{encode_notify_payload, NotifyState};

#[test]
fn test_encode_single_state() {
    let states = [NotifyState::Ready];
    let payload = encode_notify_payload(&states);
    assert_eq!(payload, "READY=1\n");
}

#[test]
fn test_encode_multiple_states() {
    let states = [
        NotifyState::Ready,
        NotifyState::Status("Sentry monitoring active".into()),
        NotifyState::MainPid(999),
    ];
    let payload = encode_notify_payload(&states);
    assert_eq!(
        payload,
        "READY=1\nSTATUS=Sentry monitoring active\nMAINPID=999\n"
    );
}

#[test]
fn test_encode_newline_sanitization() {
    let states = [NotifyState::Status("Line1\nLine2\r\nLine3".into())];
    let payload = encode_notify_payload(&states);
    assert!(!payload.contains("STATUS=Line1\nLine2"));
    assert_eq!(payload, "STATUS=Line1 Line2\r Line3\n");
}

#[test]
fn test_encode_empty_states() {
    let states: [NotifyState; 0] = [];
    let payload = encode_notify_payload(&states);
    assert!(payload.is_empty());
}
