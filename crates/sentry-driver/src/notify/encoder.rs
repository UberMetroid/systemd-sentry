//! Encodes notification states into wire protocol format.

use super::state::NotifyState;

/// Encodes a slice of `NotifyState` items into a protocol payload,
/// sanitizing newlines and ensuring trailing newline.
pub fn encode_notify_payload(states: &[NotifyState]) -> String {
    let mut payload = String::with_capacity(128);
    for state in states {
        let kv = state.format_key_value();
        let sanitized = kv.replace('\n', " ");
        payload.push_str(&sanitized);
        payload.push('\n');
    }
    payload
}
