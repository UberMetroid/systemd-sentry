//! Native systemd socket activation implementation.

pub mod error;
pub mod fd_flags;
pub mod model;
pub mod name_parser;
pub mod parser;
pub mod pid_validator;

pub use error::ActivationError;
pub use fd_flags::harden_activated_fd;
pub use model::ActivatedSocket;
pub use name_parser::parse_listen_fdnames;
pub use parser::{parse_listen_fds, SD_LISTEN_FDS_START};
pub use pid_validator::validate_listen_pid;
