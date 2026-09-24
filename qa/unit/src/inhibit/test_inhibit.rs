//! Tests for systemd-inhibit and shutdown watcher.

use sentry_core::models::ShutdownState;
use sentry_driver::inhibit::{InhibitorLock, ShutdownWatcher};
use std::os::unix::io::AsRawFd;

#[test]
fn test_shutdown_watcher_transitions() {
    let watcher = ShutdownWatcher::new();
    assert_eq!(watcher.current_state(), ShutdownState::Normal);
    assert!(!watcher.current_state().is_transitioning());

    // Prepare for shutdown active
    watcher.handle_prepare_for_shutdown(true);
    assert_eq!(watcher.current_state(), ShutdownState::PreparingForShutdown);
    assert!(watcher.current_state().is_transitioning());

    // Shutdown cancelled or finished
    watcher.handle_prepare_for_shutdown(false);
    assert_eq!(watcher.current_state(), ShutdownState::Normal);

    // Prepare for sleep active
    watcher.handle_prepare_for_sleep(true);
    assert_eq!(watcher.current_state(), ShutdownState::PreparingForSleep);
    assert!(watcher.current_state().is_transitioning());

    // Resumed from sleep
    watcher.handle_prepare_for_sleep(false);
    assert_eq!(watcher.current_state(), ShutdownState::Normal);
}

#[test]
fn test_inhibitor_lock_metadata() {
    let file = std::fs::File::open("/dev/null").expect("open /dev/null");
    let rustix_fd = rustix::fd::OwnedFd::from(file);
    let fd = zbus::zvariant::OwnedFd::from(rustix_fd);
    let raw = fd.as_raw_fd();

    let lock = InhibitorLock::from_fd(fd, "shutdown:sleep", "sentry", "Triage in progress");
    assert_eq!(lock.what(), "shutdown:sleep");
    assert_eq!(lock.who(), "sentry");
    assert_eq!(lock.why(), "Triage in progress");
    assert_eq!(lock.fd().as_raw_fd(), raw);
}
