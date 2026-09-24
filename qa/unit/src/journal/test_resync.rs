//! 1:1 Unit QA tests for journal stream resynchronization.

use sentry_driver::journal::resync_journal_stream;
use std::io::{BufReader, Cursor};

#[test]
fn test_resync_finds_record_separator() {
    let corrupted_data = b"random garbage\x00\xFF not valid data\n\nNEXT_KEY=value\n";
    let mut reader = BufReader::new(Cursor::new(corrupted_data));

    let skipped = resync_journal_stream(&mut reader).expect("Resync must find boundary");
    assert!(skipped > 0);

    let mut remaining = String::new();
    std::io::Read::read_to_string(&mut reader, &mut remaining).unwrap();
    assert_eq!(remaining, "NEXT_KEY=value\n");
}

#[test]
fn test_resync_on_clean_eof() {
    let empty_data = b"some garbage without double newline";
    let mut reader = BufReader::new(Cursor::new(empty_data));
    let skipped = resync_journal_stream(&mut reader).expect("Resync should handle EOF");
    assert_eq!(skipped, empty_data.len());
}
