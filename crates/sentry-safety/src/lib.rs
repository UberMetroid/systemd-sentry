//! Zero-trust safety engine, sliding-window circuit breaker, and drop-in policy gatekeeper.
//!
//! Provides deterministic safety guarantees:
//! - Sliding-window failure tracking and flap detection lockout.
//! - Bounded in-memory registry with zero heap exhaustion risk (`MAX_TRACKED_UNITS = 512`).
//! - Drop-in policy overlays (`/etc/systemd-sentry/policy.d/*.toml`) with 64 KiB size limit.
//! - Zero-trust validation preventing LLM advisories from executing unauthorized actions or mutating protected units.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod circuit;
pub mod policy;

pub use circuit::{
    CircuitBreakerRegistry, CircuitConfig, CircuitState, CircuitStateSnapshot, UnitBreaker,
    DEFAULT_IDLE_TTL, MAX_TRACKED_UNITS,
};
pub use policy::{
    load_policy_with_dropins, load_policy_file, PolicyConfig, PolicyGatekeeper,
    UnitPolicyOverride, DEFAULT_PROTECTED_UNITS, MAX_POLICY_FILE_SIZE,
};
