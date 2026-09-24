//! Linux PSI (Pressure Stall Information) text format parser for fuzzing.
//!
//! Parses `/proc/pressure/{cpu,memory,io}` streams:
//! `some avg10=0.00 avg60=0.00 avg300=0.00 total=0`
//! `full avg10=0.00 avg60=0.00 avg300=0.00 total=0`

#[derive(Debug, PartialEq, Default, Clone)]
pub struct PsiMetrics {
    pub avg10: f64,
    pub avg60: f64,
    pub avg300: f64,
    pub total_us: u64,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct PsiRecord {
    pub some: Option<PsiMetrics>,
    pub full: Option<PsiMetrics>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PsiParseError {
    InvalidFormat,
    UnknownPrefix,
    MalformedToken,
    NumericOutOfRange,
}

fn parse_line(line: &str) -> Result<(&str, PsiMetrics), PsiParseError> {
    let mut parts = line.split_whitespace();
    let prefix = parts.next().ok_or(PsiParseError::InvalidFormat)?;

    let mut metrics = PsiMetrics::default();
    let mut count = 0;

    for token in parts {
        let (k, v) = token.split_once('=').ok_or(PsiParseError::MalformedToken)?;
        match k {
            "avg10" => {
                let val: f64 = v.parse().map_err(|_| PsiParseError::NumericOutOfRange)?;
                if !val.is_finite() || val < 0.0 || val > 100.0 {
                    return Err(PsiParseError::NumericOutOfRange);
                }
                metrics.avg10 = val;
                count += 1;
            }
            "avg60" => {
                let val: f64 = v.parse().map_err(|_| PsiParseError::NumericOutOfRange)?;
                if !val.is_finite() || val < 0.0 || val > 100.0 {
                    return Err(PsiParseError::NumericOutOfRange);
                }
                metrics.avg60 = val;
                count += 1;
            }
            "avg300" => {
                let val: f64 = v.parse().map_err(|_| PsiParseError::NumericOutOfRange)?;
                if !val.is_finite() || val < 0.0 || val > 100.0 {
                    return Err(PsiParseError::NumericOutOfRange);
                }
                metrics.avg300 = val;
                count += 1;
            }
            "total" => {
                let val: u64 = v.parse().map_err(|_| PsiParseError::NumericOutOfRange)?;
                metrics.total_us = val;
                count += 1;
            }
            _ => return Err(PsiParseError::MalformedToken),
        }
    }

    if count < 4 {
        return Err(PsiParseError::InvalidFormat);
    }

    Ok((prefix, metrics))
}

pub fn parse_psi_record(text: &str) -> Result<PsiRecord, PsiParseError> {
    let mut record = PsiRecord::default();
    let mut parsed_any = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (prefix, metrics) = parse_line(trimmed)?;
        match prefix {
            "some" => record.some = Some(metrics),
            "full" => record.full = Some(metrics),
            _ => return Err(PsiParseError::UnknownPrefix),
        }
        parsed_any = true;
    }

    if !parsed_any {
        return Err(PsiParseError::InvalidFormat);
    }

    Ok(record)
}
