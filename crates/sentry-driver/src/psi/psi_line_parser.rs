//! Parses individual PSI lines (`some` or `full`).

use sentry_core::error::PsiError;
use sentry_core::models::PsiLine;

/// Parses a single line of PSI output (e.g. `some avg10=0.00 avg60=0.00 avg300=0.00 total=0`).
pub fn parse_psi_line(line: &str) -> Result<(&str, PsiLine), PsiError> {
    let trimmed = line.trim();
    let (kind, rest) = trimmed
        .split_once(char::is_whitespace)
        .ok_or_else(|| PsiError::InvalidLineFormat(trimmed.to_string()))?;

    if kind != "some" && kind != "full" {
        return Err(PsiError::UnknownLinePrefix(kind.to_string()));
    }

    let mut avg10 = None;
    let mut avg60 = None;
    let mut avg300 = None;
    let mut total = None;

    for token in rest.split_whitespace() {
        if let Some((k, v)) = token.split_once('=') {
            match k {
                "avg10" => avg10 = v.parse::<f64>().ok(),
                "avg60" => avg60 = v.parse::<f64>().ok(),
                "avg300" => avg300 = v.parse::<f64>().ok(),
                "total" => total = v.parse::<u64>().ok(),
                _ => {}
            }
        }
    }

    let parsed_line = PsiLine {
        avg10: avg10.ok_or_else(|| PsiError::MissingField("avg10", trimmed.to_string()))?,
        avg60: avg60.ok_or_else(|| PsiError::MissingField("avg60", trimmed.to_string()))?,
        avg300: avg300.ok_or_else(|| PsiError::MissingField("avg300", trimmed.to_string()))?,
        total_usec: total.ok_or_else(|| PsiError::MissingField("total", trimmed.to_string()))?,
    };

    Ok((kind, parsed_line))
}
