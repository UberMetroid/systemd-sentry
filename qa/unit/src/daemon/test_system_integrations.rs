//! Tests verifying domain models and diagnostic patterns for new systemd integrations.

use sentry_core::models::{
    DnsHealthState, NetworkOperationalState, PstorePanicReport, ShutdownState, TimeSyncStatus,
};
use sentry_diagnostic::fallback::journal_patterns::scan_journal_patterns;

#[test]
fn test_system_state_domain_models() {
    let normal = ShutdownState::Normal;
    assert!(!normal.is_transitioning());

    let shutdown = ShutdownState::PreparingForShutdown;
    assert!(shutdown.is_transitioning());

    let sleep = ShutdownState::PreparingForSleep;
    assert!(sleep.is_transitioning());

    assert_eq!(DnsHealthState::default(), DnsHealthState::Operational);
    assert_eq!(TimeSyncStatus::default(), TimeSyncStatus::Synchronized);

    let net_routable = NetworkOperationalState::Routable;
    assert!(net_routable.is_routable());
    assert!(!NetworkOperationalState::NoCarrier.is_routable());

    let pstore = PstorePanicReport {
        source_path: "/sys/fs/pstore/dmesg-1".to_string(),
        summary: "Kernel panic".to_string(),
        backtrace_snippet: vec!["[<0000>] panic".to_string()],
    };
    assert_eq!(pstore.source_path, "/sys/fs/pstore/dmesg-1");
}

#[test]
fn test_diagnostic_patterns_for_subsystems() {
    // DNS pattern
    let dns_lines = vec!["Worker thread failed: getaddrinfo failed: EAI_AGAIN".to_string()];
    let dns_res = scan_journal_patterns(&dns_lines).expect("DNS match");
    assert!(dns_res.root_cause.summary.contains("systemd-resolved"));

    // Network link down pattern
    let net_lines = vec!["FATAL: connect() failed: Network is unreachable".to_string()];
    let net_res = scan_journal_patterns(&net_lines).expect("Network match");
    assert!(net_res.root_cause.summary.contains("systemd-networkd"));

    // Timesync clock drift pattern
    let time_lines = vec!["OpenSSL verify error: certificate has expired or is not yet valid".to_string()];
    let time_res = scan_journal_patterns(&time_lines).expect("Timesync match");
    assert!(time_res.root_cause.summary.contains("systemd-timesyncd"));

    // Kernel panic pattern
    let panic_lines = vec!["[ 10.0 ] Kernel panic - not syncing: VFS: Unable to mount root fs".to_string()];
    let panic_res = scan_journal_patterns(&panic_lines).expect("Panic match");
    assert!(panic_res.root_cause.summary.contains("systemd-pstore"));
}
