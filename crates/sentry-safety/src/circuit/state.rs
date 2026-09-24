//! State representation for the sliding-window circuit breaker.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Operational state of a service unit circuit breaker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation; unit failures are within safe thresholds.
    Closed,
    /// Tripped state; automatic actions are blocked until cooldown expires.
    Open {
        /// Monotonic timestamp when the circuit tripped open.
        tripped_at: Instant,
        /// Duration required before transitioning to HalfOpen.
        cooldown: Duration,
        /// Failure count that triggered the trip.
        failure_count: usize,
    },
    /// Trial state; single remediation attempt allowed to verify recovery.
    HalfOpen,
    /// Permanently locked due to severe flapping; requires manual operator reset.
    PermanentlyLocked {
        /// Monotonic timestamp when permanent lockout was engaged.
        locked_at: Instant,
        /// Total trip count recorded in the flap window.
        flap_trips: usize,
    },
}

impl CircuitState {
    /// Returns true if the circuit allows remediation attempts.
    pub fn allows_remediation(&self) -> bool {
        match self {
            Self::Closed | Self::HalfOpen => true,
            Self::Open { .. } | Self::PermanentlyLocked { .. } => false,
        }
    }

    /// Returns a human-readable state label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Closed => "CLOSED",
            Self::Open { .. } => "OPEN",
            Self::HalfOpen => "HALF_OPEN",
            Self::PermanentlyLocked { .. } => "PERMANENTLY_LOCKED",
        }
    }
}

/// JSON/telemetry serializable snapshot of circuit state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitStateSnapshot {
    /// State label (CLOSED, OPEN, HALF_OPEN, PERMANENTLY_LOCKED).
    pub state: String,
    /// Remaining cooldown in seconds if OPEN, 0 otherwise.
    pub cooldown_remaining_secs: u64,
    /// Total recent failure count recorded in window.
    pub recent_failures: usize,
    /// Whether the unit is permanently locked due to flapping.
    pub permanently_locked: bool,
}
