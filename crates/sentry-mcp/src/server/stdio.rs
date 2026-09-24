//! Standard IO (stdio) transport listener for Model Context Protocol.

use crate::protocol::error_codes::PARSE_ERROR;
use crate::protocol::types::{JsonRpcError, JsonRpcResponse};
use crate::server::dispatcher::McpDispatcher;
use crate::storage::McpState;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

/// Maximum allowable frame size for incoming MCP JSON-RPC messages (1 MiB).
pub const MAX_MCP_FRAME_SIZE: usize = 1024 * 1024;

/// Runs the MCP JSON-RPC 2.0 loop over generic asynchronous streams.
///
/// Ensures zero TCP/network listeners are opened, strictly streaming newline-delimited JSON.
/// Resilient against non-UTF8 byte sequences, returning JSON-RPC PARSE_ERROR (-32700).
pub async fn run_stdio_stream<R, W>(
    state: McpState,
    mut reader: R,
    mut writer: W,
) -> Result<(), std::io::Error>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let mut bytes_read = 0;
        let mut exceeded = false;

        loop {
            let available = reader.fill_buf().await?;
            if available.is_empty() {
                break;
            }
            if let Some(pos) = available.iter().position(|&b| b == b'\n') {
                let to_take = pos + 1;
                if !exceeded && bytes_read + to_take <= MAX_MCP_FRAME_SIZE {
                    buf.extend_from_slice(&available[..to_take]);
                } else {
                    exceeded = true;
                }
                bytes_read += to_take;
                reader.consume(to_take);
                break;
            } else {
                let len = available.len();
                if !exceeded && bytes_read + len <= MAX_MCP_FRAME_SIZE {
                    buf.extend_from_slice(available);
                } else {
                    exceeded = true;
                }
                bytes_read += len;
                reader.consume(len);
            }
        }

        if bytes_read == 0 {
            // Clean EOF
            break;
        }

        if exceeded || bytes_read > MAX_MCP_FRAME_SIZE {
            let error_resp = JsonRpcResponse::error(
                None,
                JsonRpcError::new(PARSE_ERROR, "Parse error: frame size exceeded 1 MiB limit"),
            );
            let mut serialized = serde_json::to_string(&error_resp)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            serialized.push('\n');
            writer.write_all(serialized.as_bytes()).await?;
            writer.flush().await?;
            continue;
        }

        let line = match std::str::from_utf8(&buf) {
            Ok(s) => s,
            Err(_) => {
                let error_resp = JsonRpcResponse::error(
                    None,
                    JsonRpcError::new(PARSE_ERROR, "Parse error: stream did not contain valid UTF-8"),
                );
                let mut serialized = serde_json::to_string(&error_resp)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                serialized.push('\n');

                writer.write_all(serialized.as_bytes()).await?;
                writer.flush().await?;
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        if let Some(resp) = McpDispatcher::handle_message(&state, line) {
            let mut serialized = serde_json::to_string(&resp)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            serialized.push('\n');

            writer.write_all(serialized.as_bytes()).await?;
            writer.flush().await?;
        }
    }

    Ok(())
}

/// Runs the MCP server bound to host standard input and standard output.
pub async fn serve_stdio(state: McpState) -> Result<(), std::io::Error> {
    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let stdout = tokio::io::stdout();

    run_stdio_stream(state, reader, stdout).await
}
