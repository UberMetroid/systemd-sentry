//! 1:1 Unit QA tests for PSI record parser.

use sentry_driver::psi::parse_psi_record;

#[test]
fn test_parse_psi_record_with_some_and_full() {
    let content = "\
some avg10=1.50 avg60=1.20 avg300=0.80 total=50000
full avg10=0.50 avg60=0.20 avg300=0.10 total=10000
";
    let record = parse_psi_record(content).expect("Must parse record");
    assert_eq!(record.some.avg10, 1.50);
    assert!(record.full.is_some());
    assert_eq!(record.full.as_ref().unwrap().avg10, 0.50);
}

#[test]
fn test_parse_psi_record_some_only_kernel_legacy() {
    let content = "some avg10=5.00 avg60=4.00 avg300=3.00 total=123456\n";
    let record = parse_psi_record(content).expect("Must parse single-line record");
    assert_eq!(record.some.avg10, 5.00);
    assert!(record.full.is_none());
}

#[test]
fn test_parse_psi_record_missing_some_fails() {
    let content = "full avg10=0.50 avg60=0.20 avg300=0.10 total=10000\n";
    assert!(parse_psi_record(content).is_err());
}
