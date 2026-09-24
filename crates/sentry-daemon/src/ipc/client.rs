//! IPC client for communicating with the running systemd-sentry supervisor.

use crate::ipc::protocol::{IpcRequest, IpcResponse, MAX_IPC_FRAME_SIZE};
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Client connection handle to the supervisor daemon.
pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    /// Connect to the daemon's UNIX domain socket.
    pub async fn connect(socket_path: &str) -> Result<Self, Error> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream })
    }

    /// Send an IPC request and await the corresponding response.
    pub async fn send_request(&mut self, request: &IpcRequest) -> Result<IpcResponse, Error> {
        let mut payload = serde_json::to_vec(request).map_err(|e| {
            Error::new(ErrorKind::InvalidInput, format!("Serialization failed: {}", e))
        })?;
        payload.push(b'\n');

        self.stream.write_all(&payload).await?;
        self.stream.flush().await?;

        let (reader, _) = self.stream.split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        let bytes_read = buf_reader.read_line(&mut line).await?;
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

    /// Split stream into reader for monitoring streaming events.
    pub fn into_reader(self) -> BufReader<tokio::net::unix::OwnedReadHalf> {
        let (reader, _) = self.stream.into_split();
        BufReader::new(reader)
    }
}
