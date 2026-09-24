//! IPC client for communicating with the running systemd-sentry supervisor.

use crate::ipc::protocol::{IpcRequest, IpcResponse, MAX_IPC_FRAME_SIZE};
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Client connection handle to the supervisor daemon.
pub struct IpcClient {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl IpcClient {
    /// Connect to the daemon's UNIX domain socket.
    pub async fn connect(socket_path: &str) -> Result<Self, Error> {
        let stream = UnixStream::connect(socket_path).await?;
        let (read_half, write_half) = stream.into_split();
        Ok(Self {
            reader: BufReader::new(read_half),
            writer: write_half,
        })
    }

    /// Send an IPC request and await the corresponding response.
    pub async fn send_request(&mut self, request: &IpcRequest) -> Result<IpcResponse, Error> {
        let mut payload = serde_json::to_vec(request).map_err(|e| {
            Error::new(ErrorKind::InvalidInput, format!("Serialization failed: {}", e))
        })?;
        payload.push(b'\n');

        self.writer.write_all(&payload).await?;
        self.writer.flush().await?;

        let mut line = String::new();
        let mut handle = (&mut self.reader).take((MAX_IPC_FRAME_SIZE + 1) as u64);
        let bytes_read = handle.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Daemon closed connection"));
        }
        if bytes_read > MAX_IPC_FRAME_SIZE {
            return Err(Error::new(ErrorKind::InvalidData, "IPC response exceeded 32 KiB frame limit"));
        }

        let response: IpcResponse = serde_json::from_str(&line).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Invalid IPC response JSON: {}", e))
        })?;

        Ok(response)
    }

    /// Split stream into reader for monitoring streaming events, retaining any buffered bytes.
    pub fn into_reader(self) -> BufReader<tokio::net::unix::OwnedReadHalf> {
        self.reader
    }
}
