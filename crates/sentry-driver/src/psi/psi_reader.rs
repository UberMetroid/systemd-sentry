//! Filesystem reader for Pressure Stall Information files.

use super::psi_record_parser::parse_psi_record;
use sentry_core::error::PsiError;
use sentry_core::models::PsiRecord;
use std::fs;
use std::path::Path;

/// Reads and parses a PSI file from the specified path.
pub fn read_psi_file(path: &Path) -> Result<PsiRecord, PsiError> {
    let content = fs::read_to_string(path).map_err(PsiError::Io)?;
    parse_psi_record(&content)
}
