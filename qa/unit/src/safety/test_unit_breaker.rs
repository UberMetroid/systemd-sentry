//! Unit tests for individual UnitBreaker logic.

use sentry_safety::circuit::{CircuitConfig, CircuitState, UnitBreaker};
use std::time::{Duration, Instant};

#[test]
fn test_unit_breaker_initial_state() {
    let now = Instant::now();
    let breaker = UnitBreaker::new("web.service", now);
    assert_eq!(breaker.unit(), "web.service");
    let snap = breaker.snapshot(now);
    assert_eq!(snap.state, "CLOSED");
    assert_eq!(snap.recent_failures, 0);
    assert!(!snap.permanently_locked);
}

#[test]
fn test_unit_breaker_trip_and_cooldown_to_half_open() {
    let mut now = Instant::now();
    let config = CircuitConfig {
        max_failures: 2,
        window_duration: Duration::from_secs(60),
        cooldown_duration: Duration::from_secs(10),
        flap_window: Duration::from_secs(300),
        flap_threshold: 3,
    };

    let mut breaker = UnitBreaker::new("api.service", now);

    // 1st failure: still CLOSED
    let st = breaker.record_failure(now, &config);
    assert_eq!(*st, CircuitState::Closed);

    // 2nd failure: trips OPEN
    let st = breaker.record_failure(now, &config);
    assert!(matches!(st, CircuitState::Open { failure_count: 2, .. }));

    // Before cooldown elapses: still OPEN
    now += Duration::from_secs(5);
    let st = breaker.evaluate_state(now);
    assert!(matches!(st, CircuitState::Open { .. }));

    // After cooldown: transitions to HALF_OPEN
    now += Duration::from_secs(6);
    let st = breaker.evaluate_state(now);
    assert_eq!(*st, CircuitState::HalfOpen);

    // Success in HalfOpen: returns to CLOSED
    breaker.record_success(now);
    let snap = breaker.snapshot(now);
    assert_eq!(snap.state, "CLOSED");
    assert_eq!(snap.recent_failures, 0);
}

#[test]
fn test_unit_breaker_flapping_lockout_and_operator_reset() {
    let mut now = Instant::now();
    let config = CircuitConfig {
        max_failures: 1,
        window_duration: Duration::from_secs(60),
        cooldown_duration: Duration::from_secs(5),
        flap_window: Duration::from_secs(60),
        flap_threshold: 3,
    };

    let mut breaker = UnitBreaker::new("flapper.service", now);

    // Trip 1
    breaker.record_failure(now, &config);
    now += Duration::from_secs(6);
    breaker.evaluate_state(now); // HalfOpen

    // Trip 2
    breaker.record_failure(now, &config);
    now += Duration::from_secs(6);
    breaker.evaluate_state(now); // HalfOpen

    // Trip 3 -> PermanentlyLocked!
    let st = breaker.record_failure(now, &config);
    assert!(matches!(st, CircuitState::PermanentlyLocked { flap_trips: 3, .. }));

    // Additional failures do not panic and remain locked
    let st = breaker.record_failure(now, &config);
    assert!(matches!(st, CircuitState::PermanentlyLocked { .. }));

    // Operator reset restores CLOSED
    breaker.reset(now);
    let snap = breaker.snapshot(now);
    assert_eq!(snap.state, "CLOSED");
    assert!(!snap.permanently_locked);
}
