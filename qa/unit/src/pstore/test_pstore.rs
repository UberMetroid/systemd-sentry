//! Tests for systemd-pstore panic log extraction and directory scanning.

use sentry_driver::pstore::{extract_panic_report, scan_directory, MAX_PSTORE_REPORTS};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_extract_panic_report_valid_kernel_panic() {
    let content = r#"
[ 42.123456] Something went wrong in kernel space
[ 42.123457] Kernel panic - not syncing: Fatal exception in interrupt
[ 42.123458] CPU: 0 PID: 1234 Comm: bad_driver Not tainted 6.8.0
[ 42.123459] Hardware name: QEMU Standard PC
[ 42.123460] Call Trace:
[ 42.123461]  <TASK>
[ 42.123462]  dump_stack_lvl+0x48/0x70
[ 42.123463]  panic+0x310/0x350
[ 42.123464]  ? bad_page_fault+0x10/0x20
[ 42.123465]  [<ffffffff81001234>] do_page_fault+0x20/0x30
"#;

    let report = extract_panic_report("/sys/fs/pstore/dmesg-efi-1", content)
        .expect("Expected panic report to be parsed");

    assert_eq!(report.source_path, "/sys/fs/pstore/dmesg-efi-1");
    assert!(report.summary.contains("Kernel panic - not syncing"));
    assert!(!report.backtrace_snippet.is_empty());
}

#[test]
fn test_extract_panic_report_no_panic_returns_none() {
    let content = "[ 12.345] Normal boot message\n[ 12.346] systemd[1]: Reached target Basic System.\n";
    let report = extract_panic_report("/sys/fs/pstore/dmesg-efi-2", content);
    assert!(report.is_none());
}

#[test]
fn test_pstore_scanner_enforces_capacity_bound() {
    let temp = tempdir().expect("tempdir");
    let base = temp.path();

    // Create 40 pstore files (exceeding MAX_PSTORE_REPORTS = 32)
    for i in 0..40 {
        let file_path = base.join(format!("dmesg-test-{i}"));
        let mut f = File::create(&file_path).expect("create file");
        writeln!(f, "[ 1.0 ] Kernel panic - not syncing: Test panic {i}").expect("write panic");
    }

    let mut reports = Vec::new();
    scan_directory(base, &mut reports);

    assert_eq!(reports.len(), MAX_PSTORE_REPORTS);
}
