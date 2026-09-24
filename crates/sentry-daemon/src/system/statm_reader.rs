//! Zero-allocation process memory accounting using `/proc/self/statm`.
//!
//! Uses a fixed stack buffer and cached page size to avoid heap allocations
//! during critical failure storms.

use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static CACHED_PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);

/// Get the system page size in bytes, caching it on first invocation.
pub fn get_cached_page_size() -> usize {
    let cached = CACHED_PAGE_SIZE.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let ps = rustix::param::page_size();
    CACHED_PAGE_SIZE.store(ps, Ordering::Relaxed);
    ps
}

/// Rate-limited, stack-allocated RSS memory reader.
pub struct StatmReader {
    page_size: usize,
    last_check: Instant,
    cached_rss_bytes: usize,
    check_interval: Duration,
}

impl StatmReader {
    /// Create a new statm reader with a minimum sampling interval.
    pub fn new(interval: Duration) -> Self {
        Self {
            page_size: get_cached_page_size(),
            last_check: Instant::now() - interval,
            cached_rss_bytes: 0,
            check_interval: interval,
        }
    }

    /// Read current Resident Set Size (RSS) in bytes without heap allocations.
    ///
    /// Respects the rate limit; returns cached value if called within `check_interval`.
    pub fn read_rss_bytes(&mut self) -> usize {
        let now = Instant::now();
        if now.duration_since(self.last_check) < self.check_interval && self.cached_rss_bytes > 0 {
            return self.cached_rss_bytes;
        }

        self.last_check = now;

        // Open and read using a fixed 128-byte stack buffer (0 allocations)
        let mut buf = [0u8; 128];
        let Ok(mut file) = File::open("/proc/self/statm") else {
            return self.cached_rss_bytes;
        };

        let Ok(bytes_read) = file.read(&mut buf) else {
            return self.cached_rss_bytes;
        };

        if bytes_read == 0 {
            return self.cached_rss_bytes;
        }

        // Parse second field: resident page count
        let resident_pages = buf[..bytes_read]
            .split(|&b| b == b' ')
            .nth(1)
            .and_then(|slice| std::str::from_utf8(slice).ok())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        self.cached_rss_bytes = resident_pages * self.page_size;
        self.cached_rss_bytes
    }
}
