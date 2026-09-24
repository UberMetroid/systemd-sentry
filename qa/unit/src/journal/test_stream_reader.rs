//! 1:1 Unit QA tests for JournalStreamReader background streamer.

use sentry_driver::journal::JournalStreamReader;

#[tokio::test]
async fn test_journal_stream_reader_receives_entries() {
    let stream_data = b"MESSAGE=test message 1\n_SYSTEMD_UNIT=test1.service\n\nMESSAGE=test message 2\n_SYSTEMD_UNIT=test2.service\n\n";
    let (handle, mut rx) = JournalStreamReader::spawn_buffer_stream(stream_data.to_vec());

    let e1 = rx.recv().await.expect("Must receive entry 1");
    assert_eq!(e1.message().unwrap(), "test message 1");
    assert_eq!(e1.unit(), Some("test1.service"));

    let e2 = rx.recv().await.expect("Must receive entry 2");
    assert_eq!(e2.message().unwrap(), "test message 2");
    assert_eq!(e2.unit(), Some("test2.service"));

    assert!(rx.recv().await.is_none());
    handle.await.unwrap();
}
