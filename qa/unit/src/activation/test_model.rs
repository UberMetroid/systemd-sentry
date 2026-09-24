//! 1:1 Unit QA tests for ActivatedSocket adapters.

use sentry_driver::activation::ActivatedSocket;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

#[tokio::test]
async fn test_activated_socket_into_std_and_tokio_tcp() {
    let std_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let raw_fd = std_listener.as_raw_fd();
    std::mem::forget(std_listener); // Prevent dropping fd before ActivatedSocket takes ownership

    let activated = ActivatedSocket::new(raw_fd, "test-http", 0);
    assert_eq!(activated.name, "test-http");
    assert_eq!(activated.index, 0);

    let tokio_listener = activated
        .into_tokio_tcp_listener()
        .expect("Conversion to Tokio TCP listener must succeed");

    assert!(tokio_listener.local_addr().is_ok());
}
