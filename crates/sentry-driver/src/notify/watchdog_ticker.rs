//! Asynchronous watchdog heartbeat ticker task.

use super::sender::notify_watchdog;
use super::watchdog_config::WatchdogConfig;
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
    pub fn spawn(config: WatchdogConfig) -> Self {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let target_interval = config.interval.max(std::time::Duration::from_micros(1));

        let handle = tokio::spawn(async move {
            let mut ticker = interval(target_interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        match notify_watchdog() {
                            Ok(true) => debug!("Watchdog tick sent successfully"),
                            Ok(false) => {
                                warn!("Watchdog tick skipped: NOTIFY_SOCKET unset");
                                break;
                            }
                            Err(e) => {
                                error!("Failed to dispatch watchdog tick: {e}");
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
