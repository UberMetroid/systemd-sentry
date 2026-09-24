//! 1:1 Unit QA tests for JournalExportEntry.

use sentry_driver::journal::JournalExportEntry;

#[test]
fn test_journal_entry_accessors() {
    let mut entry = JournalExportEntry::new();
    entry.insert("MESSAGE".into(), b"Service crashed".to_vec());
    entry.insert("_SYSTEMD_UNIT".into(), b"app.service".to_vec());
    entry.insert("PRIORITY".into(), b"3".to_vec());
    entry.insert("__CURSOR".into(), b"s=abc123".to_vec());
    entry.insert("__REALTIME_TIMESTAMP".into(), b"1700000000000000".to_vec());
    entry.insert("__MONOTONIC_TIMESTAMP".into(), b"5000000".to_vec());

    assert_eq!(entry.message().unwrap(), "Service crashed");
    assert_eq!(entry.unit(), Some("app.service"));
    assert_eq!(entry.priority(), Some(3));
    assert_eq!(entry.cursor(), Some("s=abc123"));
    assert_eq!(entry.realtime_timestamp_usec(), Some(1700000000000000));
    assert_eq!(entry.monotonic_timestamp_usec(), Some(5000000));
    assert!(!entry.is_coredump());
}

#[test]
fn test_journal_entry_lossy_utf8() {
    let mut entry = JournalExportEntry::new();
    // Non-UTF8 byte sequence 0xFF 0xFE
    entry.insert("MESSAGE".into(), vec![0xFF, 0xFE, b'h', b'i']);
    assert!(entry.message().unwrap().contains('\u{FFFD}'));
}

#[test]
fn test_journal_entry_coredump_detection() {
    let mut entry = JournalExportEntry::new();
    entry.insert(
        "MESSAGE_ID".into(),
        b"fc2e22bc6ee647b6b90729ab34a250b1".to_vec(),
    );
    assert!(entry.is_coredump());

    let mut entry2 = JournalExportEntry::new();
    entry2.insert("COREDUMP_SIGNAL".into(), b"11".to_vec());
    assert!(entry2.is_coredump());
}
