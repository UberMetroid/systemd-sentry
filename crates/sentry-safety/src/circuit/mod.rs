//! Sliding-window circuit breaker and flap detection subsystem.

pub mod config;
pub mod registry;
pub mod state;
pub mod unit_breaker;

pub use config::CircuitConfig;
pub use registry::{CircuitBreakerRegistry, DEFAULT_IDLE_TTL, MAX_TRACKED_UNITS};
pub use state::{CircuitState, CircuitStateSnapshot};
pub use unit_breaker::UnitBreaker;
