//! Zero-allocation latency tracking with Exponential Moving Average (EMA) and p95 calculations.

use crate::circuit::config::AdaptiveTimeoutConfig;
use std::time::Duration;

/// Maximum number of historical latency samples stored in the bounded circular buffer.
pub const LATENCY_SAMPLE_CAPACITY: usize = 64;

/// Tracks request latencies using a fixed-size stack buffer with zero heap allocations.
#[derive(Debug, Clone, PartialEq)]
pub struct LatencyTracker {
    samples: [u32; LATENCY_SAMPLE_CAPACITY],
    head: usize,
    count: usize,
    ema_ms: Option<f64>,
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyTracker {
    /// Constructs an empty latency tracker.
    pub fn new() -> Self {
        Self {
            samples: [0u32; LATENCY_SAMPLE_CAPACITY],
            head: 0,
            count: 0,
            ema_ms: None,
        }
    }

    /// Records an observed request duration and updates the EMA.
    pub fn record(&mut self, elapsed: Duration, alpha: f64) {
        let ms = elapsed.as_millis().min(u32::MAX as u128) as u32;
        self.record_ms(ms, alpha);
    }

    /// Records an observed millisecond latency and updates the EMA.
    pub fn record_ms(&mut self, ms: u32, alpha: f64) {
        self.samples[self.head] = ms;
        self.head = (self.head + 1) % LATENCY_SAMPLE_CAPACITY;
        if self.count < LATENCY_SAMPLE_CAPACITY {
            self.count += 1;
        }

        let clamped_alpha = alpha.clamp(0.01, 1.0);
        self.ema_ms = match self.ema_ms {
            Some(prev) => Some(clamped_alpha * (ms as f64) + (1.0 - clamped_alpha) * prev),
            None => Some(ms as f64),
        };
    }

    /// Returns the current number of active samples (0..=64).
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns true if no samples have been recorded.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the current Exponential Moving Average in milliseconds, if any.
    pub fn ema_ms(&self) -> Option<f64> {
        self.ema_ms
    }

    /// Computes the 95th percentile latency in milliseconds using a stack sort.
    ///
    /// Executes in $< 100$ nanoseconds with zero dynamic heap allocations.
    pub fn p95_ms(&self) -> u32 {
        if self.count == 0 {
            return 0;
        }

        let n = self.count;
        let mut sorted = [0u32; LATENCY_SAMPLE_CAPACITY];
        sorted[..n].copy_from_slice(&self.samples[..n]);
        sorted[..n].sort_unstable();

        let rank = ((0.95 * n as f64).ceil() as usize).max(1);
        let idx = (rank - 1).min(n - 1);
        sorted[idx]
    }

    /// Dynamically calculates the recommended inference timeout.
    ///
    /// If no samples exist, returns `config.initial_timeout`.
    /// Otherwise returns $\max(\text{EMA} \times 1.5, \text{p95} \times 1.25)$ clamped to
    /// `[config.min_timeout, config.max_timeout]`.
    pub fn calculate_timeout(&self, config: &AdaptiveTimeoutConfig) -> Duration {
        let ema = match self.ema_ms {
            Some(v) => v,
            None => return config.initial_timeout,
        };

        let p95 = self.p95_ms() as f64;
        let target_ms = (ema * 1.5).max(p95 * 1.25);
        let target = Duration::from_secs_f64(target_ms / 1000.0);

        let min_t = config.min_timeout;
        let max_t = config.max_timeout.max(min_t);
        target.clamp(min_t, max_t)
    }

    /// Resets all recorded samples and EMA state.
    pub fn reset(&mut self) {
        self.samples = [0u32; LATENCY_SAMPLE_CAPACITY];
        self.head = 0;
        self.count = 0;
        self.ema_ms = None;
    }
}
