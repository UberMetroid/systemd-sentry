//! Passive listener for logind shutdown and sleep transitions.

use futures_lite::stream::StreamExt;
use sentry_core::models::ShutdownState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use zbus::{Connection, MatchRule, MessageStream};

/// Monitors systemd-logind shutdown and sleep lifecycle signals.
pub struct ShutdownWatcher {
    state_tx: watch::Sender<ShutdownState>,
    state_rx: watch::Receiver<ShutdownState>,
    is_shutting_down: Arc<AtomicBool>,
    is_sleeping: Arc<AtomicBool>,
}

impl ShutdownWatcher {
    /// Creates a new shutdown watcher with baseline `Normal` state.
    pub fn new() -> Self {
        let (state_tx, state_rx) = watch::channel(ShutdownState::Normal);
        Self {
            state_tx,
            state_rx,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns a receiver for current host shutdown/sleep state changes.
    pub fn subscribe(&self) -> watch::Receiver<ShutdownState> {
        self.state_rx.clone()
    }

    /// Current snapshot of host shutdown/sleep state.
    pub fn current_state(&self) -> ShutdownState {
        *self.state_rx.borrow()
    }

    /// Handles a PrepareForShutdown signal argument (`true` = starting, `false` = cancelled).
    pub fn handle_prepare_for_shutdown(&self, active: bool) {
        self.is_shutting_down.store(active, Ordering::SeqCst);
        self.update_state();
    }

    /// Handles a PrepareForSleep signal argument (`true` = sleeping, `false` = resumed).
    pub fn handle_prepare_for_sleep(&self, active: bool) {
        self.is_sleeping.store(active, Ordering::SeqCst);
        self.update_state();
    }

    fn update_state(&self) {
        let state = if self.is_shutting_down.load(Ordering::SeqCst) {
            ShutdownState::PreparingForShutdown
        } else if self.is_sleeping.load(Ordering::SeqCst) {
            ShutdownState::PreparingForSleep
        } else {
            ShutdownState::Normal
        };
        let _ = self.state_tx.send(state);
    }

    /// Runs a background stream listener over the D-Bus connection.
    pub async fn run_listener(
        &self,
        conn: &Connection,
        mut cancel: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), zbus::Error> {
        let rule = MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender("org.freedesktop.login1")?
            .interface("org.freedesktop.login1.Manager")?
            .build();

        let mut stream = MessageStream::for_match_rule(rule, conn, None).await?;

        loop {
            tokio::select! {
                _ = cancel.recv() => break,
                item = stream.next() => {
                    match item {
                        Some(Ok(msg)) => {
                            let header = msg.header();
                            if let Some(member) = header.member() {
                                match member.as_str() {
                                    "PrepareForShutdown" => {
                                        if let Ok((active,)) = msg.body().deserialize::<(bool,)>() {
                                            self.handle_prepare_for_shutdown(active);
                                        }
                                    }
                                    "PrepareForSleep" => {
                                        if let Ok((active,)) = msg.body().deserialize::<(bool,)>() {
                                            self.handle_prepare_for_sleep(active);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(Err(_)) | None => break,
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for ShutdownWatcher {
    fn default() -> Self {
        Self::new()
    }
}
