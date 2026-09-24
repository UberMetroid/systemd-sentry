//! Pure Rust `sd_notify` socket writer and background watchdog ticker.

pub mod encoder;
pub mod error;
pub mod sender;
pub mod socket_addr;
pub mod state;
pub mod watchdog_config;
pub mod watchdog_ticker;

pub use encoder::encode_notify_payload;
pub use error::NotifyError;
pub use sender::{
    notify_ready, notify_reloading, notify_status, notify_stopping, notify_watchdog, send_notify,
};
pub use socket_addr::resolve_notify_address;
pub use state::NotifyState;
pub use watchdog_config::{parse_watchdog_config, WatchdogConfig};
pub use watchdog_ticker::WatchdogTicker;
