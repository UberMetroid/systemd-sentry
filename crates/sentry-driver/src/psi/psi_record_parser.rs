//! Parses multi-line PSI files into unified `PsiRecord` models.

use super::psi_line_parser::parse_psi_line;
use sentry_core::error::PsiError;
use sentry_core::models::{PsiLine, PsiRecord};

/// Parses multi-line PSI content containing a required `some` line and optional `full` line.
pub fn parse_psi_record(content: &str) -> Result<PsiRecord, PsiError> {
    let mut some_line: Option<PsiLine> = None;
    let mut full_line: Option<PsiLine> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (kind, parsed) = parse_psi_line(trimmed)?;
        match kind {
            "some" => some_line = Some(parsed),
            "full" => full_line = Some(parsed),
            _ => {}
        }
    }

    let some = some_line.ok_or(PsiError::MissingSomeLine)?;
    Ok(PsiRecord {
        some,
        full: full_line,
    })
}
