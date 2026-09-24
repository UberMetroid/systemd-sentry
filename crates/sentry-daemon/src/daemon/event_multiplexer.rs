//! Core event multiplexer using `tokio::select!` for zero busy-waiting.

use crate::daemon::incident_manager::IncidentManager;
use crate::daemon::state::DaemonState;
use crate::ipc::connection_handler::handle_ipc_connection;
use crate::system::{DaemonSignal, RemediationExecutor, SheddingTransition, SignalListener};
use rustix::process::getuid;
use sentry_driver::dbus::DbusEvent;
use sentry_driver::notify::notify_status;
use sentry_safety::policy::load_policy_with_dropins;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Runs the unified async event loop for the supervisor daemon.
pub async fn run_event_loop(
    ipc_listener: UnixListener,
    state: Arc<Mutex<DaemonState>>,
    mut dbus_rx: tokio::sync::mpsc::Receiver<DbusEvent>,
    mut signal_listener: SignalListener,
    mut remediation_executor: RemediationExecutor,
) {
    let daemon_uid = getuid().as_raw();
    let mut housekeeping = tokio::time::interval(Duration::from_secs(2));
    housekeeping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            // 1. Accept incoming IPC client connection
            accept_res = ipc_listener.accept() => {
                match accept_res {
                    Ok((stream, _)) => {
                        let state_clone = Arc::clone(&state);
                        tokio::spawn(async move {
                            handle_ipc_connection(stream, state_clone, daemon_uid).await;
                        });
                    }
                    Err(e) => warn!("IPC accept error: {}", e),
                }
            }

            // 2. Receive D-Bus systemd event
            dbus_opt = dbus_rx.recv() => {
                match dbus_opt {
                    Some(DbusEvent::UnitFailed(event)) => {
                        let state_clone = Arc::clone(&state);
                        tokio::spawn(async move {
                            let exec = RemediationExecutor::new(
                                state_clone.lock().await.policy_gatekeeper.clone()
                            );
                            IncidentManager::handle_unit_failure(event, state_clone, &exec).await;
                        });
                    }
                    Some(_other) => {}
                    None => {}
                }
            }

            // 3. Handle POSIX signal (SIGHUP reload, SIGTERM shutdown)
            sig = signal_listener.recv() => {
                match sig {
                    DaemonSignal::Reload => {
                        info!("Received SIGHUP: reloading policy and drop-in configurations");
                        let mut s = state.lock().await;
                        let base_path = std::path::Path::new(&s.config.policy_path);
                        let dropin_dir = std::path::Path::new(&s.config.policy_dropin_dir);
                        let policy = load_policy_with_dropins(base_path, dropin_dir);
                        s.policy_gatekeeper = sentry_safety::policy::PolicyGatekeeper::new(policy);
                        remediation_executor.update_gatekeeper(s.policy_gatekeeper.clone());
                        info!("Policy successfully reloaded");
                    }
                    DaemonSignal::Shutdown => {
                        info!("Received shutdown signal: initiating clean termination");
                        break;
                    }
                }
            }

            // 4. Housekeeping tick: RSS sampling and load-shedding hysteresis evaluation
            _ = housekeeping.tick() => {
                let mut s = state.lock().await;
                let rss_bytes = s.statm_reader.read_rss_bytes();
                match s.load_shedder.evaluate(rss_bytes, None) {
                    SheddingTransition::Entered => {
                        warn!("RSS reached limit ({} MB); entering degraded load-shedding mode", rss_bytes / (1024 * 1024));
                        let _ = notify_status("Degraded: Load shedding active (RSS > 13MB)");
                        s.circuit_registry.evict_idle_breakers(std::time::Instant::now());
                    }
                    SheddingTransition::Exited => {
                        info!("RSS dropped below threshold ({} MB); exiting degraded mode", rss_bytes / (1024 * 1024));
                        let _ = notify_status("Healthy: Load shedding deactivated (RSS < 11MB)");
                    }
                    SheddingTransition::Unchanged => {}
                }
            }
        }
    }
}
