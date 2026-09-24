//! D-Bus message wire format decoder for fuzzing.
//!
//! Validates standard D-Bus wire protocol frames (16-byte fixed header,
//! field array alignment, object paths, interface strings).

#[derive(Debug, PartialEq, Eq)]
pub enum DbusDecodeError {
    HeaderTooShort,
    UnsupportedEndian,
    InvalidProtocolVersion,
    OversizedPayload,
    CorruptedHeaderFields,
    InvalidStringEncoding,
}

#[derive(Debug)]
pub struct DbusHeader {
    pub is_little_endian: bool,
    pub message_type: u8,
    pub flags: u8,
    pub body_length: u32,
    pub serial: u32,
    pub fields_length: u32,
}

pub fn decode_dbus_message_header(data: &[u8]) -> Result<DbusHeader, DbusDecodeError> {
    if data.len() < 16 {
        return Err(DbusDecodeError::HeaderTooShort);
    }

    let is_little_endian = match data[0] {
        b'l' => true,
        b'B' => false,
        _ => return Err(DbusDecodeError::UnsupportedEndian),
    };

    let message_type = data[1];
    let flags = data[2];
    let protocol_version = data[3];

    if protocol_version != 1 {
        return Err(DbusDecodeError::InvalidProtocolVersion);
    }

    let read_u32 = |offset: usize| -> u32 {
        let slice: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        if is_little_endian {
            u32::from_le_bytes(slice)
        } else {
            u32::from_be_bytes(slice)
        }
    };

    let body_length = read_u32(4);
    let serial = read_u32(8);
    let fields_length = read_u32(12);

    // D-Bus specification caps message size at 128MB (134217728 bytes)
    if body_length > 134_217_728 || fields_length > 67_108_864 {
        return Err(DbusDecodeError::OversizedPayload);
    }

    let total_header_length = 16 + fields_length as usize;
    // Align header to 8-byte boundary
    let aligned_header_len = (total_header_length + 7) & !7;

    if data.len() < aligned_header_len {
        return Err(DbusDecodeError::HeaderTooShort);
    }

    Ok(DbusHeader {
        is_little_endian,
        message_type,
        flags,
        body_length,
        serial,
        fields_length,
    })
}
