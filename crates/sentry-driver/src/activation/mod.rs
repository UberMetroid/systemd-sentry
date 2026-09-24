//! Native systemd socket activation implementation.

pub mod disambiguate;
pub mod error;
pub mod fd_flags;
pub mod model;
pub mod name_parser;
pub mod parser;
pub mod pid_validator;
pub mod socket_inspector;

pub use disambiguate::disambiguate_socket;
pub use error::ActivationError;
pub use fd_flags::harden_activated_fd;
pub use model::ActivatedSocket;
pub use name_parser::parse_listen_fdnames;
pub use parser::{parse_listen_fds, SD_LISTEN_FDS_START};
pub use pid_validator::validate_listen_pid;
pub use socket_inspector::{get_socket_bound_path, is_socket, is_unix_stream_listener};


