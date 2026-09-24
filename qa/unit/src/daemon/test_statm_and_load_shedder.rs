//! 1:1 QA tests for zero-allocation memory accounting and load-shedding hysteresis.

use sentry_daemon::system::{
    get_cached_page_size, LoadShedder, SheddingTransition, StatmReader,
};
use std::time::Duration;

#[test]
fn test_statm_page_size_and_reading() {
    let ps = get_cached_page_size();
    assert!(ps >= 4096);

    let mut reader = StatmReader::new(Duration::from_millis(100));
    let rss = reader.read_rss_bytes();
    // In any active test runner process, RSS must be strictly positive
    assert!(rss > 0);
}

#[test]
fn test_load_shedder_2mb_hysteresis_window() {
    let mut shedder = LoadShedder::new(13, 11);
    assert!(!shedder.is_degraded());

    // 1. Normal operations under 13 MB
    assert_eq!(
        shedder.evaluate(10 * 1024 * 1024, None),
        SheddingTransition::Unchanged
    );
    assert!(!shedder.is_degraded());

    // 2. Cross 13 MB ceiling -> Enter degraded mode
    assert_eq!(
        shedder.evaluate(13 * 1024 * 1024 + 1024, None),
        SheddingTransition::Entered
    );
    assert!(shedder.is_degraded());

    // 3. Drop to 12 MB (within hysteresis band) -> Stays degraded!
    assert_eq!(
        shedder.evaluate(12 * 1024 * 1024, None),
        SheddingTransition::Unchanged
    );
    assert!(shedder.is_degraded());

    // 4. Drop to <= 11 MB recovery threshold -> Recovers to healthy!
    assert_eq!(
        shedder.evaluate(11 * 1024 * 1024, None),
        SheddingTransition::Exited
    );
    assert!(!shedder.is_degraded());
}

#[test]
fn test_load_shedder_high_psi_trigger() {
    let mut shedder = LoadShedder::new(13, 11);

    // High PSI memory pressure (>30%) trips degraded mode even if RSS is low
    assert_eq!(
        shedder.evaluate(8 * 1024 * 1024, Some(35.5)),
        SheddingTransition::Entered
    );
    assert!(shedder.is_degraded());
}
