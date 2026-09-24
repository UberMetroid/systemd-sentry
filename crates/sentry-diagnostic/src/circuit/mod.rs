//! Adaptive timeout calculation and provider circuit breaking.

pub mod breaker;
pub mod config;
pub mod latency_tracker;
pub mod state;

pub use breaker::ProviderBreaker;
pub use config::AdaptiveTimeoutConfig;
pub use latency_tracker::LatencyTracker;
pub use state::{ProviderAction, ProviderCircuitState};
