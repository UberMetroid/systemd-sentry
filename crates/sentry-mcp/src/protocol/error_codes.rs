//! Standard JSON-RPC 2.0 and MCP protocol error codes.

/// JSON parsing error (-32700).
pub const PARSE_ERROR: i32 = -32700;

/// Invalid Request structure (-32600).
pub const INVALID_REQUEST: i32 = -32600;

/// Method not recognized or supported (-32601).
pub const METHOD_NOT_FOUND: i32 = -32601;

/// Invalid method parameters (-32602).
pub const INVALID_PARAMS: i32 = -32602;

/// Internal server error (-32603).
pub const INTERNAL_ERROR: i32 = -32603;
