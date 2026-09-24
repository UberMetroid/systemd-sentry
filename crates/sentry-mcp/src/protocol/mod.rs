//! MCP JSON-RPC 2.0 protocol specifications and handling.

pub mod error_codes;
pub mod handshake;
pub mod types;

pub use error_codes::*;
pub use handshake::{handle_initialize, handle_ping, MCP_PROTOCOL_VERSION};
pub use types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RequestId};
