//! In-memory state and synchronization primitives for the supervisor daemon.

use crate::config::daemon_config::DaemonConfig;
use crate::ipc::protocol::IpcResponse;
use crate::system::{LoadShedder, StatmReader};
use sentry_diagnostic::DiagnosticEngine;
use sentry_mcp::McpState;
use sentry_safety::circuit::CircuitBreakerRegistry;
use sentry_safety::policy::PolicyGatekeeper;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Shared runtime state of the supervisor daemon.
pub struct DaemonState {
    /// Monotonic startup timestamp for calculating uptime.
    pub start_time: Instant,
    /// Active daemon configuration.
    pub config: DaemonConfig,
    /// Bounded sliding-window circuit breaker registry.
    pub circuit_registry: CircuitBreakerRegistry,
    /// Bounded in-memory incident storage (ring buffer).
    pub mcp_state: Arc<McpState>,
    /// AI diagnostic triage engine.
    pub diagnostic_engine: DiagnosticEngine,
    /// Active zero-trust policy gatekeeper.
    pub policy_gatekeeper: PolicyGatekeeper,
    /// Atomic counter for telemetry events dropped under pressure.
    pub dropped_events: AtomicUsize,
    /// Rate-limited zero-allocation RSS memory reader.
    pub statm_reader: StatmReader,
    /// Hysteresis controller for load-shedding degraded mode.
    pub load_shedder: LoadShedder,
    /// Broadcast channel for real-time monitoring clients.
    pub event_broadcaster: broadcast::Sender<IpcResponse>,
}

impl DaemonState {
    /// Initialize daemon state from configuration and loaded policy.
    pub fn new(config: DaemonConfig, policy_gatekeeper: PolicyGatekeeper) -> Self {
        let (tx, _) = broadcast::channel(256);
        let mcp_state = Arc::new(McpState::new());
        let provider: Arc<dyn sentry_diagnostic::LlmProvider> = Arc::from(
            sentry_diagnostic::provider::create_provider(&config.provider)
        );
        let diagnostic_engine = DiagnosticEngine::new(Some(provider));
        let circuit_registry = sentry_safety::circuit::CircuitBreakerRegistry::new(
            sentry_safety::circuit::CircuitConfig::default(),
        );

        Self {
            start_time: Instant::now(),
            load_shedder: LoadShedder::new(config.rss_degraded_mb, config.rss_recover_mb),
            statm_reader: StatmReader::new(Duration::from_secs(2)),
            circuit_registry,
            mcp_state,
            diagnostic_engine,
            policy_gatekeeper,
            dropped_events: AtomicUsize::new(0),
            event_broadcaster: tx,
            config,
        }
    }

    /// Record a dropped event under pressure.
    pub fn increment_dropped(&self) {
        self.dropped_events.fetch_add(1, Ordering::Relaxed);
    }
}
