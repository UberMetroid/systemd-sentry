//! Streaming state machine parser for systemd Journal Export Format.

use super::binary_field_reader::{read_binary_field, MAX_FIELD_SIZE};
use super::export_entry::JournalExportEntry;
use super::resync::resync_journal_stream;
use sentry_core::error::JournalError;
use std::io::BufRead;

/// Parser for systemd Journal Export streams.
pub struct JournalExportParser<R> {
    reader: R,
    max_field_size: usize,
}

impl<R: BufRead> JournalExportParser<R> {
    /// Creates a new `JournalExportParser` wrapping the given reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            max_field_size: MAX_FIELD_SIZE,
        }
    }

    /// Creates a parser with a custom maximum binary field size.
    pub fn with_max_field_size(reader: R, max_field_size: usize) -> Self {
        Self {
            reader,
            max_field_size,
        }
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Parses the next journal entry from the stream.
    ///
    /// Returns `Ok(Some(entry))` when an entry is parsed, `Ok(None)` on clean EOF,
    /// or `Err(JournalError)` if format is corrupted (automatically resyncing stream).
    pub fn parse_next_entry(&mut self) -> Result<Option<JournalExportEntry>, JournalError> {
        let mut entry = JournalExportEntry::new();
        let mut line_buf = String::new();

        loop {
            line_buf.clear();
            let bytes_read = self.reader.read_line(&mut line_buf).map_err(JournalError::Io)?;

            if bytes_read == 0 {
                // EOF reached
                if entry.fields.is_empty() {
                    return Ok(None);
                } else {
                    return Ok(Some(entry));
                }
            }

            let trimmed = line_buf.trim_end_matches(['\r', '\n']);

            if trimmed.is_empty() {
                // Empty line is record separator
                if !entry.fields.is_empty() {
                    return Ok(Some(entry));
                }
                // Skip leading blank lines
                continue;
            }

            if let Some((key, val)) = trimmed.split_once('=') {
                // Text field: KEY=VALUE
                entry.insert(key.to_string(), val.as_bytes().to_vec());
            } else {
                // Binary field: KEY followed by 8-byte LE size and payload
                let field_name = trimmed.to_string();
                if field_name.is_empty()
                    || !field_name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                {
                    let _ = resync_journal_stream(&mut self.reader);
                    return Err(JournalError::InvalidFormat(field_name));
                }

                match read_binary_field(&mut self.reader, field_name, self.max_field_size) {
                    Ok((k, v)) => entry.insert(k, v),
                    Err(e) => {
                        let _ = resync_journal_stream(&mut self.reader);
                        return Err(e);
                    }
                }
            }
        }
    }
}
