//! Fuzzing reference decoders and target harnesses for systemd-sentry.
//!
//! Provides protocol parsers for journal export streams, D-Bus message framing,
//! PSI pressure text metrics, and JSON diagnostic payloads.

pub mod journal;
pub mod dbus;
pub mod psi;
pub mod triage;

pub use journal::parse_journal_export_entry;
pub use dbus::decode_dbus_message_header;
pub use psi::parse_psi_record;
pub use triage::parse_and_validate_diagnostic_json;
