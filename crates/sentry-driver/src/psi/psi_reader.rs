//! Filesystem reader for Pressure Stall Information files.

use super::psi_record_parser::parse_psi_record;
use sentry_core::error::PsiError;
use sentry_core::models::PsiRecord;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Reads and parses a PSI file from the specified path into a fixed stack buffer.
pub fn read_psi_file(path: &Path) -> Result<PsiRecord, PsiError> {
    let mut file = File::open(path).map_err(PsiError::Io)?;
    let mut buf = [0u8; 512];
    let mut total = 0;
    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(PsiError::Io(e)),
        }
    }
    let content = std::str::from_utf8(&buf[..total])
        .map_err(|e| PsiError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    parse_psi_record(content)
}
