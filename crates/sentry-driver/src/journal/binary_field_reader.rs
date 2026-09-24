//! Reads 64-bit LE length-prefixed binary fields from Journal Export stream.

use sentry_core::error::JournalError;
use std::io::{self, BufRead};

/// Maximum permissible size for a single journal binary field (16 MiB).
pub const MAX_FIELD_SIZE: usize = 16 * 1024 * 1024;

/// Reads a 64-bit little-endian length-prefixed binary field from the buffer.
///
/// Ensures size <= `max_size` and confirms that the payload is followed by `\n`.
pub fn read_binary_field<R: BufRead>(
    reader: &mut R,
    field_name: String,
    max_size: usize,
) -> Result<(String, Vec<u8>), JournalError> {
    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            JournalError::UnexpectedEof
        } else {
            JournalError::Io(e)
        }
    })?;

    let length = u64::from_le_bytes(len_buf) as usize;
    if length > max_size {
        return Err(JournalError::FieldTooLarge {
            field: field_name,
            size: length,
            max: max_size,
        });
    }

    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            JournalError::UnexpectedEof
        } else {
            JournalError::Io(e)
        }
    })?;

    let mut trailing = [0u8; 1];
    reader.read_exact(&mut trailing).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            JournalError::UnexpectedEof
        } else {
            JournalError::Io(e)
        }
    })?;

    if trailing[0] != b'\n' {
        return Err(JournalError::InvalidBinaryDelimiter {
            field: field_name,
            expected: b'\n',
            found: trailing[0],
        });
    }

    Ok((field_name, payload))
}
