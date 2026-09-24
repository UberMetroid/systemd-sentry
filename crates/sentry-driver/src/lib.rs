//! Pure Rust host drivers and systemd integrations for systemd-sentry.
//!
//! 100% pure Rust, zero unsafe code in driver logic, zero dynamic C libraries.

#![deny(missing_docs)]

pub mod activation;
pub mod cgroup;
pub mod coredump;
pub mod dbus;
pub mod inhibit;
pub mod journal;
pub mod logind;
pub mod networkd;
pub mod notify;
pub mod pstore;
pub mod psi;
pub mod resolved;
pub mod timesync;
pub mod user_session;

pub use activation::{
    parse_listen_fds, parse_listen_fdnames, validate_listen_pid, ActivatedSocket, ActivationError,
    SD_LISTEN_FDS_START,
};
pub use cgroup::{
    collect_cgroup_telemetry, locate_unit_cgroup, read_cgroup_cpu, read_cgroup_io,
    read_cgroup_memory,
};
pub use coredump::{extract_backtrace, find_latest_coredump, match_coredump_record, read_coredump_xattrs};
pub use dbus::{
    decode_unit_properties, escape_unit_name, extract_unit_failed_event,
    object_path_to_unit_name, unescape_unit_name, unit_name_to_object_path, DbusDriverError,
    DbusEvent, SystemdDbusListener, UnitFailedEvent, UnitStateUpdate,
};
pub use journal::{
    read_binary_field, resync_journal_stream, JournalExportEntry, JournalExportParser,
    JournalStreamReader,
};
pub use logind::{
    discover_active_graphical_sessions, filter_graphical_sessions, list_login_sessions,
    resolve_user_bus_address, GraphicalSession, LogindError, SessionType,
};
pub use notify::{
    encode_notify_payload, notify_ready, notify_reloading, notify_status, notify_stopping,
    notify_watchdog, parse_watchdog_config, resolve_notify_address, send_notify, NotifyError,
    NotifyState, WatchdogConfig, WatchdogTicker,
};
pub use psi::{
    collect_cgroup_psi, collect_system_psi, parse_psi_line, parse_psi_record, read_psi_file,
};
pub use inhibit::{InhibitorLock, ShutdownWatcher};
pub use networkd::NetworkdClient;
pub use pstore::{
    extract_panic_report, parse_pstore_file, scan_directory, scan_pstore_reports,
    MAX_PSTORE_REPORTS,
};
pub use resolved::ResolvedClient;
pub use timesync::TimesyncClient;
pub use user_session::UserSessionDiscoverer;
