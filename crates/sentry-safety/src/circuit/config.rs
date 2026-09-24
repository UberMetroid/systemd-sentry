//! Configuration settings for sliding-window circuit breaking and flap detection.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration governing sliding-window circuit breaker thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitConfig {
    /// Maximum allowable failures within `window_duration` before tripping OPEN.
    pub max_failures: usize,
    /// Rolling time window for failure tracking.
    pub window_duration: Duration,
    /// Cooldown duration during OPEN state before transitioning to HalfOpen.
    pub cooldown_duration: Duration,
    /// Time window for monitoring trip frequency to detect flapping.
    pub flap_window: Duration,
    /// Number of breaker trips within `flap_window` that permanently locks the unit.
    pub flap_threshold: usize,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            max_failures: 3,
            window_duration: Duration::from_secs(300),   // 5 minutes
            cooldown_duration: Duration::from_secs(600), // 10 minutes
            flap_window: Duration::from_secs(900),       // 15 minutes
            flap_threshold: 3,                           // 3 trips in 15 mins -> Lockout
        }
    }
}

impl CircuitConfig {
    /// Creates a custom circuit configuration.
    pub fn new(
        max_failures: usize,
        window_duration: Duration,
        cooldown_duration: Duration,
        flap_window: Duration,
        flap_threshold: usize,
    ) -> Self {
        Self {
            max_failures: max_failures.max(1),
            window_duration,
            cooldown_duration,
            flap_window,
            flap_threshold: flap_threshold.max(1),
        }
    }
}
