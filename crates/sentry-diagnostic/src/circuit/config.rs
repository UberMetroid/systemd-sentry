//! Configuration for adaptive LLM inference timeouts and circuit breaking.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration governing dynamic inference timeout calculation and provider circuit breaking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveTimeoutConfig {
    /// Minimum allowable timeout floor to absorb network jitter (default 1.5s).
    pub min_timeout: Duration,
    /// Maximum allowable timeout ceiling preventing daemon stalls (default 15.0s).
    pub max_timeout: Duration,
    /// Initial timeout used when zero latency samples are recorded (default 5.0s).
    pub initial_timeout: Duration,
    /// Consecutive failure threshold before tripping circuit OPEN (default 3).
    pub failure_threshold: usize,
    /// Base cooldown duration before attempting a probe request in HalfOpen (default 30s).
    pub base_cooldown: Duration,
    /// Maximum backoff cooldown ceiling for repeated probe failures (default 300s).
    pub max_cooldown: Duration,
    /// Smoothing factor for exponential moving average latency (default 0.20).
    pub ema_alpha: f64,
}

impl Default for AdaptiveTimeoutConfig {
    fn default() -> Self {
        Self {
            min_timeout: Duration::from_millis(1500),
            max_timeout: Duration::from_secs(15),
            initial_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            base_cooldown: Duration::from_secs(30),
            max_cooldown: Duration::from_secs(300),
            ema_alpha: 0.20,
        }
    }
}

impl AdaptiveTimeoutConfig {
    /// Creates a new configuration with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum allowable timeout floor.
    pub fn with_min_timeout(mut self, timeout: Duration) -> Self {
        self.min_timeout = timeout;
        self
    }

    /// Sets the maximum allowable timeout ceiling.
    pub fn with_max_timeout(mut self, timeout: Duration) -> Self {
        self.max_timeout = timeout;
        self
    }

    /// Sets the initial timeout used when no latency samples exist.
    pub fn with_initial_timeout(mut self, timeout: Duration) -> Self {
        self.initial_timeout = timeout;
        self
    }

    /// Sets the consecutive failure threshold to trip the circuit.
    pub fn with_failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Sets the base cooldown duration.
    pub fn with_base_cooldown(mut self, cooldown: Duration) -> Self {
        self.base_cooldown = cooldown;
        self
    }

    /// Sets the maximum backoff cooldown ceiling.
    pub fn with_max_cooldown(mut self, max_cooldown: Duration) -> Self {
        self.max_cooldown = max_cooldown;
        self
    }

    /// Sets the EMA smoothing factor alpha (clamped between 0.01 and 1.0).
    pub fn with_ema_alpha(mut self, alpha: f64) -> Self {
        self.ema_alpha = alpha.clamp(0.01, 1.0);
        self
    }
}
