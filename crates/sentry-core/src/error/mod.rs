//! Granular error types using `thiserror`.

pub mod circuit_error;
pub mod config_error;
pub mod coredump_error;
pub mod diagnostic_error;
pub mod driver_error;
pub mod journal_error;
pub mod psi_error;
pub mod safety_error;
pub mod sentry_error;
pub mod telemetry_error;

pub use circuit_error::CircuitError;
pub use config_error::ConfigError;
pub use coredump_error::CoredumpError;
pub use diagnostic_error::DiagnosticError;
pub use driver_error::DriverError;
pub use journal_error::JournalError;
pub use psi_error::PsiError;
pub use safety_error::SafetyError;
pub use sentry_error::SentryError;
pub use telemetry_error::TelemetryError;
