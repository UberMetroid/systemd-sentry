//! Systemd-inhibit and logind shutdown/sleep integrations.

pub mod listener;
pub mod lock;

pub use listener::ShutdownWatcher;
pub use lock::InhibitorLock;
