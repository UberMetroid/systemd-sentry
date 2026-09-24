//! Strongly-typed representation of all `sd_notify` protocol states.

/// Representation of systemd notification states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifyState {
    /// READY=1: Service startup completed.
    Ready,
    /// STATUS=...: Single-line status string.
    Status(String),
    /// WATCHDOG=1: Heartbeat ping.
    Watchdog,
    /// RELOADING=1: Service configuration reload initiated.
    Reloading,
    /// STOPPING=1: Service clean shutdown initiated.
    Stopping,
    /// MAINPID=...: Main process PID.
    MainPid(u32),
    /// ERRNO=...: Errno value on failure.
    Errno(i32),
    /// BUSERROR=...: D-Bus error string on failure.
    BusError(String),
    /// EXTEND_TIMEOUT_USEC=...: Extend startup or shutdown timeout.
    ExtendTimeout(u64),
    /// BARRIER=1: Barrier synchronization.
    Barrier,
    /// Custom key-value notification pair.
    Custom(String, String),
}

impl NotifyState {
    /// Formats the state into a systemd notification key=value string without trailing newline.
    pub fn format_key_value(&self) -> String {
        match self {
            Self::Ready => "READY=1".to_string(),
            Self::Status(s) => format!("STATUS={}", s),
            Self::Watchdog => "WATCHDOG=1".to_string(),
            Self::Reloading => "RELOADING=1".to_string(),
            Self::Stopping => "STOPPING=1".to_string(),
            Self::MainPid(pid) => format!("MAINPID={}", pid),
            Self::Errno(e) => format!("ERRNO={}", e),
            Self::BusError(err) => format!("BUSERROR={}", err),
            Self::ExtendTimeout(usec) => format!("EXTEND_TIMEOUT_USEC={}", usec),
            Self::Barrier => "BARRIER=1".to_string(),
            Self::Custom(k, v) => format!("{}={}", k, v),
        }
    }
}
