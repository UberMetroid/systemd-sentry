//! 1:1 Unit QA tests for watchdog heartbeat ticker task.

use super::NOTIFY_ENV_LOCK;
use sentry_driver::notify::{WatchdogConfig, WatchdogTicker};
use std::env;
use std::os::unix::net::UnixDatagram;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_watchdog_ticker_sends_heartbeats() {
    let _guard = NOTIFY_ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let sock_path = dir.path().join("watchdog.sock");
    let receiver = UnixDatagram::bind(&sock_path).unwrap();

    env::set_var("NOTIFY_SOCKET", sock_path.to_str().unwrap());

    let config = WatchdogConfig {
        interval: Duration::from_millis(20),
        raw_usec: 40000,
        target_pid: None,
    };

    let ticker = WatchdogTicker::spawn(config);

    let mut buf = [0u8; 64];
    receiver
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();
    let (n, _) = receiver
        .recv_from(&mut buf)
        .expect("Must receive at least one watchdog heartbeat");
    let msg = std::str::from_utf8(&buf[..n]).unwrap();
    assert_eq!(msg, "WATCHDOG=1\n");

    ticker.stop().await;
    env::remove_var("NOTIFY_SOCKET");
}
