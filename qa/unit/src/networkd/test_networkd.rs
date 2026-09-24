//! Tests for systemd-networkd client and carrier status evaluation.

use sentry_core::models::NetworkOperationalState;
use sentry_driver::networkd::NetworkdClient;

#[test]
fn test_network_operational_state_parsing() {
    assert_eq!(NetworkOperationalState::from_dbus_str("routable"), NetworkOperationalState::Routable);
    assert!(NetworkOperationalState::Routable.is_routable());

    assert_eq!(NetworkOperationalState::from_dbus_str("degraded"), NetworkOperationalState::Degraded);
    assert!(!NetworkOperationalState::Degraded.is_routable());

    assert_eq!(NetworkOperationalState::from_dbus_str("carrier"), NetworkOperationalState::Carrier);
    assert_eq!(NetworkOperationalState::from_dbus_str("no-carrier"), NetworkOperationalState::NoCarrier);
    assert_eq!(NetworkOperationalState::from_dbus_str("off"), NetworkOperationalState::Off);
    assert_eq!(NetworkOperationalState::from_dbus_str("unexpected-state"), NetworkOperationalState::Unknown);
}

#[test]
fn test_network_down_error_detection() {
    assert!(NetworkdClient::is_network_down_error("connect failed: Network is unreachable"));
    assert!(NetworkdClient::is_network_down_error("fatal: No route to host"));
    assert!(NetworkdClient::is_network_down_error("interface eth0: Carrier lost"));
    assert!(NetworkdClient::is_network_down_error("systemd-networkd: link down"));
    assert!(NetworkdClient::is_network_down_error("errno 113: EHOSTUNREACH"));

    assert!(!NetworkdClient::is_network_down_error("Connection refused"));
    assert!(!NetworkdClient::is_network_down_error("Permission denied"));
}
