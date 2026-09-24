//! In-memory representation of a Journal Export entry.

use std::borrow::Cow;
use std::collections::HashMap;

/// Parsed entry from systemd's Journal Export Format.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JournalExportEntry {
    /// Mapping of field names to raw byte values.
    pub fields: HashMap<String, Vec<u8>>,
}

impl JournalExportEntry {
    /// Creates an empty journal entry.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Inserts a field into the entry.
    pub fn insert(&mut self, key: String, value: Vec<u8>) {
        self.fields.insert(key, value);
    }

    /// Gets raw byte slice for a field.
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.fields.get(key).map(|v| v.as_slice())
    }

    /// Gets field value as valid UTF-8 string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| std::str::from_utf8(v).ok())
    }

    /// Gets field value using lossy UTF-8 decoding.
    pub fn get_str_lossy(&self, key: &str) -> Option<Cow<'_, str>> {
        self.get(key).map(String::from_utf8_lossy)
    }

    /// Convenience getter for `MESSAGE` field with lossy UTF-8 conversion.
    pub fn message(&self) -> Option<Cow<'_, str>> {
        self.get_str_lossy("MESSAGE")
    }

    /// Convenience getter for systemd unit name (`_SYSTEMD_UNIT` or `COREDUMP_UNIT`).
    pub fn unit(&self) -> Option<&str> {
        self.get_str("_SYSTEMD_UNIT")
            .or_else(|| self.get_str("COREDUMP_UNIT"))
    }

    /// Convenience getter for syslog `PRIORITY` numeric level.
    pub fn priority(&self) -> Option<u8> {
        self.get_str("PRIORITY").and_then(|s| s.parse::<u8>().ok())
    }

    /// Convenience getter for journal `__CURSOR` token.
    pub fn cursor(&self) -> Option<&str> {
        self.get_str("__CURSOR")
    }

    /// Convenience getter for `__REALTIME_TIMESTAMP` in microseconds.
    pub fn realtime_timestamp_usec(&self) -> Option<u64> {
        self.get_str("__REALTIME_TIMESTAMP")
            .and_then(|s| s.parse::<u64>().ok())
    }

    /// Convenience getter for `__MONOTONIC_TIMESTAMP` in microseconds.
    pub fn monotonic_timestamp_usec(&self) -> Option<u64> {
        self.get_str("__MONOTONIC_TIMESTAMP")
            .and_then(|s| s.parse::<u64>().ok())
    }

    /// Returns true if this journal entry represents a crash recorded by systemd-coredump.
    pub fn is_coredump(&self) -> bool {
        self.get_str("MESSAGE_ID") == Some("fc2e22bc6ee647b6b90729ab34a250b1")
            || self.fields.contains_key("COREDUMP_SIGNAL")
    }
}
