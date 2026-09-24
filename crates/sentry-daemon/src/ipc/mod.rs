//! Inter-process communication via UNIX domain sockets.

pub mod client;
pub mod connection_handler;
pub mod listener;
pub mod peer_cred;
pub mod protocol;

pub use client::IpcClient;
pub use connection_handler::handle_ipc_connection;
pub use listener::bind_or_activate_socket;
pub use peer_cred::{authorize_action, get_peer_credentials, PeerCredentials};
pub use protocol::{IpcRequest, IpcResponse, MAX_IPC_FRAME_SIZE};
