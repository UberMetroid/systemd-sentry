//! Deterministic rule-based fallback triage system.

pub mod deterministic_engine;
pub mod exit_codes;
pub mod journal_patterns;

pub use deterministic_engine::DeterministicFallbackEngine;
pub use exit_codes::{triage_exit_code, ExitCodeTriageResult};
pub use journal_patterns::{scan_journal_patterns, JournalMatchResult};
