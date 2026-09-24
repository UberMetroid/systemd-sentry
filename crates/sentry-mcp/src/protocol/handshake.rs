//! MCP lifecycle handshake and connection initialization.

use crate::protocol::types::{JsonRpcResponse, RequestId};
use serde_json::json;

/// Standard MCP protocol version.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Handles the MCP `initialize` request.
pub fn handle_initialize(id: Option<RequestId>) -> JsonRpcResponse {
    let result = json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "resources": {
                "subscribe": false,
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "systemd-sentry-mcp",
            "version": "0.1.0"
        }
    });

    JsonRpcResponse::success(id, result)
}

/// Handles the standard MCP `ping` request.
pub fn handle_ping(id: Option<RequestId>) -> JsonRpcResponse {
    JsonRpcResponse::success(id, json!({}))
}
