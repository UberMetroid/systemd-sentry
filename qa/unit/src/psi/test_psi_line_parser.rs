//! 1:1 Unit QA tests for PSI line parser.

use sentry_driver::psi::parse_psi_line;

#[test]
fn test_parse_psi_line_some() {
    let line = "some avg10=2.18 avg60=3.17 avg300=2.32 total=198966813";
    let (kind, psi_line) = parse_psi_line(line).expect("Must parse valid some line");
    assert_eq!(kind, "some");
    assert_eq!(psi_line.avg10, 2.18);
    assert_eq!(psi_line.avg60, 3.17);
    assert_eq!(psi_line.avg300, 2.32);
    assert_eq!(psi_line.total_usec, 198966813);
}

#[test]
fn test_parse_psi_line_full() {
    let line = "full avg10=0.00 avg60=0.00 avg300=0.00 total=0";
    let (kind, psi_line) = parse_psi_line(line).expect("Must parse valid full line");
    assert_eq!(kind, "full");
    assert_eq!(psi_line.avg10, 0.0);
    assert_eq!(psi_line.total_usec, 0);
}

#[test]
fn test_parse_psi_line_invalid_prefix() {
    let line = "invalid avg10=0.00 avg60=0.00 avg300=0.00 total=0";
    assert!(parse_psi_line(line).is_err());
}

#[test]
fn test_parse_psi_line_missing_field() {
    let line = "some avg10=1.00 avg60=2.00 total=1000";
    assert!(parse_psi_line(line).is_err());
}
