//! JSON-RPC 2.0 message dispatcher for Model Context Protocol.

use crate::prompts::{generate_prompt, list_prompts};
use crate::protocol::error_codes::{INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR};
use crate::protocol::handshake::{handle_initialize, handle_ping};
use crate::protocol::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::resources::{list_resources, read_resource};
use crate::storage::McpState;
use crate::tools::{call_tool, list_tools};
use serde_json::json;

/// Stateless dispatcher routing JSON-RPC 2.0 requests to protocol handlers.
#[derive(Debug, Default, Clone)]
pub struct McpDispatcher;

impl McpDispatcher {
    /// Parses and dispatches a single raw JSON-RPC frame.
    ///
    /// Returns `Some(response)` for requests, or `None` for notifications and ignored frames.
    pub fn handle_message(state: &McpState, raw_json: &str) -> Option<JsonRpcResponse> {
        let trimmed = raw_json.trim();
        if trimmed.is_empty() {
            return None;
        }

        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                return Some(JsonRpcResponse::error(
                    None,
                    JsonRpcError::new(PARSE_ERROR, format!("Parse error: {e}")),
                ));
            }
        };

        if request.jsonrpc != "2.0" {
            return Some(JsonRpcResponse::error(
                request.id,
                JsonRpcError::new(INVALID_REQUEST, "Invalid JSON-RPC version: expected '2.0'"),
            ));
        }

        // Notification handler (no ID)
        if request.id.is_none() {
            tracing::debug!("Received MCP notification: {}", request.method);
            return None;
        }

        let id = request.id;
        let params = request.params.as_ref();

        let response = match request.method.as_str() {
            "initialize" => handle_initialize(id),
            "ping" => handle_ping(id),
            "tools/list" => JsonRpcResponse::success(id, json!({ "tools": list_tools() })),
            "tools/call" => {
                let name = match params.and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => {
                        return Some(JsonRpcResponse::error(
                            id,
                            JsonRpcError::new(INVALID_PARAMS, "Missing 'name' parameter for tools/call"),
                        ));
                    }
                };
                let args = params.and_then(|p| p.get("arguments"));
                let result = call_tool(state, name, args);
                JsonRpcResponse::success(id, result)
            }
            "resources/list" => {
                JsonRpcResponse::success(id, json!({ "resources": list_resources() }))
            }
            "resources/read" => {
                let uri = match params.and_then(|p| p.get("uri")).and_then(|v| v.as_str()) {
                    Some(u) => u,
                    None => {
                        return Some(JsonRpcResponse::error(
                            id,
                            JsonRpcError::new(INVALID_PARAMS, "Missing 'uri' parameter for resources/read"),
                        ));
                    }
                };
                match read_resource(state, uri) {
                    Ok(res) => JsonRpcResponse::success(id, res),
                    Err(err) => JsonRpcResponse::error(id, err),
                }
            }
            "prompts/list" => {
                JsonRpcResponse::success(id, json!({ "prompts": list_prompts() }))
            }
            "prompts/get" => {
                let name = match params.and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => {
                        return Some(JsonRpcResponse::error(
                            id,
                            JsonRpcError::new(INVALID_PARAMS, "Missing 'name' parameter for prompts/get"),
                        ));
                    }
                };
                let args = params.and_then(|p| p.get("arguments"));
                match generate_prompt(state, name, args) {
                    Ok(res) => JsonRpcResponse::success(id, res),
                    Err(err) => JsonRpcResponse::error(id, err),
                }
            }
            other => JsonRpcResponse::error(
                id,
                JsonRpcError::new(METHOD_NOT_FOUND, format!("Method not found: '{other}'")),
            ),
        };

        Some(response)
    }
}
