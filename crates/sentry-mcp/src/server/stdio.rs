//! Standard IO (stdio) transport listener for Model Context Protocol.

use crate::server::dispatcher::McpDispatcher;
use crate::storage::McpState;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

/// Runs the MCP JSON-RPC 2.0 loop over generic asynchronous streams.
///
/// Ensures zero TCP/network listeners are opened, strictly streaming newline-delimited JSON.
pub async fn run_stdio_stream<R, W>(
    state: McpState,
    reader: R,
    mut writer: W,
) -> Result<(), std::io::Error>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(resp) = McpDispatcher::handle_message(&state, &line) {
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
