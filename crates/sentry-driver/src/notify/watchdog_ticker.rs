//! Asynchronous watchdog heartbeat ticker task.

use super::socket_addr::resolve_notify_address;
use super::watchdog_config::WatchdogConfig;
use std::os::unix::net::UnixDatagram;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, error, warn};

/// Background task handle that periodically transmits `WATCHDOG=1` heartbeats.
pub struct WatchdogTicker {
    cancel_tx: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl WatchdogTicker {
    /// Spawns an asynchronous watchdog heartbeat loop in Tokio.
    ///
    /// Persists an unbound UnixDatagram socket across ticks to eliminate redundant syscalls.
    pub fn spawn(config: WatchdogConfig) -> Self {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let target_interval = config.interval.max(std::time::Duration::from_micros(1));

        let handle = tokio::spawn(async move {
            let mut ticker = interval(target_interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            let persistent = match std::env::var("NOTIFY_SOCKET") {
                Ok(val) if !val.is_empty() => match resolve_notify_address(&val) {
                    Ok(addr) => match UnixDatagram::unbound() {
                        Ok(sock) => {
                            let _ = sock.set_nonblocking(true);
                            Some((sock, addr))
                        }
                        Err(e) => {
                            error!("Failed to create persistent watchdog datagram socket: {e}");
                            None
                        }
                    },
                    Err(e) => {
                        error!("Failed to resolve NOTIFY_SOCKET for watchdog ticker: {e}");
                        None
                    }
                },
                _ => None,
            };

            let payload = b"WATCHDOG=1\n";

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        match &persistent {
                            Some((sock, addr)) => {
                                match sock.send_to_addr(payload, addr) {
                                    Ok(_) => debug!("Watchdog tick sent successfully"),
                                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                        debug!("Watchdog tick would block; socket buffer full");
                                    }
                                    Err(e) => {
                                        error!("Failed to dispatch watchdog tick: {e}");
                                    }
                                }
                            }
                            None => {
                                warn!("Watchdog tick skipped: NOTIFY_SOCKET unset");
                                break;
                            }
                        }
                    }
                    _ = &mut cancel_rx => {
                        debug!("Watchdog ticker cancelled, exiting heartbeat loop");
                        break;
                    }
                }
            }
        });

        Self {
            cancel_tx: Some(cancel_tx),
            handle,
        }
    }

    /// Stops the ticker and awaits termination of the background task.
    pub async fn stop(mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
        let _ = self.handle.await;
    }
}
