//! Adversarial stress test demonstrating the non-UTF8 byte stream vulnerability.
//!
//! When non-UTF8 bytes are injected into `run_stdio_stream`, `tokio::io::Lines::next_line()`
//! returns `io::ErrorKind::InvalidData`.
//! In `crates/sentry-mcp/src/server/stdio.rs:21`, `next_line().await?` uses `?` which causes
//! the server loop to immediately abort and exit with `Err(InvalidData)`.
//! As an empirical consequence:
//! 1. No JSON-RPC 2.0 error object (e.g. PARSE_ERROR -32700) is returned to the client.
//! 2. The stdio stream terminates prematurely, crashing the MCP session on unparseable bytes.

use sentry_mcp::server::run_stdio_stream;
use sentry_mcp::storage::McpState;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn test_adversarial_non_utf8_stream_termination() {
    let state = McpState::new();
    let (client_write, server_read) = tokio::io::duplex(4096);
    let (server_write, client_read) = tokio::io::duplex(4096);

    let server_handle = tokio::spawn(async move {
        let reader = BufReader::new(server_read);
        run_stdio_stream(state, reader, server_write).await
    });

    let mut writer = client_write;
    let mut reader = BufReader::new(client_read);

    // 1. First send a valid ping to prove server is initially healthy
    writer
        .write_all(b"{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"ping\"}\n")
        .await
        .expect("write ping");
    writer.flush().await.expect("flush ping");

    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read ping response");
    assert!(line.contains("\"result\":{}"), "Initial ping must succeed");

    // 2. Inject non-UTF8 raw bytes followed by newline delimiter
    let non_utf8_payload = vec![0xFF, 0xFE, 0xFD, 0x80, b'\n'];
    writer
        .write_all(&non_utf8_payload)
        .await
        .expect("write non-utf8");
    writer.flush().await.expect("flush non-utf8");

    // 3. Observe client read: server sends 0 bytes and closes writer (EOF)
    line.clear();
    let bytes_read = reader.read_line(&mut line).await.expect("read after non-utf8");

    // EMPIRICAL VERIFICATION: Client receives EOF (0 bytes), NOT a JSON-RPC 2.0 error object!
    assert_eq!(
        bytes_read, 0,
        "Server failed to write JSON-RPC 2.0 error object; stream closed with EOF"
    );
    assert!(
        line.is_empty(),
        "Server output was empty instead of returning PARSE_ERROR"
    );

    // 4. Observe server task result: aborted with InvalidData IO error
    let server_result = server_handle.await.expect("join server");
    assert!(
        server_result.is_err(),
        "Expected server to return Err due to ? operator on next_line()"
    );
    let err = server_result.unwrap_err();
    assert_eq!(
        err.kind(),
        std::io::ErrorKind::InvalidData,
        "Server aborted with InvalidData error: '{err}'"
    );
}
