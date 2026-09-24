//! Stream resynchronization logic for corrupted journal streams.

use sentry_core::error::JournalError;
use std::io::BufRead;

/// Scans forward in a corrupted journal export stream until consecutive newlines (`\n\n`)
/// are encountered, synchronizing on the next entry boundary.
pub fn resync_journal_stream<R: BufRead>(reader: &mut R) -> Result<usize, JournalError> {
    let mut skipped_bytes = 0usize;
    let mut last_byte_was_newline = false;

    loop {
        let available = reader.fill_buf().map_err(JournalError::Io)?;
        if available.is_empty() {
            return Ok(skipped_bytes); // Clean EOF during resync
        }

        let mut consumed = 0;
        let mut found_sync = false;

        for &b in available {
            consumed += 1;
            skipped_bytes += 1;
            if b == b'\n' {
                if last_byte_was_newline {
                    found_sync = true;
                    break;
                }
                last_byte_was_newline = true;
            } else {
                last_byte_was_newline = false;
            }
        }

        reader.consume(consumed);
        if found_sync {
            return Ok(skipped_bytes);
        }
    }
}
