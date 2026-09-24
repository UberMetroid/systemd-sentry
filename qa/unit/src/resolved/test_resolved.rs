//! Tests for systemd-resolved DNS client and error classification.

use sentry_driver::resolved::ResolvedClient;

#[test]
fn test_resolved_dns_error_detection() {
    assert!(ResolvedClient::is_dns_error("Fatal: EAI_AGAIN lookup failed"));
    assert!(ResolvedClient::is_dns_error("Error: name or service not known"));
    assert!(ResolvedClient::is_dns_error("getaddrinfo failed: Temporary failure in name resolution"));
    assert!(ResolvedClient::is_dns_error("Query returned NXDOMAIN"));
    assert!(ResolvedClient::is_dns_error("Server returned SERVFAIL response"));

    // False positives should return false
    assert!(!ResolvedClient::is_dns_error("Connection refused on port 8080"));
    assert!(!ResolvedClient::is_dns_error("Segmentation fault at 0x0"));
    assert!(!ResolvedClient::is_dns_error("Out of memory: Killed process"));
}
