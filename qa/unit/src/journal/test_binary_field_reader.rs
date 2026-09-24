//! 1:1 Unit QA tests for journal binary field reader.

use sentry_driver::journal::read_binary_field;
use std::io::{BufReader, Cursor};

#[test]
fn test_read_binary_field_success() {
    let mut data = Vec::new();
    let payload = b"line1\nline2\n";
    let len = payload.len() as u64;
    data.extend_from_slice(&len.to_le_bytes());
    data.extend_from_slice(payload);
    data.push(b'\n'); // trailing newline

    let mut reader = BufReader::new(Cursor::new(data));
    let (name, read_payload) =
        read_binary_field(&mut reader, "MESSAGE".into(), 1024).expect("Read must succeed");

    assert_eq!(name, "MESSAGE");
    assert_eq!(read_payload, payload);
}

#[test]
fn test_read_binary_field_oversize_rejected() {
    let mut data = Vec::new();
    let len = 2048u64;
    data.extend_from_slice(&len.to_le_bytes());

    let mut reader = BufReader::new(Cursor::new(data));
    let err = read_binary_field(&mut reader, "PAYLOAD".into(), 1024).unwrap_err();
    assert!(err.to_string().contains("exceeds maximum limit"));
}

#[test]
fn test_read_binary_field_invalid_delimiter() {
    let mut data = Vec::new();
    let payload = b"hello";
    let len = payload.len() as u64;
    data.extend_from_slice(&len.to_le_bytes());
    data.extend_from_slice(payload);
    data.push(b'X'); // Not a newline!

    let mut reader = BufReader::new(Cursor::new(data));
    let err = read_binary_field(&mut reader, "DATA".into(), 1024).unwrap_err();
    assert!(err.to_string().contains("Invalid binary field delimiter"));
}
