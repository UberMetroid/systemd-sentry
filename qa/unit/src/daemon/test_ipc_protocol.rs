//! 1:1 QA tests for IPC protocol framing, serialization, and peer credentials authorization.

use sentry_daemon::ipc::peer_cred::{authorize_action, PeerCredentials};
use sentry_daemon::ipc::protocol::{IpcRequest, IpcResponse, MAX_IPC_FRAME_SIZE};

#[test]
fn test_ipc_request_serialization() {
    let req = IpcRequest::ResetCircuit {
        unit: "nginx.service".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: IpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, deserialized);
}

#[test]
fn test_ipc_response_serialization() {
    let resp = IpcResponse::Ok {
        data: serde_json::json!({ "uptime_seconds": 120, "rss_mb": 11.5 }),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: IpcResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, deserialized);

    let err_resp = IpcResponse::Error {
        code: -32003,
        message: "Permission denied".to_string(),
    };
    let err_json = serde_json::to_string(&err_resp).unwrap();
    let err_deserialized: IpcResponse = serde_json::from_str(&err_json).unwrap();
    assert_eq!(err_resp, err_deserialized);
}

#[test]
fn test_max_ipc_frame_size_constant() {
    assert_eq!(MAX_IPC_FRAME_SIZE, 32 * 1024);
}

#[test]
fn test_peer_credentials_authorization() {
    let daemon_uid = 995;

    // Read-only actions allowed for any caller
    let unpriv_creds = PeerCredentials { uid: 1000, gid: 1000, pid: Some(1234) };
    assert!(authorize_action(&unpriv_creds, daemon_uid, false).is_ok());

    // Mutating actions blocked for unprivileged user
    let err = authorize_action(&unpriv_creds, daemon_uid, true).unwrap_err();
    assert_eq!(err.0, -32003);

    // Mutating actions allowed for root (UID 0)
    let root_creds = PeerCredentials { uid: 0, gid: 0, pid: Some(5678) };
    assert!(authorize_action(&root_creds, daemon_uid, true).is_ok());

    // Mutating actions allowed for daemon UID
    let daemon_creds = PeerCredentials { uid: daemon_uid, gid: daemon_uid, pid: Some(9999) };
    assert!(authorize_action(&daemon_creds, daemon_uid, true).is_ok());
}
