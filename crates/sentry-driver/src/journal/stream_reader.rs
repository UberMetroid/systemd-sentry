//! Asynchronous background ingestion reader for journal export streams.

use super::export_entry::JournalExportEntry;
use super::export_parser::JournalExportParser;
use std::io::{BufReader, Cursor, Read};
use tokio::sync::mpsc;
use tracing::{debug, error};

/// Spawns a background worker thread reading journal export entries from a synchronous reader.
pub struct JournalStreamReader;

impl JournalStreamReader {
    /// Spawns a background task reading from an arbitrary reader and emitting entries via channel.
    pub fn spawn_reader_stream<R: Read + Send + 'static>(
        reader: R,
        channel_capacity: usize,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Receiver<JournalExportEntry>) {
        let (tx, rx) = mpsc::channel(channel_capacity);

        let handle = tokio::task::spawn_blocking(move || {
            let mut parser = JournalExportParser::new(BufReader::new(reader));
            loop {
                match parser.parse_next_entry() {
                    Ok(Some(entry)) => {
                        if tx.blocking_send(entry).is_err() {
                            debug!("Receiver dropped; stopping journal stream");
                            break;
                        }
                    }
                    Ok(None) => {
                        debug!("Reached EOF on journal stream");
                        break;
                    }
                    Err(e) => {
                        error!("Journal export stream error (resynced): {e}");
                    }
                }
            }
        });

        (handle, rx)
    }

    /// Convenience helper: streams entries from an in-memory byte buffer (useful in tests).
    pub fn spawn_buffer_stream(
        buffer: Vec<u8>,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Receiver<JournalExportEntry>) {
        Self::spawn_reader_stream(Cursor::new(buffer), 64)
    }
}
