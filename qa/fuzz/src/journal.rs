//! Journal Export Format parser for fuzzing and stress testing.
//!
//! Complies with systemd Journal Export Format:
//! - ASCII key=value\n
//! - Binary key\n<8 bytes uint64_t le length><payload>\n
//! - \n delimiter between entries

use std::collections::HashMap;

pub const MAX_FIELD_SIZE: usize = 4 * 1024 * 1024; // 4MB DoS limit
pub const MAX_KEYS: usize = 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum JournalParseError {
    CorruptedStream,
    OversizedField,
    TruncatedPayload,
    InvalidKeyName,
    TooManyKeys,
}

pub struct JournalEntry {
    pub fields: HashMap<String, Vec<u8>>,
}

pub fn parse_journal_export_entry(data: &[u8]) -> Result<Option<(JournalEntry, usize)>, JournalParseError> {
    if data.is_empty() {
        return Ok(None);
    }

    let mut fields = HashMap::new();
    let mut cursor = 0;

    while cursor < data.len() {
        // Double newline indicates end of entry
        if data[cursor] == b'\n' {
            cursor += 1;
            return Ok(Some((JournalEntry { fields }, cursor)));
        }

        // Find newline
        let line_end = match data[cursor..].iter().position(|&b| b == b'\n') {
            Some(pos) => cursor + pos,
            None => return Err(JournalParseError::TruncatedPayload),
        };

        let line = &data[cursor..line_end];
        cursor = line_end + 1;

        if let Some(eq_pos) = line.iter().position(|&b| b == b'=') {
            // Text field: KEY=VALUE
            let key_bytes = &line[..eq_pos];
            if key_bytes.is_empty() || !key_bytes.iter().all(|&b| b.is_ascii_uppercase() || b == b'_') {
                return Err(JournalParseError::InvalidKeyName);
            }
            let key = String::from_utf8_lossy(key_bytes).to_string();
            let val = line[eq_pos + 1..].to_vec();

            if fields.len() >= MAX_KEYS {
                return Err(JournalParseError::TooManyKeys);
            }
            fields.insert(key, val);
        } else {
            // Binary field: KEY\n<8-byte length><payload>\n
            if line.is_empty() || !line.iter().all(|&b| b.is_ascii_uppercase() || b == b'_') {
                return Err(JournalParseError::InvalidKeyName);
            }
            let key = String::from_utf8_lossy(line).to_string();

            if cursor + 8 > data.len() {
                return Err(JournalParseError::TruncatedPayload);
            }

            let len_bytes: [u8; 8] = match data[cursor..cursor + 8].try_into() {
                Ok(b) => b,
                Err(_) => return Err(JournalParseError::CorruptedStream),
            };
            cursor += 8;

            let length = u64::from_le_bytes(len_bytes) as usize;
            if length > MAX_FIELD_SIZE {
                return Err(JournalParseError::OversizedField);
            }

            if cursor + length > data.len() {
                return Err(JournalParseError::TruncatedPayload);
            }

            let val = data[cursor..cursor + length].to_vec();
            cursor += length;

            // Trailing newline after binary payload
            if cursor < data.len() && data[cursor] == b'\n' {
                cursor += 1;
            }

            if fields.len() >= MAX_KEYS {
                return Err(JournalParseError::TooManyKeys);
            }
            fields.insert(key, val);
        }
    }

    Ok(Some((JournalEntry { fields }, cursor)))
}
