//! Load-shedding controller with hysteresis to prevent thrashing.
//!
//! Enforces degraded operation under high memory pressure (<15MB RSS target).

/// Transition state for load shedding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheddingTransition {
    /// Entered degraded mode.
    Entered,
    /// Recovered back to normal mode.
    Exited,
    /// State unchanged.
    Unchanged,
}

/// Controller managing degraded load-shedding mode with hysteresis.
pub struct LoadShedder {
    degraded_threshold_bytes: usize,
    recover_threshold_bytes: usize,
    is_degraded: bool,
}

impl LoadShedder {
    /// Create a new load shedder with configured byte thresholds.
    pub fn new(degraded_mb: usize, recover_mb: usize) -> Self {
        Self {
            degraded_threshold_bytes: degraded_mb * 1024 * 1024,
            recover_threshold_bytes: recover_mb * 1024 * 1024,
            is_degraded: false,
        }
    }

    /// Check if currently in degraded mode.
    pub fn is_degraded(&self) -> bool {
        self.is_degraded
    }

    /// Evaluate current RSS and optional PSI memory pressure against hysteresis thresholds.
    pub fn evaluate(
        &mut self,
        rss_bytes: usize,
        psi_memory_full_10: Option<f64>,
    ) -> SheddingTransition {
        let high_psi = psi_memory_full_10.map(|v| v > 30.0).unwrap_or(false);
        let low_psi = psi_memory_full_10.map(|v| v < 10.0).unwrap_or(true);

        if !self.is_degraded {
            // Trigger transition into degraded mode
            if rss_bytes >= self.degraded_threshold_bytes || high_psi {
                self.is_degraded = true;
                return SheddingTransition::Entered;
            }
        } else {
            // Recover from degraded mode only after dropping below lower recovery threshold
            if rss_bytes <= self.recover_threshold_bytes && low_psi {
                self.is_degraded = false;
                return SheddingTransition::Exited;
            }
        }

        SheddingTransition::Unchanged
    }
}
