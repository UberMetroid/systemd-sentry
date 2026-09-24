//! 1:1 Unit QA tests for JournalExportParser.

use sentry_driver::journal::JournalExportParser;
use std::io::{BufReader, Cursor};

#[test]
fn test_parse_single_text_entry() {
    let stream = b"MESSAGE=Service failed\n_SYSTEMD_UNIT=test.service\nPRIORITY=3\n\n";
    let mut parser = JournalExportParser::new(BufReader::new(Cursor::new(stream)));

    let entry = parser
        .parse_next_entry()
        .unwrap()
        .expect("Must parse entry");
    assert_eq!(entry.message().unwrap(), "Service failed");
    assert_eq!(entry.unit(), Some("test.service"));
    assert_eq!(entry.priority(), Some(3));

    assert!(parser.parse_next_entry().unwrap().is_none());
}

#[test]
fn test_parse_interleaved_binary_and_text_fields() {
    let mut data = Vec::new();
    data.extend_from_slice(b"_SYSTEMD_UNIT=demo.service\n");
    data.extend_from_slice(b"MESSAGE\n");
    let payload = b"Multiline error\nStack trace line\n";
    let len = payload.len() as u64;
    data.extend_from_slice(&len.to_le_bytes());
    data.extend_from_slice(payload);
    data.push(b'\n'); // trailing newline
    data.extend_from_slice(b"PRIORITY=2\n\n");

    let mut parser = JournalExportParser::new(BufReader::new(Cursor::new(data)));
    let entry = parser
        .parse_next_entry()
        .unwrap()
        .expect("Must parse entry");

    assert_eq!(entry.unit(), Some("demo.service"));
    assert_eq!(
        entry.message().unwrap(),
        "Multiline error\nStack trace line\n"
    );
    assert_eq!(entry.priority(), Some(2));
}

#[test]
fn test_parse_multiple_entries() {
    let stream = b"KEY1=VAL1\n\nKEY2=VAL2\n\n";
    let mut parser = JournalExportParser::new(BufReader::new(Cursor::new(stream)));

    let e1 = parser.parse_next_entry().unwrap().unwrap();
    assert_eq!(e1.get_str("KEY1"), Some("VAL1"));

    let e2 = parser.parse_next_entry().unwrap().unwrap();
    assert_eq!(e2.get_str("KEY2"), Some("VAL2"));

    assert!(parser.parse_next_entry().unwrap().is_none());
}
