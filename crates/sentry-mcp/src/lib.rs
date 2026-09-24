//! Model Context Protocol (MCP) server for systemd-sentry.
//!
//! Exposes:
//! - JSON-RPC 2.0 communication over standard input/output (stdio).
//! - Tools: `get_incident`, `list_incidents`, `get_unit_telemetry`, `explain_incident`.
//! - Resources: `incident://`, `telemetry://`, `policy://`, `circuit://`.
//! - Prompts: `triage_incident`, `analyze_flapping_service`.
//! - Zero network listeners (strictly 0 TCP ports opened!).
//!
//! 100% pure Rust, zero unsafe code, zero dynamic C library dependencies.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod storage;
pub mod tools;

pub use prompts::{generate_prompt, list_prompts};
pub use protocol::{
    handle_initialize, handle_ping, JsonRpcError, JsonRpcRequest, JsonRpcResponse, RequestId,
    INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, MCP_PROTOCOL_VERSION, METHOD_NOT_FOUND,
    PARSE_ERROR,
};
pub use resources::{list_resources, read_resource};
pub use server::{run_stdio_stream, serve_stdio, McpDispatcher};
pub use storage::McpState;
pub use tools::{call_tool, list_tools};
