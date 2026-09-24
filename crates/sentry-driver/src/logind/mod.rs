//! Pure Rust systemd-logind session discovery via `zbus`.

pub mod discoverer;
pub mod error;
pub mod model;
pub mod session_filter;
pub mod session_lister;
pub mod user_bus;

pub use discoverer::discover_active_graphical_sessions;
pub use error::LogindError;
pub use model::{GraphicalSession, SessionType};
pub use session_filter::filter_graphical_sessions;
pub use session_lister::{list_login_sessions, RawSessionInfo};
pub use user_bus::{resolve_user_bus_address, resolve_user_bus_address_with_base};
