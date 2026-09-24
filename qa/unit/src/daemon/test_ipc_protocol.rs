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

    let reload_req = IpcRequest::ReloadConfig;
    let reload_json = serde_json::to_string(&reload_req).unwrap();
    let reload_deser: IpcRequest = serde_json::from_str(&reload_json).unwrap();
    assert_eq!(reload_req, reload_deser);
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

#[test]
fn test_status_circuit_breakers_payload_compatibility() {
    use sentry_safety::circuit::CircuitStateSnapshot;
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(
        "api-worker.service".to_string(),
        CircuitStateSnapshot {
            state: "OPEN".to_string(),
            cooldown_remaining_secs: 42,
            recent_failures: 3,
            permanently_locked: false,
        },
    );

    let status_val = serde_json::json!({
        "uptime_seconds": 3600,
        "rss_mb": 12.3,
        "degraded_mode": false,
        "dropped_events": 0,
        "tracked_units_count": 1,
        "circuit_breakers": map,
    });

    let breakers_obj = status_val.get("circuit_breakers").and_then(|v| v.as_object()).unwrap();
    let b = breakers_obj.get("api-worker.service").unwrap();
    assert_eq!(b.get("state").and_then(|v| v.as_str()), Some("OPEN"));
    assert_eq!(b.get("recent_failures").and_then(|v| v.as_u64()), Some(3));
    assert_eq!(b.get("permanently_locked").and_then(|v| v.as_bool()), Some(false));
}

#[test]
fn test_inspect_and_incidents_diagnostic_payload_extraction() {
    use sentry_core::models::{DiagnosticPayload, Evidence, ProposedRemediation, RemediationAction, RiskLevel, RootCause, Severity};

    let payload = DiagnosticPayload {
        incident_id: uuid::Uuid::nil(),
        timestamp: chrono::Utc::now(),
        unit_name: "db.service".to_string(),
        root_cause: RootCause {
            summary: "Out of memory error".to_string(),
            detail: "Killed by OOM killer due to memory exhaustion".to_string(),
        },
        evidence: Evidence {
            journal_lines: vec!["oom-killer invoked".to_string()],
            exit_code: Some(137),
            signal: Some("SIGKILL".to_string()),
            coredump: None,
            psi: None,
            cgroup: None,
        },
        severity: Severity::Critical,
        proposed_remediation: ProposedRemediation {
            action: RemediationAction::RestartWithBackoff,
            rationale: "Backoff restart to avoid thrashing".to_string(),
            risk_level: RiskLevel::Medium,
            confidence: 0.95,
        },
    };

    let val = serde_json::to_value(&payload).unwrap();

    // Verify incidents table root_cause extraction
    let cause = val.get("root_cause")
        .and_then(|v| v.as_str().or_else(|| v.get("summary").and_then(|s| s.as_str())))
        .unwrap();
    assert_eq!(cause, "Out of memory error");

    // Verify inspect field extraction
    let unit = val.get("unit_name").and_then(|v| v.as_str()).unwrap();
    assert_eq!(unit, "db.service");
    let detail = val.get("root_cause").unwrap().get("detail").and_then(|v| v.as_str()).unwrap();
    assert_eq!(detail, "Killed by OOM killer due to memory exhaustion");
    let exit_code = val.get("evidence").unwrap().get("exit_code").and_then(|v| v.as_i64()).unwrap();
    assert_eq!(exit_code, 137);
}

#[tokio::test]
async fn test_ipc_client_request_response_and_streaming() {
    use sentry_daemon::ipc::client::IpcClient;
    use sentry_daemon::ipc::protocol::{IpcRequest, IpcResponse};
    use tempfile::tempdir;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let dir = tempdir().unwrap();
    let sock_path = dir.path().join("test.sock");
    let sock_str = sock_path.to_str().unwrap().to_string();

    let listener = UnixListener::bind(&sock_path).unwrap();

    let server_task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let (read_half, mut write_half) = stream.split();
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut line = String::new();

        // 1. Respond to Ping
        reader.read_line(&mut line).await.unwrap();
        let resp = serde_json::to_vec(&IpcResponse::Ok { data: serde_json::json!({"pong": true}) }).unwrap();
        write_half.write_all(&resp).await.unwrap();
        write_half.write_all(b"\n").await.unwrap();
        line.clear();

        // 2. Respond to Subscribe + immediately send Event in same write burst
        reader.read_line(&mut line).await.unwrap();
        let sub_resp = serde_json::to_vec(&IpcResponse::Ok { data: serde_json::json!({"subscribed": true}) }).unwrap();
        let ev_resp = serde_json::to_vec(&IpcResponse::Event { data: serde_json::json!({"unit": "web.service"}) }).unwrap();
        write_half.write_all(&sub_resp).await.unwrap();
        write_half.write_all(b"\n").await.unwrap();
        write_half.write_all(&ev_resp).await.unwrap();
        write_half.write_all(b"\n").await.unwrap();
    });

    let mut client = IpcClient::connect(&sock_str).await.unwrap();

    // 1. Send Ping
    let res = client.send_request(&IpcRequest::Ping).await.unwrap();
    assert!(matches!(res, IpcResponse::Ok { .. }));

    // 2. Send SubscribeEvents and verify stream retains buffered events
    let sub_res = client.send_request(&IpcRequest::SubscribeEvents).await.unwrap();
    assert!(matches!(sub_res, IpcResponse::Ok { .. }));

    let mut event_reader = client.into_reader();
    let mut ev_line = String::new();
    let n = event_reader.read_line(&mut ev_line).await.unwrap();
    assert!(n > 0);
    let parsed: IpcResponse = serde_json::from_str(&ev_line).unwrap();
    if let IpcResponse::Event { data } = parsed {
        assert_eq!(data.get("unit").and_then(|v| v.as_str()), Some("web.service"));
    } else {
        panic!("Expected IpcResponse::Event, got {:?}", parsed);
    }

    server_task.await.unwrap();
}
