//! Circuit breaker governing remote LLM provider availability and fallback failover.

use crate::circuit::config::AdaptiveTimeoutConfig;
use crate::circuit::latency_tracker::LatencyTracker;
use crate::circuit::state::{ProviderAction, ProviderCircuitState};
use std::time::{Duration, Instant};

/// Circuit breaker tracking provider failures, latency tracking, and probe throttling.
#[derive(Debug, Clone)]
pub struct ProviderBreaker {
    state: ProviderCircuitState,
    config: AdaptiveTimeoutConfig,
    tracker: LatencyTracker,
    consecutive_failures: usize,
    tripped_at: Option<Instant>,
    current_cooldown: Duration,
    probe_in_flight: bool,
}

impl ProviderBreaker {
    /// Constructs a new provider circuit breaker with the given configuration.
    pub fn new(config: AdaptiveTimeoutConfig) -> Self {
        let base_cooldown = config.base_cooldown;
        Self {
            state: ProviderCircuitState::Closed,
            current_cooldown: base_cooldown,
            config,
            tracker: LatencyTracker::new(),
            consecutive_failures: 0,
            tripped_at: None,
            probe_in_flight: false,
        }
    }

    /// Evaluates circuit state and determines whether to proceed or short-circuit.
    pub fn before_request(&mut self) -> ProviderAction {
        match self.state {
            ProviderCircuitState::Closed => {
                let timeout = self.tracker.calculate_timeout(&self.config);
                ProviderAction::Proceed { timeout }
            }
            ProviderCircuitState::Open => {
                let is_cooldown_elapsed = match self.tripped_at {
                    Some(t) => t.elapsed() >= self.current_cooldown,
                    None => true,
                };

                if is_cooldown_elapsed {
                    self.state = ProviderCircuitState::HalfOpen;
                    self.probe_in_flight = true;
                    let timeout = self.tracker.calculate_timeout(&self.config);
                    ProviderAction::Proceed { timeout }
                } else {
                    ProviderAction::ShortCircuit
                }
            }
            ProviderCircuitState::HalfOpen => {
                if self.probe_in_flight {
                    // Prevent stampede / thundering herd while a probe is active
                    ProviderAction::ShortCircuit
                } else {
                    self.probe_in_flight = true;
                    let timeout = self.tracker.calculate_timeout(&self.config);
                    ProviderAction::Proceed { timeout }
                }
            }
        }
    }

    /// Notifies breaker of successful response, recording latency and restoring Closed state.
    pub fn on_success(&mut self, elapsed: Duration) {
        self.tracker.record(elapsed, self.config.ema_alpha);
        self.consecutive_failures = 0;
        self.current_cooldown = self.config.base_cooldown;
        self.state = ProviderCircuitState::Closed;
        self.probe_in_flight = false;
        self.tripped_at = None;
    }

    /// Notifies breaker of an inference request timeout.
    pub fn on_timeout(&mut self) {
        self.handle_failure();
    }

    /// Notifies breaker of an inference error (e.g. connection refused, 5xx).
    pub fn on_error(&mut self) {
        self.handle_failure();
    }

    fn handle_failure(&mut self) {
        let prev_state = self.state;
        self.consecutive_failures += 1;

        if prev_state == ProviderCircuitState::HalfOpen {
            // Re-trip to Open with exponential backoff on cooldown
            self.current_cooldown = self
                .current_cooldown
                .saturating_mul(2)
                .min(self.config.max_cooldown);
            self.state = ProviderCircuitState::Open;
            self.tripped_at = Some(Instant::now());
            self.probe_in_flight = false;
        } else if self.consecutive_failures >= self.config.failure_threshold {
            // Initial trip from Closed to Open
            if prev_state != ProviderCircuitState::Open {
                self.current_cooldown = self.config.base_cooldown;
            }
            self.state = ProviderCircuitState::Open;
            self.tripped_at = Some(Instant::now());
            self.probe_in_flight = false;
        }
    }

    /// Returns the current circuit state.
    pub fn state(&self) -> ProviderCircuitState {
        self.state
    }

    /// Returns the number of consecutive failures.
    pub fn consecutive_failures(&self) -> usize {
        self.consecutive_failures
    }

    /// Returns the current active cooldown duration.
    pub fn current_cooldown(&self) -> Duration {
        self.current_cooldown
    }

    /// Returns the instant the circuit was tripped, if open.
    pub fn tripped_at(&self) -> Option<Instant> {
        self.tripped_at
    }

    /// Returns whether a probe request is currently in flight.
    pub fn is_probe_in_flight(&self) -> bool {
        self.probe_in_flight
    }

    /// Returns a reference to the underlying latency tracker.
    pub fn tracker(&self) -> &LatencyTracker {
        &self.tracker
    }

    /// Returns a reference to the active configuration.
    pub fn config(&self) -> &AdaptiveTimeoutConfig {
        &self.config
    }

    /// Sets the tripped timestamp (used for deterministic unit tests).
    pub fn set_tripped_at(&mut self, instant: Option<Instant>) {
        self.tripped_at = instant;
    }
}
