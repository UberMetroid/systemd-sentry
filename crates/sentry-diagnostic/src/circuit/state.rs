//! State machine definitions for LLM provider circuit breaking.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Operational state of the provider circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderCircuitState {
    /// Normal operation: requests proceed with adaptive timeout.
    Closed,
    /// Breaker tripped: requests short-circuit immediately to deterministic fallback.
    Open,
    /// Trial probe mode: one probe request proceeds while others short-circuit.
    HalfOpen,
}

impl ProviderCircuitState {
    /// Returns true if circuit is in Closed state.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Returns true if circuit is in Open state.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Returns true if circuit is in HalfOpen state.
    pub fn is_half_open(&self) -> bool {
        matches!(self, Self::HalfOpen)
    }
}

/// Action instructed by the circuit breaker prior to initiating an inference request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderAction {
    /// Proceed with remote inference bounded by the given adaptive timeout.
    Proceed {
        /// Dynamically calculated inference timeout.
        timeout: Duration,
    },
    /// Short-circuit remote inference immediately; execute deterministic fallback.
    ShortCircuit,
}

impl ProviderAction {
    /// Returns true if the action is to proceed with inference.
    pub fn is_proceed(&self) -> bool {
        matches!(self, Self::Proceed { .. })
    }

    /// Returns true if the action is to short-circuit.
    pub fn is_short_circuit(&self) -> bool {
        matches!(self, Self::ShortCircuit)
    }
}
