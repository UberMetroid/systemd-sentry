//! Async POSIX signal listener for hot-reloads and clean shutdowns.

use tokio::signal::unix::{signal, SignalKind};

/// Signals handled by the supervisor daemon event loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonSignal {
    /// SIGHUP: Hot reload configuration and drop-in safety policies.
    Reload,
    /// SIGTERM / SIGINT: Gracefully stop daemon and notify systemd.
    Shutdown,
}

/// Listener monitoring asynchronous OS signals.
pub struct SignalListener {
    sighup: tokio::signal::unix::Signal,
    sigterm: tokio::signal::unix::Signal,
    sigint: tokio::signal::unix::Signal,
}

impl SignalListener {
    /// Initialize listener for SIGHUP, SIGTERM, and SIGINT.
    pub fn new() -> Result<Self, std::io::Error> {
        let sighup = signal(SignalKind::hangup())?;
        let sigterm = signal(SignalKind::terminate())?;
        let sigint = signal(SignalKind::interrupt())?;

        Ok(Self { sighup, sigterm, sigint })
    }

    /// Wait asynchronously for the next incoming signal.
    pub async fn recv(&mut self) -> DaemonSignal {
        tokio::select! {
            _ = self.sighup.recv() => DaemonSignal::Reload,
            _ = self.sigterm.recv() => DaemonSignal::Shutdown,
            _ = self.sigint.recv() => DaemonSignal::Shutdown,
        }
    }
}
