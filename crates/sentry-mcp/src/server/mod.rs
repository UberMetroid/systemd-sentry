//! MCP Server transport and dispatch implementations.

pub mod dispatcher;
pub mod stdio;

pub use dispatcher::McpDispatcher;
pub use stdio::{run_stdio_stream, serve_stdio};
